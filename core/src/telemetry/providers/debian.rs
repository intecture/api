// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use erased_serde::Serialize;
use errors::*;
use ExecutableProvider;
use host::*;
use pnet::datalink::interfaces;
use std::{env, process, str};
use target::{default, linux};
use target::linux::LinuxFlavour;
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry, TelemetryProvider, serializable};

pub struct Debian;

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum RemoteProvider {
    Available,
    Load,
}

impl <'de>ExecutableProvider<'de> for RemoteProvider {
    fn exec(self, host: &Host) -> Result<Box<Serialize>> {
        match self {
            RemoteProvider::Available => Ok(Box::new(Debian::available(host))),
            RemoteProvider::Load => {
                let t: serializable::Telemetry = Debian::load(host)?.into();
                Ok(Box::new(t))
            },
        }
    }
}

impl TelemetryProvider for Debian {
    fn available(host: &Host) -> bool {
        if host.is_local() {
            cfg!(target_os="linux") && linux::fingerprint_os() == Some(LinuxFlavour::Debian)
        } else {
            unimplemented!();
            // let r = RemoteProvider::Available;
            // self.host.send(r).chain_err(|| ErrorKind::RemoteProvider("Telemetry", "available"))?;
            // let t: Telemetry = self.host.recv()?;
            // Ok(t)
        }
    }

    fn load(host: &Host) -> Result<Telemetry> {
        if host.is_local() {
            let cpu_vendor = linux::cpu_vendor()?;
            let cpu_brand = linux::cpu_brand_string()?;
            let (version_str, version_maj, version_min) = version()?;

            Ok(Telemetry {
                cpu: Cpu {
                    vendor: cpu_vendor,
                    brand_string: cpu_brand,
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
                    version_patch: 0
                },
            })
        } else {
            unimplemented!();
            // let r = RemoteProvider::Load;
            // self.host.send(r).chain_err(|| ErrorKind::RemoteProvider("Telemetry", "load"))?;
            // let t: Telemetry = self.host.recv()?;
            // Ok(t)
        }
    }
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
