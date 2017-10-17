// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use erased_serde::Serialize;
use errors::*;
use futures::{future, Future};
use host::{Host, HostType};
use host::local::Local;
use host::remote::Plain;
use pnet::datalink::interfaces;
use provider::Provider;
use remote::{Executable, Runnable};
use std::{env, process, str};
use super::{TelemetryProvider, TelemetryRunnable};
use target::{default, linux};
use target::linux::LinuxFlavour;
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry, serializable};

pub struct Nixos;
struct LocalNixos;
struct RemoteNixos;

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum NixosRunnable {
    Available,
    Load,
}

impl<H: Host + 'static> Provider<H> for Nixos {
    fn available(host: &H) -> Box<Future<Item = bool, Error = Error>> {
        match host.get_type() {
            HostType::Local(_) => LocalNixos::available(),
            HostType::Remote(r) => RemoteNixos::available(r),
        }
    }

    fn try_new(host: &H) -> Box<Future<Item = Option<Nixos>, Error = Error>> {
        let host = host.clone();
        Box::new(Self::available(&host)
            .and_then(|available| {
                if available {
                    future::ok(Some(Nixos))
                } else {
                    future::ok(None)
                }
            }))
    }
}

impl<H: Host + 'static> TelemetryProvider<H> for Nixos {
    fn load(&self, host: &H) -> Box<Future<Item = Telemetry, Error = Error>> {
        match host.get_type() {
            HostType::Local(_) => LocalNixos::load(),
            HostType::Remote(r) => RemoteNixos::load(r),
        }
    }
}

impl LocalNixos {
    fn available() -> Box<Future<Item = bool, Error = Error>> {
        Box::new(future::ok(cfg!(target_os="linux") && linux::fingerprint_os() == Some(LinuxFlavour::Nixos)))
    }

    fn load() -> Box<Future<Item = Telemetry, Error = Error>> {
        Box::new(future::lazy(|| match do_load() {
            Ok(t) => future::ok(t),
            Err(e) => future::err(e),
        }))
    }
}

impl RemoteNixos {
    fn available(host: &Plain) -> Box<Future<Item = bool, Error = Error>> {
        let runnable = Runnable::Telemetry(
                           TelemetryRunnable::Nixos(
                               NixosRunnable::Available));
        host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Nixos", func: "available" })
    }

    fn load(host: &Plain) -> Box<Future<Item = Telemetry, Error = Error>> {
        let runnable = Runnable::Telemetry(
                           TelemetryRunnable::Nixos(
                               NixosRunnable::Load));
        let host = host.clone();

        Box::new(host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Nixos", func: "load" })
            .map(|t: serializable::Telemetry| Telemetry::from(t)))
    }
}

impl Executable for NixosRunnable {
    fn exec(self, _: &Local) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            NixosRunnable::Available => Box::new(LocalNixos::available().map(|b| Box::new(b) as Box<Serialize>)),
            NixosRunnable::Load => Box::new(LocalNixos::load().map(|t| {
                let t: serializable::Telemetry = t.into();
                Box::new(t) as Box<Serialize>
            }))
        }
    }
}

fn do_load() -> Result<Telemetry> {
    let (version_str, version_maj, version_min, version_patch) = version()?;

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
            platform: OsPlatform::Nixos,
            version_str: version_str,
            version_maj: version_maj,
            version_min: version_min,
            version_patch: version_patch
        },
    })
}

fn version() -> Result<(String, u32, u32, u32)> {
    let out = process::Command::new("nixos-version")
                               .output()
                               .chain_err(|| ErrorKind::SystemCommand("nixos-version"))?;
    let version_str = str::from_utf8(&out.stdout)
                          .chain_err(|| ErrorKind::SystemCommandOutput("nixos-version"))?
                          .trim()
                          .to_owned();
    let (maj, min, patch) = {
        let mut parts = version_str.split('.');
        let errstr = format!("Expected OS version format `u32.u32.u32.hash (codename)`, got: '{}'", version_str);
        (
            parts.next().ok_or(&*errstr)?.parse().chain_err(|| ErrorKind::SystemCommandOutput("nixos-version"))?,
            parts.next().ok_or(&*errstr)?.parse().chain_err(|| ErrorKind::SystemCommandOutput("nixos-version"))?,
            parts.next().unwrap_or("0").parse().chain_err(|| ErrorKind::SystemCommandOutput("nixos-version"))?
        )
    };
    Ok((version_str, maj, min, patch))
}
