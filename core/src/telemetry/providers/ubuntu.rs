// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use futures::future;
use pnet::datalink::interfaces;
use provider::Provider;
use regex::Regex;
use remote::{ExecutableResult, ProviderName, Response, ResponseResult};
use std::{env, process, str};
use super::TelemetryProvider;
use target::{default, linux};
use target::linux::LinuxFlavour;
use telemetry::{Cpu, LinuxDistro, Os, OsFamily, OsPlatform, Telemetry};
use tokio_proto::streaming::Message;

pub struct Ubuntu;

impl Provider for Ubuntu {
    fn available() -> Result<bool> {
        Ok(cfg!(target_os="linux") && linux::fingerprint_os() == Some(LinuxFlavour::Ubuntu))
    }

    fn name(&self) -> ProviderName {
        ProviderName::TelemetryUbuntu
    }
}

impl TelemetryProvider for Ubuntu {
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
            family: OsFamily::Linux(LinuxDistro::Debian),
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
                   .chain_err(|| ErrorKind::SystemCommand("Ubuntu-version"))?;

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
