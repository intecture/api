// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Data structures containing information about your managed host.
//!
//! The Telemetry struct stores metadata about a host, such as its
//! network interfaces, disk mounts, CPU stats and hostname.
//!
//! # Examples
//!
//! Initialise a new Host:
//!
//! ```no_run
//! # use inapi::Host;
#![cfg_attr(feature = "local-run", doc = "let mut host = Host::local(None);")]
#![cfg_attr(feature = "remote-run", doc = "let mut host = Host::connect(\"data/nodes/mynode.json\").unwrap();")]
//! ```
//!
//! Now run your command and get the result:
//!
//! ```no_run
//! # use inapi::{Host, Telemetry};
//! # let mut host = Host::local(None);
//! let telemetry = Telemetry::init(&mut host);
//! ```

use error::Result;
use host::Host;
use serde_json::Value;
#[cfg(feature = "local-run")]
use serde_json::{to_value, Map};
use target::Target;

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

    /// Initialise a new Telemetry struct for the given Host.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::{Host, Telemetry};
    /// # let mut host = Host::local(None);
    /// let telemetry = Telemetry::init(&mut host);
    /// ```
    pub fn init(host: &mut Host) -> Result<Value> {
        Target::telemetry_init(host)
    }

    // XXX While Macros 1.1 are unstable, we can't use Serde to
    // handle Telemetry => Value, so we have to roll our own.
    // (https://github.com/rust-lang/rust/issues/35900)
    #[cfg(feature = "local-run")]
    pub fn into_value(self) -> Value {
        let mut cpu: Map<String, Value> = Map::new();
        cpu.insert("vendor".into(), to_value(self.cpu.vendor));
        cpu.insert("brand_string".into(), to_value(self.cpu.brand_string));
        cpu.insert("cores".into(), to_value(self.cpu.cores));

        let mut fs = Vec::new();
        for mount in self.fs {
            let mut map: Map<String, Value> = Map::new();
            map.insert("filesystem".into(), to_value(mount.filesystem));
            map.insert("mountpoint".into(), to_value(mount.mountpoint));
            map.insert("size".into(), to_value(mount.size));
            map.insert("used".into(), to_value(mount.used));
            map.insert("available".into(), to_value(mount.available));
            map.insert("capacity".into(), to_value(mount.capacity));
            fs.push(map);
        }

        let mut net = Vec::new();
        for netif in self.net {
            let mut map: Map<String, Value> = Map::new();
            map.insert("interface".into(), to_value(netif.interface));
            map.insert("mac".into(), to_value(netif.mac));
            if let Some(inet) = netif.inet {
                let mut map1: Map<String, Value> = Map::new();
                map1.insert("address".into(), to_value(inet.address));
                map1.insert("netmask".into(), to_value(inet.netmask));
                map.insert("inet".into(), to_value(map1));
            }
            if let Some(inet6) = netif.inet6 {
                let mut map1: Map<String, Value> = Map::new();
                map1.insert("address".into(), to_value(inet6.address));
                map1.insert("prefixlen".into(), to_value(inet6.prefixlen));
                map1.insert("scopeid".into(), to_value(inet6.scopeid));
                map.insert("inet6".into(), to_value(map1));
            }
            if let Some(status) = netif.status {
                map.insert("status".into(), to_value(if status == NetifStatus::Active { "Active".to_string() } else { "Inactive".to_string() }));
            }
            net.push(map);
        }

        let mut os: Map<String, Value> = Map::new();
        os.insert("arch".into(), to_value(self.os.arch));
        os.insert("family".into(), to_value(self.os.family));
        os.insert("platform".into(), to_value(self.os.platform));
        os.insert("version".into(), to_value(self.os.version));

        let mut telemetry: Map<String, Value> = Map::new();
        telemetry.insert("cpu".into(), to_value(cpu));
        telemetry.insert("fs".into(), to_value(fs));
        telemetry.insert("hostname".into(), to_value(self.hostname));
        telemetry.insert("memory".into(), to_value(self.memory));
        telemetry.insert("net".into(), to_value(net));
        telemetry.insert("os".into(), to_value(os));
        to_value(telemetry)
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
impl FsMount {
    pub fn new(filesystem: &str, mountpoint: &str, size: u64, used: u64, available: u64, capacity: f32) -> FsMount {
        FsMount {
            filesystem: filesystem.to_string(),
            mountpoint: mountpoint.to_string(),
            size: size,
            used: used,
            available: available,
            capacity: capacity,
        }
    }
}

#[cfg(feature = "local-run")]
#[derive(Debug, RustcEncodable)]
pub struct Netif {
    pub interface: String,
    pub mac: Option<String>,
    pub inet: Option<NetifIPv4>,
    pub inet6: Option<NetifIPv6>,
    pub status: Option<NetifStatus>,
}

#[cfg(feature = "local-run")]
impl Netif {
    pub fn new(interface: &str, mac: Option<&str>, inet: Option<NetifIPv4>, inet6: Option<NetifIPv6>, status: Option<NetifStatus>) -> Netif {
        Netif {
            interface: interface.to_string(),
            mac: if mac.is_some() {
                Some(mac.unwrap().to_string())
            } else {
                None
            },
            inet: inet,
            inet6: inet6,
            status: status,
        }
    }
}

#[cfg(feature = "local-run")]
#[derive(Debug, RustcEncodable, PartialEq)]
pub enum NetifStatus {
    Active,
    Inactive,
}

#[cfg(feature = "local-run")]
#[derive(Debug, RustcEncodable)]
pub struct NetifIPv4 {
    pub address: String,
    pub netmask: String,
}

#[cfg(feature = "local-run")]
impl NetifIPv4 {
    pub fn new(address: &str, netmask: &str) -> NetifIPv4 {
        NetifIPv4 {
            address: address.to_string(),
            netmask: netmask.to_string(),
        }
    }
}

#[cfg(feature = "local-run")]
#[derive(Debug, RustcEncodable)]
pub struct NetifIPv6 {
    pub address: String,
    pub prefixlen: u8,
    pub scopeid: Option<String>,
}

#[cfg(feature = "local-run")]
impl NetifIPv6 {
    pub fn new(address: &str, prefixlen: u8, scopeid: Option<&str>) -> NetifIPv6 {
        NetifIPv6 {
            address: address.to_string(),
            prefixlen: prefixlen,
            scopeid: if scopeid.is_some() {
                Some(scopeid.unwrap().to_string())
            } else {
                None
            }
        }
    }
}

#[cfg(feature = "local-run")]
#[derive(Debug, RustcEncodable)]
pub struct Os {
    pub arch: String,
    pub family: String,
    pub platform: String,
    pub version: String,
}

#[cfg(feature = "local-run")]
impl Os {
    pub fn new(arch: &str, family: &str, platform: &str, version: &str) -> Os {
        Os {
            arch: arch.to_string(),
            family: family.to_string(),
            platform: platform.to_string(),
            version: version.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "local-run")]
    use Host;
    #[cfg(feature = "local-run")]
    use super::*;

    #[cfg(feature = "local-run")]
    #[test]
    fn test_telemetry_init() {
        let mut host = Host::local(None);
        assert!(Telemetry::init(&mut host).is_ok());
    }
}
