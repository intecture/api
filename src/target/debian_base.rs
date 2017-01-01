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
use std::fs::read_dir;
use std::process::Command;
use std::str;
use target::bin_resolver::BinResolver;
use target::default_base as default;

pub fn service_init(name: &str, action: &str) -> Result<Option<CommandResult>> {
    if action == "enable" || action == "disable" {
        let output = try!(Command::new(try!(BinResolver::resolve("runlevel"))).output());
        if !output.status.success() {
            return Err(Error::Generic("Could not get runlevel".into()));
        }

        let regex = try!(Regex::new(r"^[A-Z] ([0-9])\s?$"));
        let runlevel = match regex.captures(str::from_utf8(&output.stdout).unwrap_or("")) {
            Some(caps) => caps.at(1).unwrap_or(""),
            None => "",
        };

        if runlevel == "" {
            return Err(Error::Generic("Could not interpret runlevel".into()));
        }

        let regex = try!(Regex::new(&format!("/S[0-9]{{2}}{}$", name)));
        let mut enabled = false;
        for file in try!(read_dir(&format!("/etc/rc{}.d", runlevel))) {
            if regex.is_match(try!(file).path().to_str().unwrap_or("")) {
                enabled = true;
                break;
            }
        }

        // XXX `update-rc.d` enable/disable is marked as unstable
        let update_rcd = BinResolver::resolve("update-rc.d")?;
        match action {
            "enable" if !enabled => Ok(Some(default::command_exec(&format!("{} {} enable", update_rcd.to_str().unwrap(), name))?)),
            "disable" if enabled => Ok(Some(default::command_exec(&format!("{} {} disable", update_rcd.to_str().unwrap(), name))?)),
            _ => Ok(None)
        }
    } else {
        default::service_action(name, action)
    }
}
