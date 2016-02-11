// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use error::Error;
use regex::Regex;
use Result;
use std::{process, str};
use super::default_base as default;
use telemetry::Netif;

pub fn file_get_mode(path: &str) -> Result<u16> {
    default::file_get_mode(path, vec!["-f", "%Lp"])
}

pub fn version() -> Result<String> {
    let output = try!(process::Command::new("uname").arg("-r").output());
    Ok(str::from_utf8(&output.stdout).unwrap().trim().to_string())
}

pub fn net() -> Result<Vec<Netif>> {
    let if_pattern = r"(?m)^(?P<if>[a-z0-9]+): flags=(?P<content>.+\n(?:^\s+.+\n)+)";
    let kv_pattern = r"^\s+(?P<key>[a-z0-9]+)(?:\s|:\s|=)(?P<value>.+)";
    let ipv4_pattern = r"^(?P<ip>(?:[0-9]{1,3}\.){3}[0-9]{1,3}) netmask (?P<mask>0x[a-f0-9]{8})";
    let ipv6_pattern = r"^(?P<ip>(?:[a-f0-9]{4}::(?:[a-f0-9]{1,4}:){3}[a-f0-9]{1,4})|::1)(?:%[a-z]+[0-9]+)? prefixlen (?P<prefix>[0-9]+)(?: scopeid (?P<scope>[a-z0-9]+))?\s*$";
    default::parse_net(if_pattern, kv_pattern, ipv4_pattern, ipv6_pattern)
}

pub fn get_sysctl_item(item: &str) -> Result<String> {
    // XXX This result should be cached
    let sysctl_out = try!(process::Command::new("sysctl").arg("-a").output());
    let sysctl = String::from_utf8(sysctl_out.stdout).unwrap();

    let exp = format!("{}: (.+)", item);
    let regex = Regex::new(&exp).unwrap();

    if let Some(cap) = regex.captures(&sysctl) {
        Ok(cap.at(1).unwrap().to_string())
    } else {
        Err(Error::Generic("Could not match sysctl item".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_net() {
        // XXX Not a proper test. Requires mocking.
        assert!(net().is_ok());
    }

    #[test]
    fn test_get_sysctl_item_err() {
        assert!(super::get_sysctl_item("moo").is_err());
    }
}
