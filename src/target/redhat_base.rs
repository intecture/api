// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use {CommandResult, Result};
use error::Error;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use target::bin_resolver::BinResolver;
use target::default_base as default;

pub fn service_init(name: &str, action: &str) -> Result<CommandResult> {
    match action {
        "enable" => default::command_exec(&format!("{} {} on", &try!(BinResolver::resolve("chkconfig")), name)),
        "disable" => default::command_exec(&format!("{} {} off", &try!(BinResolver::resolve("chkconfig")), name)),
        _ => default::service_action(name, action),
    }
}

pub fn version() -> Result<String> {
    let mut fh = try!(File::open("/etc/redhat-release"));
    let mut fc = String::new();
    fh.read_to_string(&mut fc).unwrap();

    let regex = Regex::new(r"release ([0-9.]+)").unwrap();
    if let Some(cap) = regex.captures(&fc) {
        Ok(cap.at(1).unwrap().to_string())
    } else {
        Err(Error::Generic("Could not match OS version".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        // XXX Not a proper test. Requires mocking.
        assert!(version().is_ok());
    }
}
