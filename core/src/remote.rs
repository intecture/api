// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

// Hopefully in the near future this will be auto-generated from `derive` attributes.

use errors::*;
use futures::Future;
use host::local::Local;
use std::io;
use telemetry::serializable::Telemetry;
use tokio_core::reactor::Handle;
use tokio_proto::streaming::{Body, Message};

pub type ExecutableResult = Box<Future<Item = Message<ResponseResult, Body<Vec<u8>, io::Error>>, Error = Error>>;

pub trait Executable {
    fn exec(self, &Local, &Handle) -> ExecutableResult;
}

#[derive(Serialize, Deserialize)]
pub enum Request {
    Command(CommandRequest),
    Telemetry(TelemetryRequest),
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Command(CommandResponse),
    Telemetry(TelemetryResponse),
}

#[derive(Serialize, Deserialize)]
pub enum ResponseResult {
    Ok(Response),
    Err(String),
}

impl Executable for Request {
    fn exec(self, host: &Local, handle: &Handle) -> ExecutableResult {
        match self {
            Request::Command(p) => p.exec(host, handle),
            Request::Telemetry(p) => p.exec(host, handle),
        }
    }
}

//
// Command
//

#[derive(Serialize, Deserialize)]
pub enum CommandRequest {
    Generic(GenericRequest),
}

#[derive(Serialize, Deserialize)]
pub enum GenericRequest {
    Available,
    Exec(String, Vec<String>),
}

#[derive(Serialize, Deserialize)]
pub enum CommandResponse {
    Available(bool),
    Exec,
}

impl Executable for CommandRequest {
    fn exec(self, host: &Local, handle: &Handle) -> ExecutableResult {
        match self {
            CommandRequest::Generic(p) => p.exec(host, handle)
        }
    }
}

//
// Telemetry
//

#[derive(Serialize, Deserialize)]
pub enum TelemetryRequest {
    Centos(CentosRequest),
    Debian(DebianRequest),
    Fedora(FedoraRequest),
    Freebsd(FreebsdRequest),
    Macos(MacosRequest),
    Nixos(NixosRequest),
    Ubuntu(UbuntuRequest),
}

#[derive(Serialize, Deserialize)]
pub enum CentosRequest {
    Available,
    Load,
}

#[derive(Serialize, Deserialize)]
pub enum DebianRequest {
    Available,
    Load,
}

#[derive(Serialize, Deserialize)]
pub enum FedoraRequest {
    Available,
    Load,
}

#[derive(Serialize, Deserialize)]
pub enum FreebsdRequest {
    Available,
    Load,
}

#[derive(Serialize, Deserialize)]
pub enum MacosRequest {
    Available,
    Load,
}

#[derive(Serialize, Deserialize)]
pub enum NixosRequest {
    Available,
    Load,
}

#[derive(Serialize, Deserialize)]
pub enum UbuntuRequest {
    Available,
    Load,
}

#[derive(Serialize, Deserialize)]
pub enum TelemetryResponse {
    Available(bool),
    Load(Telemetry),
}

impl Executable for TelemetryRequest {
    fn exec(self, host: &Local, handle: &Handle) -> ExecutableResult {
        match self {
            TelemetryRequest::Centos(p) => p.exec(host, handle),
            TelemetryRequest::Debian(p) => p.exec(host, handle),
            TelemetryRequest::Fedora(p) => p.exec(host, handle),
            TelemetryRequest::Freebsd(p) => p.exec(host, handle),
            TelemetryRequest::Macos(p) => p.exec(host, handle),
            TelemetryRequest::Nixos(p) => p.exec(host, handle),
            TelemetryRequest::Ubuntu(p) => p.exec(host, handle),
        }
    }
}
