// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate clap;
extern crate env_logger;
#[macro_use] extern crate error_chain;
extern crate futures;
extern crate intecture_api;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate toml;

mod errors;

use error_chain::ChainedError;
use errors::*;
use futures::{future, Future};
use intecture_api::host::local::Local;
use intecture_api::host::remote::{JsonLineProto, LineMessage};
use intecture_api::remote::{Executable, Request, ResponseResult};
use std::fs::File;
use std::io::{self, Read};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_core::reactor::Remote;
use tokio_proto::streaming::Message;
use tokio_proto::TcpServer;
use tokio_service::{NewService, Service};

pub struct Api {
    host: Local,
    remote: Remote,
}

impl Service for Api {
    type Request = LineMessage;
    type Response = LineMessage;
    type Error = Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let req = match req {
            Message::WithBody(req, _) => req,
            Message::WithoutBody(req) => req,
        };

        let request: Request = match serde_json::from_value(req).chain_err(|| "Could not deserialize Request") {
            Ok(r) => r,
            Err(e) => return Box::new(future::ok(error_to_msg(e))),
        };

        // XXX Danger zone! If we're running multiple threads, this `unwrap()`
        // will explode. The API requires a `Handle`, but we can only send a
        // `Remote` to this Service. Currently we force the `Handle`, which is
        // only safe for the current thread.
        // See https://github.com/alexcrichton/tokio-process/issues/23
        let handle = self.remote.handle().unwrap();
        Box::new(request.exec(&self.host, &handle)
            .chain_err(|| "Failed to execute Request")
            .then(|req| {
                match req {
                    Ok(mut msg) => {
                        let body = msg.take_body();
                        match serde_json::to_value(msg.into_inner()).chain_err(|| "Could not serialize Result") {
                            Ok(v) => match body {
                                Some(b) => future::ok(Message::WithBody(v, b)),
                                None => future::ok(Message::WithoutBody(v)),
                            },
                            Err(e) => future::ok(error_to_msg(e)),
                        }
                    },
                    Err(e) => future::ok(error_to_msg(e)),
                }
            }))
    }
}

impl NewService for Api {
    type Request = LineMessage;
    type Response = LineMessage;
    type Error = Error;
    type Instance = Api;
    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(Api {
            host: self.host.clone(),
            remote: self.remote.clone(),
        })
    }
}

#[derive(Deserialize)]
struct Config {
    address: SocketAddr,
}

quick_main!(|| -> Result<()> {
    env_logger::init().chain_err(|| "Could not start logging")?;

    let matches = clap::App::new("Intecture Agent")
                            .version(env!("CARGO_PKG_VERSION"))
                            .author(env!("CARGO_PKG_AUTHORS"))
                            .about(env!("CARGO_PKG_DESCRIPTION"))
                            .arg(clap::Arg::with_name("config")
                                .short("c")
                                .long("config")
                                .value_name("FILE")
                                .help("Path to the agent configuration file")
                                .takes_value(true))
                            .arg(clap::Arg::with_name("addr")
                                .short("a")
                                .long("address")
                                .value_name("ADDR")
                                .help("Set the socket address this server will listen on (e.g. 0.0.0.0:7101)")
                                .takes_value(true))
                            .group(clap::ArgGroup::with_name("config_or_else")
                                .args(&["config", "addr"])
                                .required(true))
                            .get_matches();

    let config = if let Some(c) = matches.value_of("config") {
        let mut fh = File::open(c).chain_err(|| "Could not open config file")?;
        let mut buf = Vec::new();
        fh.read_to_end(&mut buf).chain_err(|| "Could not read config file")?;
        toml::from_slice(&buf).chain_err(|| "Config file contained invalid TOML")?
    } else {
        let address = matches.value_of("addr").unwrap().parse().chain_err(|| "Invalid server address")?;
        Config { address }
    };

    let host = Local::new().wait()?;
    // XXX We can only run a single thread here, or big boom!!
    // The API requires a `Handle`, but we can only send a `Remote`.
    // Currently we force the issue (`unwrap()`), which is only safe
    // for the current thread.
    // See https://github.com/alexcrichton/tokio-process/issues/23
    let server = TcpServer::new(JsonLineProto, config.address);
    server.with_handle(move |handle| {
        let api = Api {
            host: host.clone(),
            remote: handle.remote().clone(),
        };
        Arc::new(api)
    });
    Ok(())
});

fn error_to_msg(e: Error) -> LineMessage {
    let response = ResponseResult::Err(format!("{}", e.display_chain()));
    // If we can't serialize this, we can't serialize anything, so
    // panicking is appropriate.
    let value = serde_json::to_value(response)
        .expect("Cannot serialize ResponseResult::Err. This is bad...");
    Message::WithoutBody(value)
}
