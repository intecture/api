// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use bytes::{BufMut, BytesMut};
use errors::*;
use futures::{future, Future};
use remote::Runnable;
use serde::Deserialize;
use serde_json;
use std::{io, result};
use std::sync::Arc;
use std::net::SocketAddr;
use super::{Host, HostType};
use telemetry::{self, Telemetry};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_proto::pipeline::{ClientProto, ClientService, ServerProto};
use tokio_proto::TcpClient;
use tokio_service::Service;

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
    inner: ClientService<TcpStream, JsonProto>,
    telemetry: Option<Telemetry>,
}

pub struct JsonCodec;
pub struct JsonProto;

impl Plain {
    /// Create a new Host connected to addr.
    pub fn connect(addr: &str, handle: &Handle) -> Box<Future<Item = Plain, Error = Error>> {
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
    pub fn run<D: 'static>(&self, provider: Runnable) -> Box<Future<Item = D, Error = Error>>
        where for<'de> D: Deserialize<'de>
    {
        let value = match serde_json::to_value(provider).chain_err(|| "Could not encode provider to send to host") {
            Ok(v) => v,
            Err(e) => return Box::new(future::err(e))
        };
        Box::new(self.call(value)
            .chain_err(|| "Error while running provider on host")
            .and_then(|v| match serde_json::from_value::<D>(v).chain_err(|| "Could not understand response from host") {
                Ok(d) => future::ok(d),
                Err(e) => future::err(e)
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
    type Request = serde_json::Value;
    type Response = serde_json::Value;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(self.inner.inner.call(req)) as Self::Future
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
