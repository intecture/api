// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

mod centos;
mod debian;
mod fedora;
mod freebsd;
mod macos;
mod nixos;
mod ubuntu;

pub use self::centos::{Centos, CentosRunnable};
pub use self::debian::{Debian, DebianRunnable};
pub use self::fedora::{Fedora, FedoraRunnable};
pub use self::freebsd::{Freebsd, FreebsdRunnable};
pub use self::macos::{Macos, MacosRunnable};
pub use self::nixos::{Nixos, NixosRunnable};
pub use self::ubuntu::{Ubuntu, UbuntuRunnable};

use erased_serde::Serialize;
use errors::*;
use futures::future::{self, Future};
use host::Host;
use host::local::Local;
use provider::Provider;
use remote::Executable;
use super::Telemetry;

pub trait TelemetryProvider<H: Host>: Provider<H> {
    fn load(&mut self) -> Box<Future<Item = Telemetry, Error = Error>>;
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

impl Executable for TelemetryRunnable {
    fn exec(self, host: &Local) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            TelemetryRunnable::Centos(p) => p.exec(host),
            TelemetryRunnable::Debian(p) => p.exec(host),
            TelemetryRunnable::Fedora(p) => p.exec(host),
            TelemetryRunnable::Freebsd(p) => p.exec(host),
            TelemetryRunnable::Macos(p) => p.exec(host),
            TelemetryRunnable::Nixos(p) => p.exec(host),
            TelemetryRunnable::Ubuntu(p) => p.exec(host),
        }
    }
}

pub fn factory<H: Host + 'static>(host: &H) -> Box<Future<Item = Telemetry, Error = Error>> {
    let mut providers: Vec<Box<Future<Item = Telemetry, Error = Error>>> = Vec::new();

    providers.push(Box::new(Centos::try_new(host).and_then(option_to_result)));
    providers.push(Box::new(Debian::try_new(host).and_then(option_to_result)));
    providers.push(Box::new(Fedora::try_new(host).and_then(option_to_result)));
    providers.push(Box::new(Freebsd::try_new(host).and_then(option_to_result)));
    providers.push(Box::new(Macos::try_new(host).and_then(option_to_result)));
    providers.push(Box::new(Nixos::try_new(host).and_then(option_to_result)));
    providers.push(Box::new(Ubuntu::try_new(host).and_then(option_to_result)));

    Box::new(future::select_ok(providers).map(|p| p.0))
}

fn option_to_result<H, P>(opt: Option<P>) -> Box<Future<Item = Telemetry, Error = Error>>
    where H: Host,
          P: TelemetryProvider<H>
{
    match opt {
        Some(mut t) => t.load(),
        None => Box::new(future::err(ErrorKind::ProviderUnavailable("Telemetry").into()))
    }
}
