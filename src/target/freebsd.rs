// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use regex::Regex;
use std::{env, fs};
use std::io::Read;
use super::{Result, Target, TargetError, TargetInterface};
use target::{default_base as default, unix_base as unix};
use telemetry::{FsMount, Netif};

impl TargetInterface for Target {
    fn hostname() -> Result<String> {
        default::hostname()
    }

    fn arch() -> String {
        env::consts::ARCH.to_string()
    }

    fn family() -> String {
        "unix".to_string()
    }

    fn platform() -> String {
        "freebsd".to_string()
    }

    fn version() -> Result<String> {
        unix::version()
    }

    fn memory() -> Result<u64> {
        Ok(try!(unix::get_sysctl_item("hw\\.physmem")).parse::<u64>().unwrap())
    }

    fn cpu_vendor() -> Result<String> {
        let mut fh = try!(fs::File::open("/var/run/dmesg.boot"));
        let mut fc = String::new();
        fh.read_to_string(&mut fc).unwrap();

        let regex = Regex::new(r#"(?m)^CPU:.+$\n\s+Origin="([A-Za-z]+)""#).unwrap();
        if let Some(cap) = regex.captures(&fc) {
            Ok(cap.at(1).unwrap().to_string())
        } else {
            Err(TargetError::Generic("Could not match CPU vendor".to_string()))
        }
    }

    fn cpu_brand_string() -> Result<String> {
        unix::get_sysctl_item("hw\\.model")
    }

    fn cpu_cores() -> Result<u32> {
        Ok(try!(unix::get_sysctl_item("hw\\.ncpu")).parse::<u32>().unwrap())
    }

    fn fs() -> Result<Vec<FsMount>> {
        default::fs()
    }

    fn net() -> Result<Vec<Netif>> {
        unix::net()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_version() {
        // XXX Not a proper test. Requires mocking.
        assert!(Target::version().is_ok());
    }

    #[test]
    fn test_fs() {
        // XXX Not a proper test. Requires mocking.
        assert!(Target::fs().is_ok());
    }
}