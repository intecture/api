// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

#![recursion_limit = "1024"]

extern crate env_logger;
#[macro_use] extern crate error_chain;
extern crate futures;
extern crate intecture_api;
extern crate serde_json;
extern crate tokio_proto;
extern crate tokio_service;

mod errors;

use errors::*;
use futures::{future, Future};
use intecture_api::{Executable, Runnable};
use intecture_api::host::remote::JsonProto;
use std::io;
use tokio_proto::TcpServer;
use tokio_service::Service;

pub struct Api;

impl Service for Api {
    type Request = serde_json::Value;
    type Response = serde_json::Value;
    type Error = io::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let runnable: Runnable = match serde_json::from_value(req).chain_err(|| "Received invalid Runnable") {
            Ok(r) => r,
            Err(e) => return Box::new(
                future::err(
                    io::Error::new(
                        // @todo Can't wrap 'e' as error_chain Error doesn't derive Sync.
                        // Waiting for https://github.com/rust-lang-nursery/error-chain/pull/163
                        io::ErrorKind::Other, e.description()
                    ))),
        };
        Box::new(runnable.exec()
            // @todo Can't wrap 'e' as error_chain Error doesn't derive Sync.
            // Waiting for https://github.com/rust-lang-nursery/error-chain/pull/163
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.description()))
            .and_then(|ser| match serde_json::to_value(ser).chain_err(|| "Could not serialize result") {
                Ok(v) => future::ok(v),
                Err(e) => future::err(io::Error::new(io::ErrorKind::Other, e.description())),
            }))
    }
}

quick_main!(|| -> Result<()> {
    env_logger::init().chain_err(|| "Could not start logging")?;

    let addr = "127.0.0.1:7101".parse().chain_err(|| "Invalid server address")?;
    let server = TcpServer::new(JsonProto, addr);
    server.serve(|| Ok(Api));
    Ok(())
});
