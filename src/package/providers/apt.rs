// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Apt package provider

use command::{Command, CommandResult};
use error::Result;
use host::Host;
use super::*;

pub struct Apt;

impl Provider for Apt {
    fn get_providers(&self) -> Providers {
        Providers::Apt
    }

    fn is_active(&self, host: &mut Host) -> Result<bool> {
        let cmd = Command::new("type apt-get");
        let result = try!(cmd.exec(host));

        Ok(result.exit_code == 0)
    }

    fn is_installed(&self, host: &mut Host, name: &str) -> Result<bool> {
        let cmd = Command::new(&format!("dpkg --get-selections | grep -E \"{}\\s+install$\"", name));
        let result = try!(cmd.exec(host));

        Ok(result.exit_code == 0)
    }

    fn install(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        let cmd = Command::new(&format!("apt-get -y install {}", name));
        cmd.exec(host)
    }

    fn uninstall(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        let cmd = Command::new(&format!("apt-get -y remove {}", name));
        cmd.exec(host)
    }
}
