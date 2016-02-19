// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use {CommandResult, Host, ProviderFactory, Providers, Result};
use error::Error;
use regex::Regex;
use std::{fs, process, str};
use target::bin_resolver::BinResolver;
use telemetry::{FsMount, Netif, NetifIPv4, NetifIPv6, NetifStatus};

pub fn default_provider(host: &mut Host, providers: Vec<Providers>) -> Result<Providers> {
    for p in providers {
        let provider = ProviderFactory::create(host, Some(p));

        if provider.is_ok() {
            return Ok(provider.unwrap().get_providers());
        }
    }

    Err(Error::Generic("No package providers are available".to_string()))
}

pub fn command_exec(cmd: &str) -> Result<CommandResult> {
    let output = try!(process::Command::new("sh").arg("-c").arg(cmd).output());

    Ok(CommandResult {
        exit_code: output.status.code().unwrap(),
        stdout: str::from_utf8(&output.stdout).unwrap().trim().to_string(),
        stderr: str::from_utf8(&output.stderr).unwrap().trim().to_string(),
    })
}

pub fn file_is_file(path: &str) -> Result<bool> {
    let meta = fs::metadata(path);
    Ok(meta.is_err() || meta.unwrap().is_file())
}

pub fn file_exists(path: &str) -> Result<bool> {
    Ok(fs::metadata(path).is_ok())
}

pub fn file_delete(path: &str) -> Result<()> {
    try!(fs::remove_file(path));
    Ok(())
}

pub fn file_mv(path: &str, new_path: &str) -> Result<()> {
    Ok(try!(fs::rename(path, new_path)))
}

pub fn file_copy(path: &str, new_path: &str) -> Result<()> {
    try!(fs::copy(path, new_path));
    Ok(())
}

pub fn file_set_owner(path: &str, user: &str, group: &str) -> Result<()> {
    let user_group = format!("{}:{}", user, group);
    let args: Vec<&str> = vec![&user_group, path];
    let output = process::Command::new(&try!(BinResolver::resolve("chown"))).args(&args).output().unwrap();

    if !output.status.success() {
        return Err(Error::Generic(format!("Could not chown file with error: {}", str::from_utf8(&output.stderr).unwrap())));
    }

    Ok(())
}

pub fn file_stat<'a>(path: &'a str, args: Vec<&'a str>) -> Result<String> {
    let mut args = args;
    args.push(path);
    let output = process::Command::new(&try!(BinResolver::resolve("stat"))).args(&args).output().unwrap();

    if !output.status.success() {
        return Err(Error::Generic(format!("Could not stat file with error: {}", str::from_utf8(&output.stderr).unwrap())));
    }

    Ok(try!(str::from_utf8(&output.stdout)).trim().to_string())
}

pub fn file_set_mode(path: &str, mode: u16) -> Result<()> {
    let mode_s: &str = &mode.to_string();
    let output = process::Command::new(&try!(BinResolver::resolve("chmod"))).args(&vec![mode_s, path]).output().unwrap();

    if !output.status.success() {
        return Err(Error::Generic(format!("Could not chmod file with error: {}", str::from_utf8(&output.stderr).unwrap())));
    }

    Ok(())
}

pub fn hostname() -> Result<String> {
    let output = try!(process::Command::new(&try!(BinResolver::resolve("hostname"))).arg("-f").output());

    if output.status.success() == false {
        return Err(Error::Generic(format!("Could not determine hostname with error: {}", str::from_utf8(&output.stderr).unwrap())));
    }

    Ok(try!(str::from_utf8(&output.stdout)).trim().to_string())
}

pub enum FsFieldOrder {
    Filesystem,
    Size,
    Used,
    Available,
    Capacity,
    Mount,
    Blank,
}

pub fn fs() -> Result<Vec<FsMount>> {
    self::parse_fs(vec![
        self::FsFieldOrder::Filesystem,
        self::FsFieldOrder::Size,
        self::FsFieldOrder::Used,
        self::FsFieldOrder::Available,
        self::FsFieldOrder::Capacity,
        self::FsFieldOrder::Mount,
    ])
}

