// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Telemetry primitive.

mod providers;

pub use self::providers::Macos;

use errors::*;
use ExecutableProvider;
use host::Host;
use self::providers::MacosRemoteProvider;

pub trait TelemetryProvider<'a> {
    fn available(&Host) -> bool where Self: Sized;
    fn try_new(&'a Host) -> Option<Self> where Self: Sized;
    fn load(&self) -> Result<Telemetry>;
}

#[derive(Serialize, Deserialize)]
pub enum RemoteProvider {
    Macos(MacosRemoteProvider)
}

impl <'de>ExecutableProvider<'de> for RemoteProvider {
    fn exec(&self, host: &Host) -> Result<()> {
        match *self {
            RemoteProvider::Macos(ref p) => p.exec(host)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Telemetry {
    pub cpu: Cpu,
    pub fs: Vec<FsMount>,
    pub hostname: String,
    pub memory: u64,
    pub net: Vec<Netif>,
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
pub struct Netif {
    pub name: String,
    pub index: u32,
    pub mac: Option<String>,
    pub ips: Option<Vec<String>>,
    pub flags: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Os {
    pub arch: String,
    pub family: String,
    pub platform: String,
    pub version_str: String,
    pub version_maj: u32,
    pub version_min: u32,
    pub version_patch: u32,
}

pub fn factory<'a>(host: &'a Host) -> Result<Box<TelemetryProvider + 'a>> {
    if let Some(p) = Macos::try_new(host) {
        Ok(Box::new(p))
    } else {
        Err(ErrorKind::ProviderUnavailable("Telemetry").into())
    }
}
