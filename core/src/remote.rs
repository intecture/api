// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

// Hopefully in the near future this will be auto-generated.

use command;
use errors::*;
use futures::{future, Future};
use std::io;
use telemetry;
use tokio_core::reactor::Handle;
use tokio_proto::streaming::{Body, Message};

pub type ExecutableResult = Box<Future<Item = Message<ResponseResult, Body<Vec<u8>, io::Error>>, Error = Error>>;

#[derive(Serialize, Deserialize)]
pub enum Request {
    CommandExec(Option<ProviderName>, String, Vec<String>),
    TelemetryLoad,
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Null,
    TelemetryLoad(telemetry::serializable::Telemetry),
}

#[derive(Serialize, Deserialize)]
pub enum ResponseResult {
    Ok(Response),
    Err(String),
}

#[derive(Serialize, Deserialize)]
pub enum ProviderName {
    CommandGeneric,
    TelemetryCentos,
    TelemetryDebian,
    TelemetryFedora,
    TelemetryFreebsd,
    TelemetryMacos,
    TelemetryNixos,
    TelemetryUbuntu,
}

pub trait Executable {
    fn exec(self, &Handle) -> ExecutableResult;
}

impl Executable for Request {
    fn exec(self, handle: &Handle) -> ExecutableResult {
        match self {
            Request::CommandExec(provider, cmd, shell) => {
                let provider = match provider {
                    Some(ProviderName::CommandGeneric) => Box::new(command::providers::Generic),
                    None => match command::providers::factory() {
                        Ok(p) => p,
                        Err(e) => return Box::new(future::err(e)),
                    },
                    _ => unreachable!(),
                };
                provider.exec(handle, &cmd, &shell)
            }

            Request::TelemetryLoad => {
                let provider = match telemetry::providers::factory() {
                    Ok(p) => p,
                    Err(e) => return Box::new(future::err(e)),
                };
                provider.load()
            }
        }
    }
}