pub fn parse_fs(fields: Vec<FsFieldOrder>) -> Result<Vec<FsMount>> {
    let mount_out = try!(process::Command::new(&try!(BinResolver::resolve("df"))).arg("-Pk").output());
    let mount = try!(String::from_utf8(mount_out.stdout));

    let mut pattern = "(?m)^".to_string();

    for field in fields {
        match field {
            FsFieldOrder::Filesystem => pattern.push_str("(?P<fs>.+?)"),
            FsFieldOrder::Size => pattern.push_str("(?P<size>[0-9]+)"),
            FsFieldOrder::Used => pattern.push_str("(?P<used>[0-9]+)"),
            FsFieldOrder::Available => pattern.push_str("(?P<available>[0-9]+)"),
            FsFieldOrder::Capacity => pattern.push_str("(?P<capacity>[0-9]{1,3})%"),
            FsFieldOrder::Mount => pattern.push_str("(?P<mount>/.*)"),
            FsFieldOrder::Blank => pattern.push_str(r"[^\s]+"),
        }

        pattern.push_str(r"[\s]*");
    }

    pattern.push_str("$");

    let regex = Regex::new(&pattern).unwrap();
    let mut fs = vec!();

    let lines: Vec<&str> = mount.lines().collect();
    for line in lines {
        if let Some(cap) = regex.captures(line) {
            fs.push(FsMount {
                filesystem: cap.name("fs").unwrap().to_string(),
                mountpoint: cap.name("mount").unwrap().to_string(),
                size: try!(cap.name("size").unwrap().parse::<u64>()),
                used: try!(cap.name("used").unwrap().parse::<u64>()),
                available: try!(cap.name("available").unwrap().parse::<u64>()),
                capacity: try!(cap.name("capacity").unwrap().parse::<f32>())/100.0,
                // inodes_used: try!(cap.name("iused").unwrap().parse::<u64>()),
                // inodes_available: try!(cap.name("iavailable").unwrap().parse::<u64>()),
                // inodes_capacity: try!(cap.name("icapacity").unwrap().parse::<f32>())/100.0,
            });
        }
    };

    Ok(fs)
}

pub fn parse_net(if_pattern: &str, kv_pattern: &str, ipv4_pattern: &str, ipv6_pattern: &str) -> Result<Vec<Netif>> {
    let ifconfig_out = try!(process::Command::new(&try!(BinResolver::resolve("ifconfig"))).output());
    let ifconfig = try!(str::from_utf8(&ifconfig_out.stdout));

    // Rust Regex doesn't support negative lookahead, so we are
    // forced to pass this line by line.
    // Match interfaces:
    // (?m:^([a-z]+[0-9]+):((?:(?!^[a-z]+[0-9]+:).)+))
    // Match options:
    // (?m:^\s*([a-z0-9]+)(?:\s|:\s|=)(.+))
    let if_regex = Regex::new(if_pattern).unwrap();

    let mut net = vec!();

    for cap in if_regex.captures_iter(ifconfig) {
        net.push(try!(parse_netif(cap.name("if").unwrap(), cap.name("content").unwrap(), kv_pattern, ipv4_pattern, ipv6_pattern)));
    }

    Ok(net)
}

fn parse_netif(iface: &str, content: &str, kv_pattern: &str, ipv4_pattern: &str, ipv6_pattern: &str) -> Result<Netif> {
    let mut netif = Netif {
        interface: iface.to_string(),
        mac: None,
        inet: None,
        inet6: None,
        status: None,
    };

    let kv_regex = Regex::new(kv_pattern).unwrap();
    let ipv4_regex = Regex::new(ipv4_pattern).unwrap();
    let ipv6_regex = Regex::new(ipv6_pattern).unwrap();

    let lines: Vec<&str> = content.lines().collect();
    for line in lines {
        if let Some(kv_capture) = kv_regex.captures(line) {
            let value = kv_capture.name("value").unwrap().trim();

            match kv_capture.name("key").unwrap() {
                "ether" | "HWaddr" => netif.mac = Some(value.to_string()),
                "inet" => {
                    if let Some(ipv4_capture) = ipv4_regex.captures(value) {
                        netif.inet = Some(NetifIPv4 {
                            address: ipv4_capture.name("ip").unwrap().to_string(),
                            netmask: ipv4_capture.name("mask").unwrap().to_string(),
                        });
                    }
                },
                "inet6" => {
                    if let Some(ipv6_capture) = ipv6_regex.captures(value) {
                        netif.inet6 = Some(NetifIPv6 {
                            address: ipv6_capture.name("ip").unwrap().to_string(),
                            prefixlen: try!(ipv6_capture.name("prefix").unwrap().parse::<u8>()),
                            scopeid: if ipv6_capture.name("scope").is_some() {
                                Some(ipv6_capture.name("scope").unwrap().to_string())
                            } else {
                                None
                            },
                        });
                    }
                },
                "status" => netif.status = match value {
                    "active" => Some(NetifStatus::Active),
                    "inactive" => Some(NetifStatus::Inactive),
                    _ => unreachable!(),
                },
                _ => (),
            }
        }
    }

    Ok(netif)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hostname() {
        // XXX Not a proper test. Requires mocking.
        assert!(hostname().is_ok());
    }
}
