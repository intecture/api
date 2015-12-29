// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Yum package provider

use {Command, CommandResult, Host};
use Result;
use super::*;

pub struct Yum;

impl Provider for Yum {
    fn is_active(&self, host: &mut Host) -> Result<bool> {
        let cmd = Command::new("which yum");
        let result = try!(cmd.exec(host));

        Ok(result.exit_code == 0)
    }

    fn is_installed(&self, host: &mut Host, name: &str) -> Result<bool> {
        let cmd = Command::new(&format!("yum list installed | grep {}", name));
        let result = try!(cmd.exec(host));

        Ok(result.exit_code == 0)
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
