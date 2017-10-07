// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! System generated data about your host.

pub mod providers;
mod serializable;

use erased_serde::Serialize;
use errors::*;
use Executable;
use futures::future::{self, Future, FutureResult};
use host::Host;
use pnet::datalink::NetworkInterface;
use self::providers::{Centos, CentosRunnable,
                      Debian, DebianRunnable,
                      Fedora, FedoraRunnable,
                      Freebsd, FreebsdRunnable,
                      Macos, MacosRunnable,
                      Nixos, NixosRunnable,
                      Ubuntu, UbuntuRunnable};
use std::sync::Arc;

pub trait TelemetryProvider<H: Host> {
    fn available(&Arc<H>) -> Box<Future<Item = bool, Error = Error>>;
    fn try_load(&Arc<H>) -> Box<Future<Item = Option<Telemetry>, Error = Error>>;
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum TelemetryRunnable {
    Centos(CentosRunnable),
    Debian(DebianRunnable),
    Fedora(FedoraRunnable),
    Freebsd(FreebsdRunnable),
    Macos(MacosRunnable),
    Nixos(NixosRunnable),
    Ubuntu(UbuntuRunnable),
}

#[derive(Debug)]
pub struct Telemetry {
    pub cpu: Cpu,
    pub fs: Vec<FsMount>,
    pub hostname: String,
    pub memory: u64,
    pub net: Vec<NetworkInterface>,
    pub os: Os,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cpu {
    pub vendor: String,
    pub brand_string: String,
    pub cores: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FsMount {
    pub filesystem: String,
    pub mountpoint: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub capacity: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Os {
    pub arch: String,
    pub family: OsFamily,
    pub platform: OsPlatform,
    pub version_str: String,
    pub version_maj: u32,
    pub version_min: u32,
    pub version_patch: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OsFamily {
    Bsd,
    Darwin,
    Linux,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OsPlatform {
    Centos,
    Debian,
    Fedora,
    Freebsd,
    Macos,
    Nixos,
    Ubuntu,
}

impl Executable for TelemetryRunnable {
    fn exec(self) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            TelemetryRunnable::Centos(p) => p.exec(),
            TelemetryRunnable::Debian(p) => p.exec(),
            TelemetryRunnable::Fedora(p) => p.exec(),
            TelemetryRunnable::Freebsd(p) => p.exec(),
            TelemetryRunnable::Macos(p) => p.exec(),
            TelemetryRunnable::Nixos(p) => p.exec(),
            TelemetryRunnable::Ubuntu(p) => p.exec(),
        }
    }
}

pub fn load<H: Host + 'static>(host: &Arc<H>) -> Box<Future<Item = Telemetry, Error = Error>> {
    let mut providers = Vec::new();

    providers.push(Centos::try_load(host).and_then(option_to_result));
    providers.push(Debian::try_load(host).and_then(option_to_result));
    providers.push(Fedora::try_load(host).and_then(option_to_result));
    providers.push(Freebsd::try_load(host).and_then(option_to_result));
    providers.push(Macos::try_load(host).and_then(option_to_result));
    providers.push(Nixos::try_load(host).and_then(option_to_result));
    providers.push(Ubuntu::try_load(host).and_then(option_to_result));

    Box::new(future::select_ok(providers).map(|p| p.0))
}

fn option_to_result(opt: Option<Telemetry>) -> FutureResult<Telemetry, Error> {
    match opt {
        Some(t) => future::ok(t),
        None => future::err(ErrorKind::ProviderUnavailable("Telemetry").into())
    }
}
