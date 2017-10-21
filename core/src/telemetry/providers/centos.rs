// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use futures::{future, Future};
use host::{Host, HostType};
use host::local::Local;
use host::remote::Plain;
use pnet::datalink::interfaces;
use provider::Provider;
use remote::{CentosRequest, Executable, ExecutableResult, Request, Response,
             ResponseResult, TelemetryRequest, TelemetryResponse};
use std::env;
use super::TelemetryProvider;
use target::{default, linux, redhat};
use target::linux::LinuxFlavour;
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry, serializable};
use tokio_core::reactor::Handle;
use tokio_proto::streaming::Message;

pub struct Centos;
struct LocalCentos;
struct RemoteCentos;

impl<H: Host + 'static> Provider<H> for Centos {
    fn available(host: &H) -> Box<Future<Item = bool, Error = Error>> {
        match host.get_type() {
            HostType::Local(_) => LocalCentos::available(),
            HostType::Remote(r) => RemoteCentos::available(r),
        }
    }

    fn try_new(host: &H) -> Box<Future<Item = Option<Centos>, Error = Error>> {
        let host = host.clone();
        Box::new(Self::available(&host)
            .and_then(|available| {
                if available {
                    future::ok(Some(Centos))
                } else {
                    future::ok(None)
                }
            }))
    }
}

impl<H: Host + 'static> TelemetryProvider<H> for Centos {
    fn load(&self, host: &H) -> Box<Future<Item = Telemetry, Error = Error>> {
        match host.get_type() {
            HostType::Local(_) => LocalCentos::load(),
            HostType::Remote(r) => RemoteCentos::load(r),
        }
    }
}

impl LocalCentos {
    fn available() -> Box<Future<Item = bool, Error = Error>> {
        Box::new(future::ok(cfg!(target_os="linux") && linux::fingerprint_os() == Some(LinuxFlavour::Centos)))
    }

    fn load() -> Box<Future<Item = Telemetry, Error = Error>> {
        Box::new(future::lazy(|| match do_load() {
            Ok(t) => future::ok(t),
            Err(e) => future::err(e),
        }))
    }
}

impl RemoteCentos {
    fn available(host: &Plain) -> Box<Future<Item = bool, Error = Error>> {
        let runnable = Request::Telemetry(
                           TelemetryRequest::Centos(
                               CentosRequest::Available));
        host.run(runnable)
            .chain_err(|| ErrorKind::Request { endpoint: "Telemetry::Centos", func: "available" })
    }

    fn load(host: &Plain) -> Box<Future<Item = Telemetry, Error = Error>> {
        let runnable = Request::Telemetry(
                           TelemetryRequest::Centos(
                               CentosRequest::Load));
        Box::new(host.run(runnable)
            .chain_err(|| ErrorKind::Request { endpoint: "Telemetry::Centos", func: "load" })
            .map(|t: serializable::Telemetry| Telemetry::from(t)))
    }
}

impl Executable for CentosRequest {
    fn exec(self, _: &Local, _: &Handle) -> ExecutableResult {
        match self {
            CentosRequest::Available => Box::new(
                LocalCentos::available()
                    .map(|b| Message::WithoutBody(
                        ResponseResult::Ok(
                            Response::Telemetry(
                                TelemetryResponse::Available(b)))))),
            CentosRequest::Load => Box::new(
                LocalCentos::load()
                    .map(|t| Message::WithoutBody(
                        ResponseResult::Ok(
                            Response::Telemetry(
                                TelemetryResponse::Load(t.into()))))
                ))
        }
    }
}

fn do_load() -> Result<Telemetry> {
    let (version_str, version_maj, version_min, version_patch) = redhat::version()?;

    Ok(Telemetry {
        cpu: Cpu {
            vendor: linux::cpu_vendor()?,
            brand_string: linux::cpu_brand_string()?,
            cores: linux::cpu_cores()?,
        },
        fs: default::fs().chain_err(|| "could not resolve telemetry data")?,
        hostname: default::hostname()?,
        memory: linux::memory().chain_err(|| "could not resolve telemetry data")?,
        net: interfaces(),
        os: Os {
            arch: env::consts::ARCH.into(),
            family: OsFamily::Linux,
            platform: OsPlatform::Centos,
            version_str: version_str,
            version_maj: version_maj,
            version_min: version_min,
            version_patch: version_patch
        },
    })
}
