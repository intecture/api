// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use erased_serde::Serialize;
use errors::*;
use {Executable, Runnable};
use futures::future::{self, Future, FutureResult};
use host::*;
use pnet::datalink::interfaces;
use regex::Regex;
use std::{env, fs, str};
use std::io::Read;
use std::sync::Arc;
use target::{default, unix};
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry,
                TelemetryProvider, TelemetryRunnable, serializable};

pub struct Freebsd;

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum FreebsdRunnable {
    Available,
    Load,
}

impl <H: Host + 'static>TelemetryProvider<H> for Freebsd {
    fn available(host: &Arc<H>) -> Box<Future<Item = bool, Error = Error>> {
        host.run(Runnable::Telemetry(TelemetryRunnable::Freebsd(FreebsdRunnable::Available)))
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Freebsd", func: "available" })
    }

    fn try_load(host: &Arc<H>) -> Box<Future<Item = Option<Telemetry>, Error = Error>> {
        let host = host.clone();

        Box::new(Self::available(&host)
            .and_then(move |available| {
                if available {
                    Box::new(host.run::<serializable::Telemetry>(Runnable::Telemetry(TelemetryRunnable::Freebsd(FreebsdRunnable::Load)))
                        .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Freebsd", func: "load" })
                        .map(|t| {
                            let t: Telemetry = t.into();
                            Some(t)
                        })) as Box<Future<Item = Option<Telemetry>, Error = Error>>
                } else {
                    Box::new(future::ok(None)) as Box<Future<Item = Option<Telemetry>, Error = Error>>
                }
          }))
    }
}

impl Executable for FreebsdRunnable {
    fn exec(self) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            FreebsdRunnable::Available =>
                Box::new(future::ok(Box::new(
                    cfg!(target_os="freebsd")
                ) as Box<Serialize>)),
            FreebsdRunnable::Load => {
                Box::new(future::lazy(move || -> FutureResult<Box<Serialize>, Error> {
                    match do_load() {
                        Ok(t) => {
                            let t: serializable::Telemetry = t.into();
                            future::ok(Box::new(t) as Box<Serialize>)
                        },
                        Err(e) => future::err(e),
                    }
                }))
            }
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
