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
use target::bin_resolver::BinResolver;
use target::default_base as default;

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
    let output = process::Command::new(&try!(BinResolver::resolve("stat"))).args(&["--format=%N", "/proc/1/exe"]).output().unwrap();
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
            let output = try!(process::Command::new(&try!(BinResolver::resolve("systemctl"))).arg("is-enabled").arg(name).output());
            if (action == "enable" && output.status.success()) || (action == "disable" && !output.status.success()) {
                return Ok(None);
            }
        },
        "start" | "stop" => {
            let output = try!(process::Command::new(&try!(BinResolver::resolve("systemctl"))).arg("is-active").arg(name).output());
            if (action == "start" && output.status.success()) || (action == "stop" && !output.status.success()) {
                return Ok(None);
            }
        },
        _ => (),
    }

    let systemctl = BinResolver::resolve("systemctl")?;
    Ok(Some(try!(default::command_exec(&format!("{} {} {}", systemctl.to_str().unwrap(), action, name)))))
}

pub fn memory() -> Result<u64> {
    let output = process::Command::new(&try!(BinResolver::resolve("free"))).arg("-b").output().unwrap();

    if !output.status.success() {
        return Err(Error::Generic("Could not determine memory".to_string()));
    }

    let regex = Regex::new(r"(?m)^Mem:\s+([0-9]+)").unwrap();
    let capture = regex.captures(try!(str::from_utf8(&output.stdout)).trim());

    if capture.is_some() {
        Ok(capture.unwrap().at(1).unwrap().parse::<u64>().unwrap())
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
        Ok(capture.unwrap().at(1).unwrap().to_string())
    } else {
        Err(Error::Generic(format!("Could not find CPU item: {}", item)))
    }
}

pub fn net() -> Result<Vec<Netif>> {
    match BinResolver::resolve("ip") {
        Ok(ip_path) => {
            let ip_out = process::Command::new(&ip_path).arg("addr").output()?;
            let ip = str::from_utf8(&ip_out.stdout)?;

            let if_regex = Regex::new(r"^[0-9]+:\s+([a-z0-9]+):.+?state\s([A-Z]+)")?;
            let kv_regex = Regex::new(r"^\s+([a-z0-9/]+)\s(.+)")?;

            let mut net = vec!();

            for line in ip.lines() {
                if let Some(cap) = if_regex.captures(line) {
                    net.push(Netif {
                        interface: cap.at(1).unwrap().into(),
                        mac: None,
                        inet: None,
                        inet6: None,
                        status: match cap.at(2).unwrap() {
                            "UP" => Some(NetifStatus::Active),
                            "DOWN" => Some(NetifStatus::Inactive),
                            _ => None,
                        },
                    });
                }
                else if let Some(this) = net.last_mut() {
                    if let Some(cap) = kv_regex.captures(line) {
                        let mut words: Vec<&str> = cap.at(2).unwrap().split_whitespace().collect();

                        match cap.at(1).unwrap() {
                            "link/ether" if !words.is_empty() => this.mac = Some(words.remove(0).into()),
                            "inet" if words.len() >= 3 => {
                                let mut ip_mask: Vec<&str> = words.remove(0).split('/').collect();

                                // Convert CIDR mask to subnet mask
                                let cidr = ip_mask.remove(1).parse::<u8>()?;
                                let mask: i64 = if cidr > 0 { 0x00 - (1 << (32 - cidr)) } else { 0xFFFFFFFF };
                                let subnet_mask = format!("{}.{}.{}.{}",
                                    mask >> 24 & 0xff,
                                    mask >> 16 & 0xff,
                                    mask >> 8 & 0xff,
                                    mask & 0xff
                                );

                                this.inet = Some(NetifIPv4 {
                                    address: ip_mask.remove(0).into(),
                                    netmask: subnet_mask,
                                });
                            },
                            "inet6" if words.len() >= 3 => {
                                let mut ip_prefix: Vec<&str> = words.remove(0).split('/').collect();
                                this.inet6 = Some(NetifIPv6 {
                                    address: ip_prefix.remove(0).into(),
                                    prefixlen: ip_prefix.remove(0).parse()?,
                                    scopeid: Some(words.remove(1).into()),
                                });
                            }
                            _ => (),
                        }
                    }
                }
            }

            Ok(net)
        },
        Err(_) => {
            let if_pattern = r"(?m)^(?P<if>[a-z0-9]+)\s+Link encap:(Ethernet|Local Loopback)(?P<content>(?s).+?)\n\n";
            let kv_pattern = r"^\s+(?P<key>[A-Za-z0-9]+)(?:\s|:)(?P<value>.+)";
            let ipv4_pattern = r"^addr:(?P<ip>(?:[0-9]{1,3}\.){3}[0-9]{1,3})\s+(?:Bcast:[0-9.]{7,15}\s+)?Mask:(?P<mask>(?:[0-9]{1,3}\.){3}[0-9]{1,3})";
            let ipv6_pattern = r"^addr:\s*(?P<ip>(?:[a-f0-9]{4}::(?:[a-f0-9]{1,4}:){3}[a-f0-9]{1,4})|(?:::1))/(?P<prefix>[0-9]+)\s+Scope:(?P<scope>[A-Za-z0-9]+)";
            default::parse_nettools_net(if_pattern, kv_pattern, ipv4_pattern, ipv6_pattern)
        }
    }
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
