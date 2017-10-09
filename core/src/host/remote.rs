// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use bytes::{BufMut, BytesMut};
use errors::*;
use futures::{future, Future};
use Runnable;
use serde::Deserialize;
use serde_json;
use std::{io, result};
use std::sync::Arc;
use std::net::SocketAddr;
use super::Host;
use telemetry::{self, Telemetry};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_proto::pipeline::{ClientProto, ClientService, ServerProto};
use tokio_proto::TcpClient;
use tokio_service::Service;

pub trait RemoteHost: Host {
    fn connect(addr: &str, handle: &Handle) -> Box<Future<Item = Arc<Self>, Error = Error>>;
}

/// A Host type that uses an unencrypted socket.
///
/// *Warning! This Host type is susceptible to eavesdropping and MITM
/// attacks, and ideally should only be used for testing on secure private
/// networks.*
pub struct Plain {
    inner: ClientService<TcpStream, JsonProto>,
    telemetry: Option<Telemetry>,
}

#[doc(hidden)]
pub struct JsonCodec;
#[doc(hidden)]
pub struct JsonProto;

impl Plain {
    /// Create a new Host connected to addr.
    pub fn connect(addr: &str, handle: &Handle) -> Box<Future<Item = Arc<Plain>, Error = Error>> {
        let addr: SocketAddr = match addr.parse().chain_err(|| "Invalid host address") {
            Ok(addr) => addr,
            Err(e) => return Box::new(future::err(e)),
        };

        info!("Connecting to host {}", addr);

        Box::new(TcpClient::new(JsonProto)
            .connect(&addr, handle)
            .chain_err(|| "Could not connect to host")
            .and_then(|client_service| {
                info!("Connected!");

                let mut host = Arc::new(Plain {
                    inner: client_service,
                    telemetry: None,
                });

                telemetry::load(&host)
                          .chain_err(|| "Could not load telemetry for host")
                          .map(|t| {
                    Arc::get_mut(&mut host).unwrap().telemetry = Some(t);
                    host
                })
            }))
    }
}

impl Host for Plain {
    fn telemetry(&self) -> &Telemetry {
        self.telemetry.as_ref().unwrap()
    }

    fn run<D: 'static>(&self, provider: Runnable) -> Box<Future<Item = D, Error = Error>>
        where for<'de> D: Deserialize<'de>
    {
        let value = match serde_json::to_value(provider).chain_err(|| "Could not encode provider to send to host") {
            Ok(v) => v,
            Err(e) => return Box::new(future::err(e))
        };
        Box::new(self.inner.call(value)
                           .chain_err(|| "Error while running provider on host")
                           .and_then(|v| match serde_json::from_value::<D>(v).chain_err(|| "Could not understand response from host") {
                               Ok(d) => future::ok(d),
                               Err(e) => future::err(e)
                           }))
    }
}

impl Service for Plain {
    type Request = serde_json::Value;
    type Response = serde_json::Value;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(self.inner.call(req))
    }
}

impl Decoder for JsonCodec {
    type Item = serde_json::Value;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> result::Result<Option<Self::Item>, Self::Error> {
        // Check to see if the frame contains a new line
        if let Some(n) = buf.as_ref().iter().position(|b| *b == b'\n') {
            // remove the serialized frame from the buffer.
            let line = buf.split_to(n);

            // Also remove the '\n'
            buf.split_to(1);

            return Ok(Some(serde_json::from_slice(&line).unwrap()));
        }

        Ok(None)
    }
}

impl Encoder for JsonCodec {
    type Item = serde_json::Value;
    type Error = io::Error;

    fn encode(&mut self, value: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        let json = serde_json::to_string(&value).unwrap();
        buf.reserve(json.len() + 1);
        buf.extend(json.as_bytes());
        buf.put_u8(b'\n');

        Ok(())
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for JsonProto {
    type Request = serde_json::Value;
    type Response = serde_json::Value;
    type Transport = Framed<T, JsonCodec>;
    type BindTransport = result::Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(JsonCodec))
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for JsonProto {
    type Request = serde_json::Value;
    type Response = serde_json::Value;
    type Transport = Framed<T, JsonCodec>;
    type BindTransport = result::Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(JsonCodec))
    }
}
