// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::CommandResult;
use error::{Error, Result};
use file::FileOwner;
use host::telemetry::{Netif, NetifIPv4, NetifIPv6, NetifStatus};
use regex::Regex;
use std::{process, str};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::net::SocketAddr;
use target::default_base as default;
use ipnetwork::ipv6_mask_to_prefix;
use interfaces::Kind::{Ipv4, Ipv6};
use interfaces::{Address, Interface, flags};

pub fn file_get_owner<P: AsRef<Path>>(path: P) -> Result<FileOwner> {
    Ok(FileOwner {
        user_name: try!(default::file_stat(path.as_ref(), vec!["-c", "%U"])),
        user_uid: try!(default::file_stat(path.as_ref(), vec!["-c", "%u"])).parse::<u64>().unwrap(),
        group_name: try!(default::file_stat(path.as_ref(), vec!["-c", "%G"])),
        group_gid: try!(default::file_stat(path.as_ref(), vec!["-c", "%g"])).parse::<u64>().unwrap()
    })
}

pub fn file_get_mode<P: AsRef<Path>>(path: P) -> Result<u16> {
    Ok(try!(default::file_stat(path, vec!["-c", "%a"])).parse::<u16>().unwrap())
}

pub fn using_systemd() -> Result<bool> {
    let output = process::Command::new("stat").args(&["--format=%N", "/proc/1/exe"]).output().unwrap();
    if output.status.success() {
        let out = try!(str::from_utf8(&output.stdout));
        Ok(out.contains("systemd"))
    } else {
        Err(Error::Generic(try!(String::from_utf8(output.stdout))))
    }
}

pub fn service_systemd(name: &str, action: &str) -> Result<Option<CommandResult>> {
    match action {
        "enable" | "disable" => {
            let output = try!(process::Command::new("systemctl").arg("is-enabled").arg(name).output());
            if (action == "enable" && output.status.success()) || (action == "disable" && !output.status.success()) {
                return Ok(None);
            }
        },
        "start" | "stop" => {
            let output = try!(process::Command::new("systemctl").arg("is-active").arg(name).output());
            if (action == "start" && output.status.success()) || (action == "stop" && !output.status.success()) {
                return Ok(None);
            }
        },
        _ => (),
    }

    Ok(Some(try!(default::command_exec(&format!("systemctl {} {}", action, name)))))
}

pub fn memory() -> Result<u64> {
    let output = process::Command::new("free").arg("-b").output().unwrap();

    if !output.status.success() {
        return Err(Error::Generic("Could not determine memory".to_string()));
    }

    let regex = Regex::new(r"(?m)^Mem:\s+([0-9]+)").unwrap();
    let capture = regex.captures(try!(str::from_utf8(&output.stdout)).trim());

    if capture.is_some() {
        Ok(capture.unwrap().get(1).unwrap().as_str().parse::<u64>().unwrap())
    } else {
        Err(Error::Generic("Invalid memory output".to_string()))
    }
}

pub fn cpu_vendor() -> Result<String> {
    get_cpu_item("vendor_id")
}

pub fn cpu_brand_string() -> Result<String> {
    get_cpu_item("model name")
}

pub fn cpu_cores() -> Result<u32> {
    Ok(try!(try!(get_cpu_item("cpu cores")).parse::<u32>()))
}

fn get_cpu_item(item: &str) -> Result<String> {
    // XXX This result should be cached
    let mut cpuinfo_f = try!(File::open("/proc/cpuinfo"));
    let mut cpuinfo = String::new();
    try!(cpuinfo_f.read_to_string(&mut cpuinfo));

    let pattern = format!(r"(?m)^{}\s+: (.+)$", item);
    let regex = Regex::new(&pattern).unwrap();
    let capture = regex.captures(&cpuinfo);

    if capture.is_some() {
        Ok(capture.unwrap().get(1).unwrap().as_str().to_string())
    } else {
        Err(Error::Generic(format!("Could not find CPU item: {}", item)))
    }
}

fn ipv4(a: &Address) -> Option<NetifIPv4> {
    match a.addr {
        Some(addr) => {
            let address = addr.ip().to_string();
            let netmask = a.mask
                .and_then(|mask| Some(mask.ip().to_string()))
                .unwrap_or(String::new());

            Some(NetifIPv4 {
                address: address,
                netmask: netmask,
            })
        },
        None => None,
    }
}

fn ipv6(a: &Address) -> Option<NetifIPv6> {
    if let Some(SocketAddr::V6(address)) = a.addr {
        if let Some(SocketAddr::V6(mask)) = a.mask {
            if let Ok(prefixlen) = ipv6_mask_to_prefix(*mask.ip()) {
                return Some(NetifIPv6 {
                    address: address.ip().to_string(),
                    prefixlen: prefixlen,
                    scopeid: Some(address.scope_id().to_string()),
                });
            }
        }
    }
    None
}

pub fn net() -> Result<Vec<Netif>> {
    let mut ifaces = Vec::new();

    for iface in Interface::get_all()? {
        let mac = match iface.hardware_addr() {
            Ok(addr) => Some(addr.as_string()),
            Err(_) => None,
        };

        let inet = iface.addresses.iter()
            .filter(|addr| addr.kind == Ipv4)
            .next()
            .and_then(|addr| ipv4(addr));

        let inet6 = iface.addresses.iter()
            .filter(|addr| addr.kind == Ipv6)
            .next()
            .and_then(|addr| ipv6(addr));

        let up = iface.is_up();
        let running = iface.flags.contains(flags::IFF_RUNNING);

        let status = if up && running {
            Some(NetifStatus::Active)
        } else {
            Some(NetifStatus::Inactive)
        };

        ifaces.push(Netif {
            interface: iface.name.to_string(),
            mac: mac,
            inet: inet,
            inet6: inet6,
            status: status,
        });
    }

    Ok(ifaces)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::get_cpu_item;

    #[test]
    fn test_memory() {
        // XXX Not a proper test. Requires mocking.
        assert!(memory().is_ok());
    }

    #[test]
    fn test_cpu_vendor() {
        // XXX Not a proper test. Requires mocking.
        assert!(cpu_vendor().is_ok());
    }

    #[test]
    fn test_cpu_brand_string() {
        // XXX Not a proper test. Requires mocking.
        assert!(cpu_brand_string().is_ok());
    }

    #[test]
    fn test_cpu_cores() {
        // XXX Not a proper test. Requires mocking.
        assert!(cpu_cores().is_ok());
    }

    #[test]
    fn test_get_cpu_item_fail() {
        assert!(get_cpu_item("moocow").is_err());
    }

    #[test]
    fn test_net() {
        // XXX Not a proper test. Requires mocking.
        assert!(net().is_ok());
    }
}
