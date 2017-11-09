// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use futures::future;
use pnet::datalink::interfaces;
use remote::{ExecutableResult, Response, ResponseResult};
use std::{env, process, str};
use super::TelemetryProvider;
use target::{default, linux};
use target::linux::LinuxFlavour;
use telemetry::{Cpu, LinuxDistro, Os, OsFamily, OsPlatform, Telemetry};
use tokio_proto::streaming::Message;

pub struct Nixos;

impl TelemetryProvider for Nixos {
    fn available() -> bool {
        cfg!(target_os="linux") && linux::fingerprint_os() == Some(LinuxFlavour::Nixos)
    }

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
            family: OsFamily::Linux(LinuxDistro::Standalone),
            platform: OsPlatform::Nixos,
            version_str: version_str,
            version_maj: version_maj,
            version_min: version_min,
            version_patch: version_patch
        },
        user: default::user()?,
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
