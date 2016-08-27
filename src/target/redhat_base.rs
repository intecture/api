// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::CommandResult;
use error::{Error, Result};
use regex::Regex;
use std::fs::File;
use std::io::Read;
use target::bin_resolver::BinResolver;
use target::default_base as default;

pub fn service_init(name: &str, action: &str) -> Result<Option<CommandResult>> {
    if action == "enable" || action == "disable" {
        let result = try!(default::command_exec(&format!("{} {}", &try!(BinResolver::resolve("chkconfig")), name)));

        match action {
            "enable" if result.exit_code != 0 => Ok(Some(try!(default::command_exec(&format!("{} {} on", &try!(BinResolver::resolve("chkconfig")), name))))),
            "disable" if result.exit_code == 0 => Ok(Some(try!(default::command_exec(&format!("{} {} off", &try!(BinResolver::resolve("chkconfig")), name))))),
            _ => Ok(None)
        }
    } else {
        default::service_action(name, action)
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
