// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! A connection to a remote host.

use bytes::BytesMut;
use errors::*;
use futures::{future, Future};
use remote::{Request, Response, ResponseResult};
use serde_json;
use std::{io, result};
use std::net::SocketAddr;
use std::sync::Arc;
use super::Host;
use telemetry::{self, Telemetry};
use tokio_core::reactor::Handle;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_proto::streaming::{Body, Message};
use tokio_proto::streaming::pipeline::{ClientProto, Frame, ServerProto};
use tokio_proto::TcpClient;
use tokio_proto::util::client_proxy::ClientProxy;
use tokio_service::Service;

#[doc(hidden)]
pub type LineMessage = Message<serde_json::Value, Body<Vec<u8>, io::Error>>;

/// A `Host` type that uses an unencrypted socket.
///
/// >**Warning!** An unencrypted host is susceptible to eavesdropping and MITM
/// attacks, and should only be used on secure private networks.
#[derive(Clone)]
pub struct Plain {
    inner: Arc<Inner>,
    handle: Handle,
}

struct Inner {
    inner: ClientProxy<LineMessage, LineMessage, io::Error>,
    telemetry: Option<Telemetry>,
}

#[doc(hidden)]
pub struct JsonLineCodec {
    decoding_head: bool,
}
#[doc(hidden)]
pub struct JsonLineProto;

impl Plain {
    /// Create a new Host connected to the given address.
    pub fn connect(addr: &str, handle: &Handle) -> Box<Future<Item = Plain, Error = Error>> {
        let addr: SocketAddr = match addr.parse().chain_err(|| "Invalid host address") {
            Ok(addr) => addr,
            Err(e) => return Box::new(future::err(e)),
        };
        let handle = handle.clone();

        info!("Connecting to host {}", addr);

        Box::new(TcpClient::new(JsonLineProto)
            .connect(&addr, &handle)
            .chain_err(|| "Could not connect to host")
            .and_then(move |client_service| {
                info!("Connected!");

                let mut host = Plain {
                    inner: Arc::new(
                        Inner {
                            inner: client_service,
                            telemetry: None,
                        }),
                    handle: handle.clone(),
                };

                telemetry::Telemetry::load(&host)
                    .chain_err(|| "Could not load telemetry for host")
                    .map(|t| {
                        Arc::get_mut(&mut host.inner).unwrap().telemetry = Some(t);
                        host
                    })
            }))
    }
}

impl Host for Plain {
    fn telemetry(&self) -> &Telemetry {
        self.inner.telemetry.as_ref().unwrap()
    }

    #[doc(hidden)]
    fn request_msg(&self, msg: Message<Request, Body<Vec<u8>, io::Error>>) ->
        Box<Future<Item = Message<Response, Body<Vec<u8>, io::Error>>, Error = Error>>
    {
        self.call(msg)
    }
}

impl Service for Plain {
    type Request = Message<Request, Body<Vec<u8>, io::Error>>;
    type Response = Message<Response, Body<Vec<u8>, io::Error>>;
    type Error = Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, mut req: Self::Request) -> Self::Future {
        let body = req.take_body();
        let request = req.into_inner();

        let value = match serde_json::to_value(request).chain_err(|| "Could not encode provider to send to host") {
            Ok(v) => v,
            Err(e) => return Box::new(future::err(e))
        };

        debug!("Sending JSON request: {}", value);

        let json_msg = match body {
            Some(b) => Message::WithBody(value, b),
            None => Message::WithoutBody(value),
        };

        Box::new(self.inner.inner.call(json_msg)
            .chain_err(|| "Error while running provider on host")
            .and_then(|mut msg| {
                let body = msg.take_body();
                let header = msg.into_inner();

                debug!("Received JSON response: {}", header);

                let result: ResponseResult = match serde_json::from_value(header).chain_err(|| "Could not understand response from host") {
                    Ok(d) => d,
                    Err(e) => return Box::new(future::err(e)),
                };

                let msg = match result {
                    ResponseResult::Ok(msg) => msg,
                    ResponseResult::Err(e) => return Box::new(future::err(ErrorKind::Remote(e).into())),
                };
                Box::new(future::ok(match body {
                    Some(b) => Message::WithBody(msg, b),
                    None => Message::WithoutBody(msg),
                }))
            }))
    }
}

impl Decoder for JsonLineCodec {
    type Item = Frame<serde_json::Value, Vec<u8>, io::Error>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        let line = match buf.iter().position(|b| *b == b'\n') {
            Some(n) => buf.split_to(n),
            None => return Ok(None),
        };

        buf.split_to(1);

        if self.decoding_head {
            debug!("Decoding header: {:?}", line);

            // The last byte in this frame is a bool that indicates
            // whether we have a body stream following or not.
            // This byte must exist, or our codec is buggered and
            // panicking is appropriate.
            let (has_body, line) = line.split_last()
                .expect("Missing body byte at end of message frame");

            debug!("Body byte: {:?}", has_body);

            if *has_body == 1 {
                self.decoding_head = false;
            }

            let frame = Frame::Message {
                message: serde_json::from_slice(&line).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, e)
                })?,
                body: *has_body == 1,
            };

            debug!("Decoded header: {:?}", frame);

            Ok(Some(frame))
        } else {
            debug!("Decoding body chunk: {:?}", line);

            let frame = if line.is_empty() {
                self.decoding_head = true;
                Frame::Body { chunk: None }
            } else {
                Frame::Body { chunk: Some(line.to_vec()) }
            };

            debug!("Decoded body chunk: {:?}", frame);

            Ok(Some(frame))
        }
    }
}

impl Encoder for JsonLineCodec {
    type Item = Frame<serde_json::Value, Vec<u8>, io::Error>;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        match msg {
            Frame::Message { message, body } => {
                debug!("Encoding header: {:?}, {:?}", message, body);

                let json = serde_json::to_vec(&message)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                buf.extend(&json);
                // Add 'has-body' flag
                buf.extend(if body { &[1] } else { &[0] });
            }
            Frame::Body { chunk } => {
                debug!("Encoding chunk: {:?}", chunk);

                if let Some(chunk) = chunk {
                    buf.extend(&chunk);
                }
            }
            Frame::Error { error } => {
                // @todo Support error frames
                return Err(error)
            }
        }

        buf.extend(b"\n");

        Ok(())
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for JsonLineProto {
    type Request = serde_json::Value;
    type RequestBody = Vec<u8>;
    type Response = serde_json::Value;
    type ResponseBody = Vec<u8>;
    type Error = io::Error;
    type Transport = Framed<T, JsonLineCodec>;
    type BindTransport = result::Result<Self::Transport, Self::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        let codec = JsonLineCodec {
            decoding_head: true,
        };

        Ok(io.framed(codec))
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for JsonLineProto {
    type Request = serde_json::Value;
    type RequestBody = Vec<u8>;
    type Response = serde_json::Value;
    type ResponseBody = Vec<u8>;
    type Error = io::Error;
    type Transport = Framed<T, JsonLineCodec>;
    type BindTransport = result::Result<Self::Transport, Self::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        let codec = JsonLineCodec {
            decoding_head: true,
        };

        Ok(io.framed(codec))
    }
}
