// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
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
//! let mut host = Host::new();
#![cfg_attr(feature = "remote-run", doc = "host.connect(\"127.0.0.1\", 7101, 7102, 7103).unwrap();")]
//! ```
//!
//! Now run your command and get the result:
//!
//! ```no_run
//! # use inapi::{Host, Telemetry};
//! # let mut host = Host::new();
//! let telemetry = Telemetry::init(&mut host);
//! ```

pub mod ffi;

use Host;
use Result;
use target::Target;

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Telemetry {
    pub cpu: Cpu,
    pub fs: Vec<FsMount>,
    pub hostname: String,
    pub memory: u64,
    pub net: Vec<Netif>,
    pub os: Os,
}

impl Telemetry {
    #[doc(hidden)]
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
    /// # let mut host = Host::new();
    /// let telemetry = Telemetry::init(&mut host);
    /// ```
    pub fn init(host: &mut Host) -> Result<Telemetry> {
        Target::telemetry_init(host)
    }
}

pub trait TelemetryTarget {
    fn telemetry_init(host: &mut Host) -> Result<Telemetry>;
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Cpu {
    pub vendor: String,
    pub brand_string: String,
    pub cores: u32,
}

impl Cpu {
    #[doc(hidden)]
    pub fn new(vendor: &str, brand_string: &str, cores: u32) -> Cpu {
        Cpu {
            vendor: vendor.to_string(),
            brand_string: brand_string.to_string(),
            cores: cores,
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct FsMount {
    pub filesystem: String,
    pub mountpoint: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub capacity: f32,
//    pub inodes_used: u64,
//    pub inodes_available: u64,
//    pub inodes_capacity: f32,
}

impl FsMount {
    #[doc(hidden)]
    pub fn new(filesystem: &str, mountpoint: &str, size: u64, used: u64, available: u64, capacity: f32/*, inodes_used: u64, inodes_available: u64, inodes_capacity: f32*/) -> FsMount {
        FsMount {
            filesystem: filesystem.to_string(),
            mountpoint: mountpoint.to_string(),
            size: size,
            used: used,
            available: available,
            capacity: capacity,
            // inodes_used: inodes_used,
            // inodes_available: inodes_available,
            // inodes_capacity: inodes_capacity,
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Netif {
    pub interface: String,
    pub mac: Option<String>,
    pub inet: Option<NetifIPv4>,
    pub inet6: Option<NetifIPv6>,
    pub status: Option<NetifStatus>,
}

impl Netif {
    #[doc(hidden)]
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

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub enum NetifStatus {
    Active,
    Inactive,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct NetifIPv4 {
    pub address: String,
    pub netmask: String,
}

impl NetifIPv4 {
    #[doc(hidden)]
    pub fn new(address: &str, netmask: &str) -> NetifIPv4 {
        NetifIPv4 {
            address: address.to_string(),
            netmask: netmask.to_string(),
        }
    }
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct NetifIPv6 {
    pub address: String,
    pub prefixlen: u8,
    pub scopeid: Option<String>,
}

impl NetifIPv6 {
    #[doc(hidden)]
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

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Os {
    pub arch: String,
    pub family: String,
    pub platform: String,
    pub version: String,
}

impl Os {
    #[doc(hidden)]
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
    use Host;
    #[cfg(feature = "remote-run")]
    use rustc_serialize::json;
    #[cfg(feature = "remote-run")]
    use std::thread;
    use super::*;
    #[cfg(feature = "remote-run")]
    use zmq;

    #[cfg(feature = "local-run")]
    #[test]
    fn test_telemetry_init() {
        let mut host = Host::new();
        assert!(Telemetry::init(&mut host).is_ok());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_telemetry_init() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_init").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("telemetry", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            let telemetry = Telemetry {
                cpu: Cpu {
                    vendor: "moo".to_string(),
                    brand_string: "Moo Cow Super Fun Happy CPU".to_string(),
                    cores: 100,
                },
                fs: vec![FsMount {
                    filesystem: "/dev/disk0".to_string(),
                    mountpoint: "/".to_string(),
                    size: 10000,
                    used: 5000,
                    available: 5000,
                    capacity: 0.5,
//                    inodes_used: 20,
//                    inodes_available: 0,
//                    inodes_capacity: 1.0,
                }],
                hostname: "localhost".to_string(),
                memory: 2048,
                net: vec![Netif {
                    interface: "em0".to_string(),
                    mac: Some("01:23:45:67:89:ab".to_string()),
                    inet: Some(NetifIPv4 {
                        address: "127.0.0.1".to_string(),
                        netmask: "255.255.255.255".to_string(),
                    }),
                    inet6: Some(NetifIPv6 {
                        address: "::1".to_string(),
                        prefixlen: 8,
                        scopeid: Some("0x4".to_string()),
                    }),
                    status: Some(NetifStatus::Active),
                }],
                os: Os {
                    arch: "doctor string".to_string(),
                    family: "moo".to_string(),
                    platform: "cow".to_string(),
                    version: "1.0".to_string(),
                },
            };

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str(&json::encode(&telemetry).unwrap(), 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_init").unwrap();

        let mut host = Host::test_new(sock);

        let telemetry = Telemetry::init(&mut host).unwrap();

        assert_eq!(telemetry.memory, 2048);
        assert_eq!(telemetry.os.arch, "doctor string".to_string());

        agent_mock.join().unwrap();
    }
}
