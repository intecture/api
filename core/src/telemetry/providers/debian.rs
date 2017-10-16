// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use erased_serde::Serialize;
use errors::*;
use remote::{Executable, Runnable};
use futures::{future, Future};
use host::{Host, HostType};
use host::local::Local;
use host::remote::Plain;
use pnet::datalink::interfaces;
use provider::Provider;
use std::{env, process, str};
use super::{TelemetryProvider, TelemetryRunnable};
use target::{default, linux};
use target::linux::LinuxFlavour;
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry, serializable};

pub struct Debian<H: Host> {
    host: H,
}

struct LocalDebian;
struct RemoteDebian;

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum DebianRunnable {
    Available,
    Load,
}

impl<H: Host + 'static> Provider<H> for Debian<H> {
    fn available(host: &H) -> Box<Future<Item = bool, Error = Error>> {
        match host.get_type() {
            HostType::Local(l) => LocalDebian::available(l),
            HostType::Remote(r) => RemoteDebian::available(r),
        }
    }

    fn try_new(host: &H) -> Box<Future<Item = Option<Debian<H>>, Error = Error>> {
        let host = host.clone();
        Box::new(Self::available(&host)
            .and_then(|available| {
                if available {
                    future::ok(Some(Debian { host }))
                } else {
                    future::ok(None)
                }
            }))
    }
}

impl<H: Host + 'static> TelemetryProvider<H> for Debian<H> {
    fn load(&mut self) -> Box<Future<Item = Telemetry, Error = Error>> {
        match self.host.get_type() {
            HostType::Local(l) => LocalDebian::load(l),
            HostType::Remote(r) => RemoteDebian::load(r),
        }
    }
}

impl LocalDebian {
    fn available(_: &Local) -> Box<Future<Item = bool, Error = Error>> {
        Box::new(future::ok(cfg!(target_os="linux") && linux::fingerprint_os() == Some(LinuxFlavour::Debian)))
    }

    fn load(_: &Local) -> Box<Future<Item = Telemetry, Error = Error>> {
        Box::new(future::lazy(|| match do_load() {
            Ok(t) => future::ok(t),
            Err(e) => future::err(e),
        }))
    }
}

impl RemoteDebian {
    fn available(host: &Plain) -> Box<Future<Item = bool, Error = Error>> {
        let runnable = Runnable::Telemetry(
                           TelemetryRunnable::Debian(
                               DebianRunnable::Available));
        host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Debian", func: "available" })
    }

    fn load(host: &Plain) -> Box<Future<Item = Telemetry, Error = Error>> {
        let runnable = Runnable::Telemetry(
                           TelemetryRunnable::Debian(
                               DebianRunnable::Load));
        let host = host.clone();

        Box::new(host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Debian", func: "load" })
            .map(|t: serializable::Telemetry| Telemetry::from(t)))
    }
}

impl Executable for DebianRunnable {
    fn exec(self, host: &Local) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            DebianRunnable::Available => Box::new(LocalDebian::available(host).map(|b| Box::new(b) as Box<Serialize>)),
            DebianRunnable::Load => Box::new(LocalDebian::load(host).map(|t| {
                let t: serializable::Telemetry = t.into();
                Box::new(t) as Box<Serialize>
            }))
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
