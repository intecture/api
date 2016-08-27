// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Yum package provider

use command::{Command, CommandResult};
use error::{Error, Result};
use host::Host;
use regex::Regex;
use super::*;
use telemetry::Telemetry;

pub struct Yum;

impl Provider for Yum {
    fn get_providers(&self) -> Providers {
        Providers::Yum
    }

    fn is_active(&self, host: &mut Host) -> Result<bool> {
        let cmd = Command::new("which yum");
        let result = try!(cmd.exec(host));

        Ok(result.exit_code == 0)
    }

    fn is_installed(&self, host: &mut Host, name: &str) -> Result<bool> {
        let cmd = Command::new("yum list installed");
        let result = try!(cmd.exec(host));
        if result.exit_code != 0 {
            return Err(Error::Agent(result.stderr));
        }

        let telemetry = try!(Telemetry::init(host));

        let re = try!(Regex::new(&format!("(?m)^{}\\.({}|noarch)\\s+", name, telemetry.os.arch)));
        Ok(re.is_match(&result.stdout))
    }

    fn install(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        let cmd = Command::new(&format!("yum -y install {}", name));
        cmd.exec(host)
    }

    fn uninstall(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        let cmd = Command::new(&format!("yum -y remove {}", name));
        cmd.exec(host)
    }
}
