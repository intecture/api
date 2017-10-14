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
use std::{env, process, str};
use std::sync::Arc;
use target::{default, unix};
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry,
                TelemetryProvider, TelemetryRunnable, serializable};

pub struct Macos;

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum MacosRunnable {
    Available,
    Load,
}

impl <H: Host + 'static>TelemetryProvider<H> for Macos {
    fn available(host: &Arc<H>) -> Box<Future<Item = bool, Error = Error>> {
        host.run(Runnable::Telemetry(TelemetryRunnable::Macos(MacosRunnable::Available)))
            .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Macos", func: "available" })
    }

    fn try_load(host: &Arc<H>) -> Box<Future<Item = Option<Telemetry>, Error = Error>> {
        let host = host.clone();

        Box::new(Self::available(&host)
            .and_then(move |available| {
                if available {
                    Box::new(host.run::<serializable::Telemetry>(Runnable::Telemetry(TelemetryRunnable::Macos(MacosRunnable::Load)))
                        .chain_err(|| ErrorKind::Runnable { endpoint: "Telemetry::Macos", func: "load" })
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

impl Executable for MacosRunnable {
    fn exec(self) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            MacosRunnable::Available =>
                Box::new(future::ok(Box::new(
                    cfg!(target_os="macos")
                ) as Box<Serialize>)),
            MacosRunnable::Load => {
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
