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
use regex::Regex;
use std::{env, fs, str};
use std::io::Read;
use target::{default, unix};
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry, TelemetryProvider, serializable};

pub struct Freebsd;

#[derive(Serialize, Deserialize)]
pub enum RemoteProvider {
    Available,
    Load,
}

impl <'de>ExecutableProvider<'de> for RemoteProvider {
    fn exec(self, host: &Host) -> Result<Box<Serialize>> {
        match self {
            RemoteProvider::Available => Ok(Box::new(Freebsd::available(host))),
            RemoteProvider::Load => {
                let t: serializable::Telemetry = Freebsd::load(host)?.into();
                Ok(Box::new(t))
            },
        }
    }
}

impl TelemetryProvider for Freebsd {
    fn available(host: &Host) -> bool {
        if host.is_local() {
            cfg!(target_os="freebsd")
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
            let cpu_vendor = telemetry_cpu_vendor()?;
            let cpu_brand = unix::get_sysctl_item("hw\\.model")?;
            let hostname = default::hostname()?;
            let (version_str, version_maj, version_min) = unix::version()?;

            Ok(Telemetry {
                cpu: Cpu {
                    vendor: cpu_vendor,
                    brand_string: cpu_brand,
                    cores: unix::get_sysctl_item("hw\\.ncpu")
                                .chain_err(|| "could not resolve telemetry data")?
                                .parse::<u32>()
                                .chain_err(|| "could not resolve telemetry data")?,
                },
                fs: default::fs()?,
                hostname: hostname,
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
        } else {
            unimplemented!();
            // let r = RemoteProvider::Load;
            // self.host.send(r).chain_err(|| ErrorKind::RemoteProvider("Telemetry", "load"))?;
            // let t: Telemetry = self.host.recv()?;
            // Ok(t)
        }
    }
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
