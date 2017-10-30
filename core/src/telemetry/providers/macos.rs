// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use futures::future;
use pnet::datalink::interfaces;
use provider::Provider;
use remote::{ExecutableResult, ProviderName, Response, ResponseResult};
use std::{env, process, str};
use super::TelemetryProvider;
use target::{default, unix};
use telemetry::{Cpu, Os, OsFamily, OsPlatform, Telemetry};
use tokio_proto::streaming::Message;

pub struct Macos;

impl Provider for Macos {
    fn available() -> bool {
        cfg!(target_os="macos")
    }

    fn name(&self) -> ProviderName {
        ProviderName::TelemetryMacos
    }
}

impl TelemetryProvider for Macos {
    fn load(&self) -> ExecutableResult {
        Box::new(future::lazy(|| {
            let t = match do_load() {
                Ok(t) => t,
                Err(e) => return future::err(e),
            };

            future::ok(Message::WithoutBody(
                ResponseResult::Ok(
                    Response::TelemetryLoad(t.into()))))
        }))
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
