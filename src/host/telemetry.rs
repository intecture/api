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
#[derive(Debug, RustcEncodable)]
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

    // XXX While Macros 1.1 are unstable, we can't use Serde to
    // convert Telemetry => Value, so we have to roll our own.
    // (https://github.com/rust-lang/rust/issues/35900)
    #[cfg(feature = "local-run")]
    pub fn into_value(self) -> Value {
        let mut cpu: Map<String, Value> = Map::new();
        cpu.insert("vendor".into(), json!(self.cpu.vendor));
        cpu.insert("brand_string".into(), json!(self.cpu.brand_string));
        cpu.insert("cores".into(), json!(self.cpu.cores));

        let mut fs = Vec::new();
        for mount in self.fs {
            let mut map: Map<String, Value> = Map::new();
            map.insert("filesystem".into(), json!(mount.filesystem));
            map.insert("mountpoint".into(), json!(mount.mountpoint));
            map.insert("size".into(), json!(mount.size));
            map.insert("used".into(), json!(mount.used));
            map.insert("available".into(), json!(mount.available));
            map.insert("capacity".into(), json!(mount.capacity));
            fs.push(map);
        }

        let mut net = Vec::new();
        for netif in self.net {
            let mut map: Map<String, Value> = Map::new();
            map.insert("name".into(), json!(netif.name));
            map.insert("index".into(), json!(netif.index));
            if let Some(mac) = netif.mac {
                map.insert("mac".into(), json!(mac));
            }
            if let Some(ips) = netif.ips {
                map.insert("ips".into(), json!(ips));
            }
            map.insert("flags".into(), json!(netif.flags));
            net.push(map);
        }

        let mut os: Map<String, Value> = Map::new();
        os.insert("arch".into(), json!(self.os.arch));
        os.insert("family".into(), json!(self.os.family));
        os.insert("platform".into(), json!(self.os.platform));
        os.insert("version_str".into(), json!(self.os.version_str));
        os.insert("version_maj".into(), json!(self.os.version_maj));
        os.insert("version_min".into(), json!(self.os.version_min));
        os.insert("version_patch".into(), json!(self.os.version_patch));

        let mut telemetry: Map<String, Value> = Map::new();
        telemetry.insert("cpu".into(), json!(cpu));
        telemetry.insert("fs".into(), json!(fs));
        telemetry.insert("hostname".into(), json!(self.hostname));
        telemetry.insert("memory".into(), json!(self.memory));
        telemetry.insert("net".into(), json!(net));
        telemetry.insert("os".into(), json!(os));
        json!(telemetry)
    }
}

pub trait TelemetryTarget {
    fn telemetry_init(host: &mut Host) -> Result<Value>;
}

#[cfg(feature = "local-run")]
#[derive(Debug, RustcEncodable)]
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
#[derive(Debug, RustcEncodable)]
pub struct FsMount {
    pub filesystem: String,
    pub mountpoint: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub capacity: f32,
}

#[cfg(feature = "local-run")]
#[derive(Debug, RustcEncodable)]
pub struct Netif {
    pub name: String,
    pub index: u32,
    pub mac: Option<String>,
    pub ips: Option<Vec<String>>,
    pub flags: u32,
}

#[cfg(feature = "local-run")]
#[derive(Debug, RustcEncodable)]
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
