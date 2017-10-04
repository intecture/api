// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Manages the connection between the API and your host.

use bytes::{BufMut, BytesMut};
use errors::*;
use futures::Future;
use RemoteProvider;
use serde::Deserialize;
use serde_json;
use std::{io, result};
use std::net::SocketAddr;
use telemetry::{self, Telemetry};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_proto::pipeline::{ClientProto, ClientService};
use tokio_proto::TcpClient;
use tokio_service::Service;

pub struct Host {
    telemetry: Option<Telemetry>,
}

pub struct RemoteHost {
    inner: ClientService<TcpStream, JsonProto>,
    telemetry: Option<Telemetry>,
}

struct JsonCodec;
struct JsonProto;

impl Host {
    /// Create a new Host targeting the local machine.
    pub fn local() -> Result<Host> {
        let mut host = Host {
            telemetry: None,
        };

        host.telemetry = Some(telemetry::load(&host)?);

        Ok(host)
    }

    /// Retrieve Telemetry
    pub fn telemetry(&self) -> &Telemetry {
        self.telemetry.as_ref().unwrap()
    }

    pub fn is_local(&self) -> bool {
        true
    }
}

impl RemoteHost {
    /// Create a new Host connected to addr.
    pub fn connect(addr: &SocketAddr, handle: &Handle) -> Box<Future<Item = RemoteHost, Error = io::Error>> {
        let ret = TcpClient::new(JsonProto)
            .connect(addr, handle)
            .map(|client_service| {
                RemoteHost {
                    inner: client_service,
                    telemetry: None,
                }
            });

        Box::new(ret)
    }

    /// Retrieve Telemetry
    pub fn telemetry(&self) -> &Telemetry {
        self.telemetry.as_ref().unwrap()
    }

    pub fn is_local(&self) -> bool {
        false
    }

    pub fn run<T>(&self, provider: RemoteProvider) -> Box<Future<Item = T, Error = io::Error>>
        where for<'de> T: Deserialize<'de>
    {
        Box::new(self.inner.call(provider)
                           .map(|v| serde_json::from_value::<T>(v).unwrap()))
    }
}

impl Service for RemoteHost {
    type Request = RemoteProvider;
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
    type Item = RemoteProvider;
    type Error = io::Error;

    fn encode(&mut self, provider: Self::Item, buf: &mut BytesMut) -> io::Result<()> {
        let json = serde_json::to_string(&provider).unwrap();
        buf.reserve(json.len() + 1);
        buf.extend(json.as_bytes());
        buf.put_u8(b'\n');

        Ok(())
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for JsonProto {
    type Request = RemoteProvider;
    type Response = serde_json::Value;
    type Transport = Framed<T, JsonCodec>;
    type BindTransport = result::Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(JsonCodec))
    }
}
