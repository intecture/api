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
use target::{default, unix};
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry, serializable};

pub struct Macos<H: Host> {
    host: H,
}

struct LocalMacos;
struct RemoteMacos;

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum MacosRunnable {
    Available,
    Load,
}

impl<H: Host + 'static> Provider<H> for Macos<H> {
    fn available(host: &H) -> Box<Future<Item = bool, Error = Error>> {
        match host.get_type() {
            HostType::Local(l) => LocalMacos::available(l),
            HostType::Remote(r) => RemoteMacos::available(r),
        }
    }

    fn try_new(host: &H) -> Box<Future<Item = Option<Macos<H>>, Error = Error>> {
        let host = host.clone();
        Box::new(Self::available(&host)
            .and_then(|available| {
                if available {
                    future::ok(Some(Macos { host }))
                } else {
                    future::ok(None)
                }
            }))
    }
}

impl<H: Host + 'static> TelemetryProvider<H> for Macos<H> {
    fn load(&mut self) -> Box<Future<Item = Telemetry, Error = Error>> {
        match self.host.get_type() {
            HostType::Local(l) => LocalMacos::load(l),
            HostType::Remote(r) => RemoteMacos::load(r),
        }
    }
}

impl LocalMacos {
    fn available(_: &Local) -> Box<Future<Item = bool, Error = Error>> {
        Box::new(future::ok(cfg!(target_os="macos")))
    }

    fn load(_: &Local) -> Box<Future<Item = Telemetry, Error = Error>> {
        Box::new(future::lazy(|| match do_load() {
            Ok(t) => future::ok(t),
            Err(e) => future::err(e),
        }))
    }
}

impl RemoteMacos {
    fn available(host: &Plain) -> Box<Future<Item = bool, Error = Error>> {
        let runnable = Runnable::Telemetry(
                           TelemetryRunnable::Macos(
                               MacosRunnable::Available));
        host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Macos", func: "available" })
    }

    fn load(host: &Plain) -> Box<Future<Item = Telemetry, Error = Error>> {
        let runnable = Runnable::Telemetry(
                           TelemetryRunnable::Macos(
                               MacosRunnable::Load));
        let host = host.clone();

        Box::new(host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Macos", func: "load" })
            .map(|t: serializable::Telemetry| Telemetry::from(t)))
    }
}

impl Executable for MacosRunnable {
    fn exec(self, host: &Local) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            MacosRunnable::Available => Box::new(LocalMacos::available(host).map(|b| Box::new(b) as Box<Serialize>)),
            MacosRunnable::Load => Box::new(LocalMacos::load(host).map(|t| {
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
            vendor: unix::get_sysctl_item("machdep\\.cpu\\.vendor")?,
            brand_string: unix::get_sysctl_item("machdep\\.cpu\\.brand_string")?,
            cores: unix::get_sysctl_item("hw\\.physicalcpu")
                        .chain_err(|| "could not resolve telemetry data")?
                        .parse::<u32>()
                        .chain_err(|| "could not resolve telemetry data")?
        },
        fs: default::parse_fs(&[
            default::FsFieldOrder::Filesystem,
            default::FsFieldOrder::Size,
            default::FsFieldOrder::Used,
            default::FsFieldOrder::Available,
            default::FsFieldOrder::Capacity,
            default::FsFieldOrder::Blank,
            default::FsFieldOrder::Blank,
            default::FsFieldOrder::Blank,
            default::FsFieldOrder::Mount,
        ])?,
        hostname: default::hostname()?,
        memory: unix::get_sysctl_item("hw\\.memsize")
                     .chain_err(|| "could not resolve telemetry data")?
                     .parse::<u64>()
                     .chain_err(|| "could not resolve telemetry data")?,
        net: interfaces(),
        os: Os {
            arch: env::consts::ARCH.into(),
            family: OsFamily::Darwin,
            platform: OsPlatform::Macos,
            version_str: version_str,
            version_maj: version_maj,
            version_min: version_min,
            version_patch: version_patch
        },
    })
}

fn version() -> Result<(String, u32, u32, u32)> {
    let out = process::Command::new("sw_vers")
                               .arg("-productVersion")
                               .output()
                               .chain_err(|| ErrorKind::SystemCommand("sw_vers"))?;
    let version_str = str::from_utf8(&out.stdout)
                          .chain_err(|| ErrorKind::SystemCommandOutput("sw_vers"))?
                          .trim()
                          .to_owned();
    let (maj, min, patch) = {
        let mut parts = version_str.split('.');
        let errstr = format!("Expected OS version format `u32.u32[.u32]`, got: '{}'", version_str);
        (
            parts.next().ok_or(&*errstr)?.parse().chain_err(|| ErrorKind::SystemCommandOutput("sw_vers"))?,
            parts.next().ok_or(&*errstr)?.parse().chain_err(|| ErrorKind::SystemCommandOutput("sw_vers"))?,
            parts.next().unwrap_or("0").parse().chain_err(|| ErrorKind::SystemCommandOutput("sw_vers"))?
        )
    };
    Ok((version_str, maj, min, patch))
}
