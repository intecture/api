// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! System generated data about your host.

pub mod providers;
mod serializable;

use erased_serde::Serialize;
use errors::*;
use ExecutableProvider;
use host::Host;
use pnet::datalink::NetworkInterface;
use self::providers::{Centos, CentosRemoteProvider, Debian, DebianRemoteProvider,
                      Freebsd, FreebsdRemoteProvider, Macos, MacosRemoteProvider};

pub trait TelemetryProvider {
    fn available(&Host) -> bool where Self: Sized;
    fn load(&Host) -> Result<Telemetry>;
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum RemoteProvider {
    Centos(CentosRemoteProvider),
    Debian(DebianRemoteProvider),
    Freebsd(FreebsdRemoteProvider),
    Macos(MacosRemoteProvider),
}

impl <'de>ExecutableProvider<'de> for RemoteProvider {
    fn exec(self, host: &Host) -> Result<Box<Serialize>> {
        match self {
            RemoteProvider::Centos(p) => p.exec(host),
            RemoteProvider::Debian(p) => p.exec(host),
            RemoteProvider::Freebsd(p) => p.exec(host),
            RemoteProvider::Macos(p) => p.exec(host)
        }
    }
}

#[derive(Debug)]
pub struct Telemetry {
    pub cpu: Cpu,
    pub fs: Vec<FsMount>,
    pub hostname: String,
    pub memory: u64,
    pub net: Vec<NetworkInterface>,
    pub os: Os,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cpu {
    pub vendor: String,
    pub brand_string: String,
    pub cores: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FsMount {
    pub filesystem: String,
    pub mountpoint: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub capacity: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Os {
    pub arch: String,
    pub family: OsFamily,
    pub platform: OsPlatform,
    pub version_str: String,
    pub version_maj: u32,
    pub version_min: u32,
    pub version_patch: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OsFamily {
    Bsd,
    Darwin,
    Linux,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OsPlatform {
    Centos,
    Debian,
    Fedora,
    Freebsd,
    Macos,
    Nixos,
    Ubuntu,
}

pub fn load(host: &Host) -> Result<Telemetry> {
    if Centos::available(host) {
        Centos::load(host)
    }
    else if Debian::available(host) {
        Debian::load(host)
    }
    else if Freebsd::available(host) {
        Freebsd::load(host)
    }
    else if Macos::available(host) {
        Macos::load(host)
    } else {
        Err(ErrorKind::ProviderUnavailable("Telemetry").into())
    }
}
