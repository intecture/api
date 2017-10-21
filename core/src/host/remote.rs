// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use bytes::BytesMut;
use errors::*;
use futures::{future, Future};
use remote::Request;
use serde::Deserialize;
use serde_json;
use std::{io, result};
use std::sync::Arc;
use std::net::SocketAddr;
use super::{Host, HostType};
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
/// *Warning! An unencrypted host is susceptible to eavesdropping and MITM
/// attacks, and ideally should only be used for testing on secure private
/// networks.*
#[derive(Clone)]
pub struct Plain {
    inner: Arc<Inner>,
}

struct Inner {
    inner: ClientProxy<LineMessage, LineMessage, io::Error>,
    telemetry: Option<Telemetry>,
}

pub struct JsonLineCodec {
    decoding_head: bool,
}
pub struct JsonLineProto;

impl Plain {
    /// Create a new Host connected to addr.
    pub fn connect(addr: &str, handle: &Handle) -> Box<Future<Item = Plain, Error = Error>> {
        let addr: SocketAddr = match addr.parse().chain_err(|| "Invalid host address") {
            Ok(addr) => addr,
            Err(e) => return Box::new(future::err(e)),
        };

        info!("Connecting to host {}", addr);

        Box::new(TcpClient::new(JsonLineProto)
            .connect(&addr, handle)
            .chain_err(|| "Could not connect to host")
            .and_then(|client_service| {
                info!("Connected!");

                let mut host = Plain {
                    inner: Arc::new(
                        Inner {
                            inner: client_service,
                            telemetry: None,
                        }),
                };

                telemetry::providers::factory(&host)
                    .chain_err(|| "Could not load telemetry for host")
                    .map(|t| {
                        Arc::get_mut(&mut host.inner).unwrap().telemetry = Some(t);
                        host
                    })
            }))
    }

    #[doc(hidden)]
    pub fn run<D: 'static>(&self, provider: Request) -> Box<Future<Item = D, Error = Error>>
        where for<'de> D: Deserialize<'de>
    {
        Box::new(self.run_msg::<D>(provider)
            .map(|msg| msg.into_inner()))
    }

    #[doc(hidden)]
    pub fn run_msg<D: 'static>(&self, provider: Request) -> Box<Future<Item = Message<D, Body<Vec<u8>, io::Error>>, Error = Error>>
        where for<'de> D: Deserialize<'de>
    {
        let value = match serde_json::to_value(provider).chain_err(|| "Could not encode provider to send to host") {
            Ok(v) => v,
            Err(e) => return Box::new(future::err(e))
        };
        Box::new(self.inner.inner.call(Message::WithoutBody(value))
            .chain_err(|| "Error while running provider on host")
            .and_then(|mut msg| {
                let body = msg.take_body();
                let msg = match serde_json::from_value::<D>(msg.into_inner()).chain_err(|| "Could not understand response from host") {
                    Ok(d) => d,
                    Err(e) => return Box::new(future::err(e)),
                };
                Box::new(future::ok(match body {
                    Some(b) => Message::WithBody(msg, b),
                    None => Message::WithoutBody(msg),
                }))
            }))
    }
}

impl Host for Plain {
    fn telemetry(&self) -> &Telemetry {
        self.inner.telemetry.as_ref().unwrap()
    }

    fn get_type<'a>(&'a self) -> HostType<'a> {
        HostType::Remote(&self)
    }
}

impl Service for Plain {
    type Request = LineMessage;
    type Response = LineMessage;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(self.inner.inner.call(req)) as Self::Future
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

        if line.is_empty() {
            let decoding_head = self.decoding_head;
            self.decoding_head = !decoding_head;

            if decoding_head {
                Ok(Some(Frame::Message {
                    message: serde_json::Value::Null,
                    body: true,
                }))
            } else {
                Ok(Some(Frame::Body {
                    chunk: None
                }))
            }
        } else {
            if self.decoding_head {
                Ok(Some(Frame::Message {
                    message: serde_json::from_slice(&line).map_err(|e| {
                        io::Error::new(io::ErrorKind::Other, e)
                    })?,
                    body: false,
                }))
            } else {
                Ok(Some(Frame::Body {
                    chunk: Some(line.to_vec()),
                }))
            }
        }
    }
}

impl Encoder for JsonLineCodec {
    type Item = Frame<serde_json::Value, Vec<u8>, io::Error>;
    type Error = io::Error;

    fn encode(&mut self, msg: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        match msg {
            Frame::Message { message, body } => {
                // Our protocol dictates that a message head that
                // includes a streaming body is an empty string.
                assert!(message.is_null() == body);

                let json = serde_json::to_vec(&message)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                buf.extend(&json);
            }
            Frame::Body { chunk } => {
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
