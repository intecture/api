// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

// @todo If Rust ever relaxes its orphan rules, we'll be able to
// implement Serialize/Deserialize on 3rd party structures.
// See: https://github.com/rust-lang/rfcs/issues/1856

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use pnet::datalink::NetworkInterface;
use pnet::util::MacAddr;
use std::convert::From;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

#[derive(Serialize, Deserialize)]
pub struct Telemetry {
    pub cpu: super::Cpu,
    pub fs: Vec<super::FsMount>,
    pub hostname: String,
    pub memory: u64,
    pub net: Vec<Netif>,
    pub os: super::Os,
}

#[derive(Serialize, Deserialize)]
pub struct Netif {
    pub name: String,
    pub index: u32,
    pub mac: Option<String>,
    pub ips: Vec<IpNet>,
    pub flags: u32,
}

#[derive(Serialize, Deserialize)]
pub enum IpNet {
    V4(Ipv4Net),
    V6(Ipv6Net),
}

#[derive(Serialize, Deserialize)]
pub struct Ipv4Net {
    ip: Ipv4Addr,
    prefix: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Ipv6Net {
    ip: Ipv6Addr,
    prefix: u8,
}

impl From<super::Telemetry> for Telemetry {
    fn from(t: super::Telemetry) -> Telemetry {
        let net = t.net.into_iter().map(|iface| Netif {
            name: iface.name,
            index: iface.index,
            mac: iface.mac.map(|addr| addr.to_string()),
            ips: iface.ips.into_iter().map(|net| match net {
                IpNetwork::V4(n) => IpNet::V4(Ipv4Net {
                    ip: n.ip(),
                    prefix: n.prefix(),
                }),
                IpNetwork::V6(n) => IpNet::V6(Ipv6Net {
                    ip: n.ip(),
                    prefix: n.prefix(),
                })
            }).collect(),
            flags: iface.flags,
        }).collect();

        Telemetry {
            cpu: t.cpu,
            fs: t.fs,
            hostname: t.hostname,
            memory: t.memory,
            net: net,
            os: t.os,
        }
    }
}

impl From<Telemetry> for super::Telemetry {
    fn from(t: Telemetry) -> super::Telemetry {
        let net = t.net.into_iter().map(|iface| NetworkInterface {
            name: iface.name,
            index: iface.index,
            mac: iface.mac.map(|addr| MacAddr::from_str(&addr).unwrap()),
            ips: iface.ips.into_iter().map(|net| match net {
                IpNet::V4(n) => IpNetwork::V4(Ipv4Network::new(n.ip, n.prefix).unwrap()),
                IpNet::V6(n) => IpNetwork::V6(Ipv6Network::new(n.ip, n.prefix).unwrap()),
            }).collect(),
            flags: iface.flags,
        }).collect();

        super::Telemetry {
            cpu: t.cpu,
            fs: t.fs,
            hostname: t.hostname,
            memory: t.memory,
            net: net,
            os: t.os,
        }
    }
}
