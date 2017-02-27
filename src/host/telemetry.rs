// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use error::Result;
use host::Host;
#[cfg(feature = "local-run")]
use serde_json::Map;
use serde_json::Value;
use target::Target;
#[cfg(feature = "local-run")]

#[cfg(feature = "local-run")]
#[derive(Debug, Serialize)]
pub struct Telemetry {
    pub cpu: Cpu,
    pub fs: Vec<FsMount>,
    pub hostname: String,
    pub memory: u64,
    pub net: Vec<Netif>,
    pub os: Os,
}

#[cfg(feature = "remote-run")]
pub struct Telemetry;

impl Telemetry {
    #[cfg(feature = "local-run")]
    pub fn new(cpu: Cpu, fs: Vec<FsMount>, hostname: &str, memory: u64, net: Vec<Netif>, os: Os) -> Telemetry {
        Telemetry {
            cpu: cpu,
            fs: fs,
            hostname: hostname.to_string(),
            memory: memory,
            net: net,
            os: os,
        }
    }

    #[cfg(feature = "local-run")]
    pub fn init(host: &mut Host) -> Result<Value> {
        let t = try!(Target::telemetry_init(host));

        // Make sure telemetry is namespaced
        let mut t_map: Map<String, Value> = Map::new();
        t_map.insert("_telemetry".into(), t);
        Ok(json!(t_map))
    }

    #[cfg(feature = "remote-run")]
    pub fn init(host: &mut Host) -> Result<Value> {
        Ok(try!(Target::telemetry_init(host)))
    }
}

pub trait TelemetryTarget {
    fn telemetry_init(host: &mut Host) -> Result<Value>;
}

#[cfg(feature = "local-run")]
#[derive(Debug, Serialize)]
pub struct Cpu {
    pub vendor: String,
    pub brand_string: String,
    pub cores: u32,
}

#[cfg(feature = "local-run")]
impl Cpu {
    pub fn new(vendor: &str, brand_string: &str, cores: u32) -> Cpu {
        Cpu {
            vendor: vendor.to_string(),
            brand_string: brand_string.to_string(),
            cores: cores,
        }
    }
}

#[cfg(feature = "local-run")]
#[derive(Debug, Serialize)]
pub struct FsMount {
    pub filesystem: String,
    pub mountpoint: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub capacity: f32,
}

#[cfg(feature = "local-run")]
#[derive(Debug, Serialize)]
pub struct Netif {
    pub name: String,
    pub index: u32,
    pub mac: Option<String>,
    pub ips: Option<Vec<String>>,
    pub flags: u32,
}

#[cfg(feature = "local-run")]
#[derive(Debug, Serialize)]
pub struct Os {
    pub arch: String,
    pub family: String,
    pub platform: String,
    pub version_str: String,
    pub version_maj: u32,
    pub version_min: u32,
    pub version_patch: u32,
}

#[cfg(feature = "local-run")]
impl Os {
    pub fn new(arch: &str, family: &str, platform: &str, version_str: &str, version_maj: u32, version_min: u32, version_patch: u32) -> Os {
        Os {
            arch: arch.into(),
            family: family.into(),
            platform: platform.into(),
            version_str: version_str.into(),
            version_maj: version_maj,
            version_min: version_min,
            version_patch: version_patch,
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "local-run")]
    use Host;

    #[cfg(feature = "local-run")]
    #[test]
    fn test_telemetry_init() {
        let path: Option<String> = None;
        assert!(Host::local(path).is_ok());
    }
}
