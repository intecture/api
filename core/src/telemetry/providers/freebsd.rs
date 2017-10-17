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
use regex::Regex;
use remote::{Executable, Runnable};
use std::{env, fs, str};
use std::io::Read;
use super::{TelemetryProvider, TelemetryRunnable};
use target::{default, unix};
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry, serializable};

pub struct Freebsd;
struct LocalFreebsd;
struct RemoteFreebsd;

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum FreebsdRunnable {
    Available,
    Load,
}

impl<H: Host + 'static> Provider<H> for Freebsd {
    fn available(host: &H) -> Box<Future<Item = bool, Error = Error>> {
        match host.get_type() {
            HostType::Local(_) => LocalFreebsd::available(),
            HostType::Remote(r) => RemoteFreebsd::available(r),
        }
    }

    fn try_new(host: &H) -> Box<Future<Item = Option<Freebsd>, Error = Error>> {
        let host = host.clone();
        Box::new(Self::available(&host)
            .and_then(|available| {
                if available {
                    future::ok(Some(Freebsd))
                } else {
                    future::ok(None)
                }
            }))
    }
}

impl<H: Host + 'static> TelemetryProvider<H> for Freebsd {
    fn load(&self, host: &H) -> Box<Future<Item = Telemetry, Error = Error>> {
        match host.get_type() {
            HostType::Local(_) => LocalFreebsd::load(),
            HostType::Remote(r) => RemoteFreebsd::load(r),
        }
    }
}

impl LocalFreebsd {
    fn available() -> Box<Future<Item = bool, Error = Error>> {
        Box::new(future::ok(cfg!(target_os="freebsd")))
    }

    fn load() -> Box<Future<Item = Telemetry, Error = Error>> {
        Box::new(future::lazy(|| match do_load() {
            Ok(t) => future::ok(t),
            Err(e) => future::err(e),
        }))
    }
}

impl RemoteFreebsd {
    fn available(host: &Plain) -> Box<Future<Item = bool, Error = Error>> {
        let runnable = Runnable::Telemetry(
                           TelemetryRunnable::Freebsd(
                               FreebsdRunnable::Available));
        host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Freebsd", func: "available" })
    }

    fn load(host: &Plain) -> Box<Future<Item = Telemetry, Error = Error>> {
        let runnable = Runnable::Telemetry(
                           TelemetryRunnable::Freebsd(
                               FreebsdRunnable::Load));
        let host = host.clone();

        Box::new(host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Freebsd", func: "load" })
            .map(|t: serializable::Telemetry| Telemetry::from(t)))
    }
}

impl Executable for FreebsdRunnable {
    fn exec(self, _: &Local) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            FreebsdRunnable::Available => Box::new(LocalFreebsd::available().map(|b| Box::new(b) as Box<Serialize>)),
            FreebsdRunnable::Load => Box::new(LocalFreebsd::load().map(|t| {
                let t: serializable::Telemetry = t.into();
                Box::new(t) as Box<Serialize>
            }))
        }
    }
}

fn do_load() -> Result<Telemetry> {
    let (version_str, version_maj, version_min) = unix::version()?;

    Ok(Telemetry {
        cpu: Cpu {
            vendor: telemetry_cpu_vendor()?,
            brand_string: unix::get_sysctl_item("hw\\.model")?,
            cores: unix::get_sysctl_item("hw\\.ncpu")
                        .chain_err(|| "could not resolve telemetry data")?
                        .parse::<u32>()
                        .chain_err(|| "could not resolve telemetry data")?,
        },
        fs: default::fs()?,
        hostname: default::hostname()?,
        memory: unix::get_sysctl_item("hw\\.physmem")
                     .chain_err(|| "could not resolve telemetry data")?
                     .parse::<u64>()
                     .chain_err(|| "could not resolve telemetry data")?,
        net: interfaces(),
        os: Os {
            arch: env::consts::ARCH.into(),
            family: OsFamily::Bsd,
            platform: OsPlatform::Freebsd,
            version_str: version_str,
            version_maj: version_maj,
            version_min: version_min,
            version_patch: 0
        },
    })
}

fn telemetry_cpu_vendor() -> Result<String> {
    let mut fh = fs::File::open("/var/run/dmesg.boot")
                          .chain_err(|| ErrorKind::SystemFile("/var/run/dmesg.boot"))?;
    let mut fc = String::new();
    fh.read_to_string(&mut fc).chain_err(|| ErrorKind::SystemFileOutput("/var/run/dmesg.boot"))?;

    let regex = Regex::new(r#"(?m)^CPU:.+$\n\s+Origin="([A-Za-z]+)""#).unwrap();
    if let Some(cap) = regex.captures(&fc) {
        Ok(cap.get(1).unwrap().as_str().into())
    } else {
        Err(ErrorKind::SystemFileOutput("/var/run/dmesg.boot").into())
    }
}
