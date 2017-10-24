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
use remote::{DebianRequest, Executable, ExecutableResult, Request, Response,
             ResponseResult, TelemetryRequest, TelemetryResponse};
use std::{env, process, str};
use super::TelemetryProvider;
use target::{default, linux};
use target::linux::LinuxFlavour;
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry};
use tokio_core::reactor::Handle;
use tokio_proto::streaming::Message;

pub struct Debian;
struct LocalDebian;
struct RemoteDebian;

impl<H: Host + 'static> Provider<H> for Debian {
    fn available(host: &H) -> Box<Future<Item = bool, Error = Error>> {
        match host.get_type() {
            HostType::Local(_) => LocalDebian::available(),
            HostType::Remote(r) => RemoteDebian::available(r),
        }
    }

    fn try_new(host: &H) -> Box<Future<Item = Option<Debian>, Error = Error>> {
        let host = host.clone();
        Box::new(Self::available(&host)
            .and_then(|available| {
                if available {
                    future::ok(Some(Debian))
                } else {
                    future::ok(None)
                }
            }))
    }
}

impl<H: Host + 'static> TelemetryProvider<H> for Debian {
    fn load(&self, host: &H) -> Box<Future<Item = Telemetry, Error = Error>> {
        match host.get_type() {
            HostType::Local(_) => LocalDebian::load(),
            HostType::Remote(r) => RemoteDebian::load(r),
        }
    }
}

impl LocalDebian {
    fn available() -> Box<Future<Item = bool, Error = Error>> {
        Box::new(future::ok(cfg!(target_os="linux") && linux::fingerprint_os() == Some(LinuxFlavour::Debian)))
    }

    fn load() -> Box<Future<Item = Telemetry, Error = Error>> {
        Box::new(future::lazy(|| match do_load() {
            Ok(t) => future::ok(t),
            Err(e) => future::err(e),
        }))
    }
}

impl RemoteDebian {
    fn available(host: &Plain) -> Box<Future<Item = bool, Error = Error>> {
        let runnable = Request::Telemetry(
                           TelemetryRequest::Debian(
                               DebianRequest::Available));
        Box::new(host.call_req(runnable)
            .chain_err(|| ErrorKind::Request { endpoint: "Telemetry::Debian", func: "available" })
            .map(|msg| match msg.into_inner() {
                Response::Telemetry(TelemetryResponse::Available(b)) => b,
                _ => unreachable!(),
            }))
    }

    fn load(host: &Plain) -> Box<Future<Item = Telemetry, Error = Error>> {
        let runnable = Request::Telemetry(
                           TelemetryRequest::Debian(
                               DebianRequest::Load));
        Box::new(host.call_req(runnable)
            .chain_err(|| ErrorKind::Request { endpoint: "Telemetry::Debian", func: "load" })
            .map(|msg| match msg.into_inner() {
                Response::Telemetry(TelemetryResponse::Load(t)) => Telemetry::from(t),
                _ => unreachable!(),
            }))
    }
}

impl Executable for DebianRequest {
    fn exec(self, _: &Local, _: &Handle) -> ExecutableResult {
        match self {
            DebianRequest::Available => Box::new(
                LocalDebian::available()
                    .map(|b| Message::WithoutBody(
                        ResponseResult::Ok(
                            Response::Telemetry(
                                TelemetryResponse::Available(b)))))),
            DebianRequest::Load => Box::new(
                LocalDebian::load()
                    .map(|t| Message::WithoutBody(
                        ResponseResult::Ok(
                            Response::Telemetry(
                                TelemetryResponse::Load(t.into()))))
                ))
        }
    }
}

fn do_load() -> Result<Telemetry> {
    let (version_str, version_maj, version_min) = version()?;

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
            platform: OsPlatform::Debian,
            version_str: version_str,
            version_maj: version_maj,
            version_min: version_min,
            version_patch: 0
        },
    })
}

fn version() -> Result<(String, u32, u32)> {
    let out = process::Command::new("lsb_release")
                               .arg("-sr")
                               .output()
                               .chain_err(|| ErrorKind::SystemCommand("lsb_release"))?;
    let version_str = str::from_utf8(&out.stdout)
                          .chain_err(|| ErrorKind::SystemCommandOutput("lsb_release"))?
                          .trim()
                          .to_owned();
    let (maj, min) = {
        let mut parts = version_str.split('.');
        let errstr = format!("Expected OS version format `u32.u32`, got: '{}'", version_str);
        (
            parts.next().ok_or(&*errstr)?.parse().chain_err(|| ErrorKind::SystemCommandOutput("sw_vers"))?,
            parts.next().ok_or(&*errstr)?.parse().chain_err(|| ErrorKind::SystemCommandOutput("sw_vers"))?
        )
    };
    Ok((version_str, maj, min))
}
