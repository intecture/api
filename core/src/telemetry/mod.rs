// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! System generated data about your host.

pub mod providers;
mod serializable;

use pnet::datalink::NetworkInterface;

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
