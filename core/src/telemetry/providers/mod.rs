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

pub use self::centos::Centos;
pub use self::debian::Debian;
pub use self::fedora::Fedora;
pub use self::freebsd::Freebsd;
pub use self::macos::Macos;
pub use self::nixos::Nixos;
pub use self::ubuntu::Ubuntu;

use errors::*;
use futures::future::{self, Future};
use host::Host;
use provider::Provider;
use super::Telemetry;

pub trait TelemetryProvider<H: Host>: Provider<H> {
    fn load(&self, host: &H) -> Box<Future<Item = Telemetry, Error = Error>>;
}

pub fn factory<H: Host + 'static>(host: &H) -> Box<Future<Item = Telemetry, Error = Error>> {
    let mut providers: Vec<Box<Future<Item = Telemetry, Error = Error>>> = Vec::new();

    let h = host.clone();
    providers.push(Box::new(Centos::try_new(host).and_then(move |o| option_to_result(&h, o))));
    let h = host.clone();
    providers.push(Box::new(Debian::try_new(host).and_then(move |o| option_to_result(&h, o))));
    let h = host.clone();
    providers.push(Box::new(Fedora::try_new(host).and_then(move |o| option_to_result(&h, o))));
    let h = host.clone();
    providers.push(Box::new(Freebsd::try_new(host).and_then(move |o| option_to_result(&h, o))));
    let h = host.clone();
    providers.push(Box::new(Macos::try_new(host).and_then(move |o| option_to_result(&h, o))));
    let h = host.clone();
    providers.push(Box::new(Nixos::try_new(host).and_then(move |o| option_to_result(&h, o))));
    let h = host.clone();
    providers.push(Box::new(Ubuntu::try_new(host).and_then(move |o| option_to_result(&h, o))));

    Box::new(future::select_ok(providers).map(|p| p.0))
}

fn option_to_result<H, P>(host: &H, opt: Option<P>) -> Box<Future<Item = Telemetry, Error = Error>>
    where H: Host,
          P: TelemetryProvider<H>
{
    match opt {
        Some(t) => t.load(host),
        None => Box::new(future::err(ErrorKind::ProviderUnavailable("Telemetry").into()))
    }
}
