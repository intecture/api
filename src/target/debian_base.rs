// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use {CommandResult, Error, Result};
use regex::Regex;
use std::fs::read_dir;
use std::process::Command;
use target::bin_resolver::BinResolver;
use target::default_base as default;

pub fn service_init(name: &str, action: &str) -> Result<Option<CommandResult>> {
    if action == "enable" || action == "disable" {
        let output = try!(Command::new(try!(BinResolver::resolve("runlevel"))).output());
        if !output.status.success() {
            return Err(Error::Generic("Could not get runlevel".into()));
        }
        let runlevel = match output.stdout.last().unwrap() {
            &61 => 1,
            &62 => 2,
            &63 => 3,
            &64 => 4,
            &65 => 5,
            _ => unreachable!(),
        };

        let regex = try!(Regex::new(&format!("/S[0-9]{{2}}{}$", name)));
        let mut enabled = false;
        for file in try!(read_dir(&format!("/etc/rc{}.d", runlevel))) {
            if regex.is_match(try!(file).path().to_str().unwrap_or("")) {
                enabled = true;
                break;
            }
        }

        // XXX `update-rc.d` enable/disable is marked as unstable
        match action {
            "enable" if !enabled => Ok(Some(try!(default::command_exec(&format!("{} {} enable", try!(BinResolver::resolve("update-rc.d")), name))))),
            "disable" if enabled => Ok(Some(try!(default::command_exec(&format!("{} {} disable", try!(BinResolver::resolve("update-rc.d")), name))))),
            _ => Ok(None)
        }
    } else {
        default::service_action(name, action)
    }
}
