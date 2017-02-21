// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
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
        let chkconfig = BinResolver::resolve("chkconfig")?;
        let result = default::command_exec(&format!("{} {}", chkconfig.to_str().unwrap(), name))?;

        match action {
            "enable" if result.exit_code != 0 => Ok(Some(try!(default::command_exec(&format!("{} {} on", chkconfig.to_str().unwrap(), name))))),
            "disable" if result.exit_code == 0 => Ok(Some(try!(default::command_exec(&format!("{} {} off", chkconfig.to_str().unwrap(), name))))),
            _ => Ok(None)
        }
    } else {
        default::service_action(name, action)
    }
}

pub fn version() -> Result<(String, u32, u32, u32)> {
    let mut fh = try!(File::open("/etc/redhat-release"));
    let mut fc = String::new();
    fh.read_to_string(&mut fc).unwrap();

    let regex = Regex::new(r"release ([0-9]+)(?:\.([0-9]+)(?:\.([0-9]+))?)?").unwrap();
    if let Some(cap) = regex.captures(&fc) {
        let version_maj = cap.get(1).unwrap().as_str().parse()?;
        let version_min = match cap.get(2) {
            Some(v) => v.as_str().parse()?,
            None => 0,
        };
        let version_patch = match cap.get(3) {
            Some(v) => v.as_str().parse()?,
            None => 0,
        };
        let version_str = format!("{}.{}.{}", version_maj, version_min, version_patch);
        Ok((version_str, version_maj, version_min, version_patch))
    } else {
        Err(Error::Generic("Could not match OS version".into()))
    }
}
