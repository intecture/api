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
extern crate tokio_proto;
extern crate tokio_service;
extern crate toml;

mod errors;

use errors::*;
use futures::{future, Future};
use intecture_api::{Executable, Runnable};
use intecture_api::host::remote::JsonProto;
use std::fs::File;
use std::io::{self, Read};
use std::net::SocketAddr;
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

    let server = TcpServer::new(JsonProto, config.address);
    server.serve(|| Ok(Api));
    Ok(())
});
