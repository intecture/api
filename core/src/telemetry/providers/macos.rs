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
use target::{default, unix};
use telemetry::{Cpu, Os, Telemetry, TelemetryProvider, serializable};

pub struct Macos;

#[derive(Serialize, Deserialize)]
pub enum RemoteProvider {
    Available,
    Load,
}

impl <'de>ExecutableProvider<'de> for RemoteProvider {
    fn exec(&self, host: &Host) -> Result<Box<Serialize>> {
        match *self {
            RemoteProvider::Available => Ok(Box::new(Macos::available(host))),
            RemoteProvider::Load => {
                let t: serializable::Telemetry = Macos::load(host)?.into();
                Ok(Box::new(t))
            },
        }
    }
}

impl TelemetryProvider for Macos {
    fn available(host: &Host) -> bool {
        if host.is_local() {
            cfg!(target_os="macos")
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
            let cpu_vendor = try!(unix::get_sysctl_item("machdep\\.cpu\\.vendor"));
            let cpu_brand = try!(unix::get_sysctl_item("machdep\\.cpu\\.brand_string"));
            let hostname = try!(default::hostname());
            let (version_str, version_maj, version_min, version_patch) = try!(version());

            Ok(Telemetry {
                cpu: Cpu {
                    vendor: cpu_vendor,
                    brand_string: cpu_brand,
                    cores: unix::get_sysctl_item("hw\\.physicalcpu")
                                .chain_err(|| "could not resolve telemetry data")?
                                .parse::<u32>()
                                .chain_err(|| "could not resolve telemetry data")?
                },
                fs: try!(default::parse_fs(&[
                    default::FsFieldOrder::Filesystem,
                    default::FsFieldOrder::Size,
                    default::FsFieldOrder::Used,
                    default::FsFieldOrder::Available,
                    default::FsFieldOrder::Capacity,
                    default::FsFieldOrder::Blank,
                    default::FsFieldOrder::Blank,
                    default::FsFieldOrder::Blank,
                    default::FsFieldOrder::Mount,
                ])),
                hostname: hostname,
                memory: unix::get_sysctl_item("hw\\.memsize")
                             .chain_err(|| "could not resolve telemetry data")?
                             .parse::<u64>()
                             .chain_err(|| "could not resolve telemetry data")?,
                net: interfaces(),
                os: Os {
                    arch: env::consts::ARCH.into(),
                    family: "darwin".into(),
                    platform: "macos".into(),
                    version_str: version_str,
                    version_maj: version_maj,
                    version_min: version_min,
                    version_patch: version_patch
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
