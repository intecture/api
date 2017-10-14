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
use std::{env, process, str};
use std::sync::Arc;
use target::{default, linux};
use target::linux::LinuxFlavour;
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry,
                TelemetryProvider, TelemetryRunnable, serializable};

pub struct Ubuntu;

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum UbuntuRunnable {
    Available,
    Load,
}

impl <H: Host + 'static>TelemetryProvider<H> for Ubuntu {
    fn available(host: &Arc<H>) -> Box<Future<Item = bool, Error = Error>> {
        host.run(Runnable::Telemetry(TelemetryRunnable::Ubuntu(UbuntuRunnable::Available)))
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Ubuntu", func: "available" })
    }

    fn try_load(host: &Arc<H>) -> Box<Future<Item = Option<Telemetry>, Error = Error>> {
        let host = host.clone();

        Box::new(Self::available(&host)
            .and_then(move |available| {
                if available {
                    Box::new(host.run::<serializable::Telemetry>(Runnable::Telemetry(TelemetryRunnable::Ubuntu(UbuntuRunnable::Load)))
                        .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Ubuntu", func: "load" })
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

impl Executable for UbuntuRunnable {
    fn exec(self) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            UbuntuRunnable::Available =>
                Box::new(future::ok(Box::new(
                    cfg!(target_os="linux") && linux::fingerprint_os() == Some(LinuxFlavour::Ubuntu)
                ) as Box<Serialize>)),
            UbuntuRunnable::Load => {
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
            platform: OsPlatform::Ubuntu,
            version_str: version_str,
            version_maj: version_maj,
            version_min: version_min,
            version_patch: version_patch,
        },
    })
}

fn version() -> Result<(String, u32, u32, u32)> {
    let out = process::Command::new("lsb_release").arg("-sd").output()?;
    let desc = str::from_utf8(&out.stdout)
                   .chain_err(|| ErrorKind::SystemCommand("nixos-version"))?;

    let regex = Regex::new(r"([0-9]+)\.([0-9]+)\.([0-9]+)( LTS)?").unwrap();
    if let Some(cap) = regex.captures(&desc) {
        let version_maj = cap.get(1).unwrap().as_str().parse().chain_err(|| ErrorKind::SystemCommandOutput("lsb_release -sd"))?;
        let version_min = cap.get(2).unwrap().as_str().parse().chain_err(|| ErrorKind::SystemCommandOutput("lsb_release -sd"))?;
        let version_patch = cap.get(3).unwrap().as_str().parse().chain_err(|| ErrorKind::SystemCommandOutput("lsb_release -sd"))?;
        let mut version_str = format!("{}.{}.{}", version_maj, version_min, version_patch);
        if cap.get(4).is_some() {
            version_str.push_str(" LTS");
        }
        Ok((version_str, version_maj, version_min, version_patch))
    } else {
        Err(ErrorKind::SystemCommandOutput("lsb_release -sd").into())
    }
}
