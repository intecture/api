// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Pkg package provider

use {Command, CommandResult, Host};
use Result;
use super::*;

pub struct Pkg;

impl Provider for Pkg {
    fn get_providers(&self) -> Providers {
        Providers::Pkg
    }

    fn is_active(&self, host: &mut Host) -> Result<bool> {
        let cmd = Command::new("which pkg");
        let result = try!(cmd.exec(host));

        Ok(result.exit_code == 0)
    }

    fn is_installed(&self, host: &mut Host, name: &str) -> Result<bool> {
        let cmd = Command::new(&format!("pkg query \"%n\" {}", name));
        let result = try!(cmd.exec(host));

        Ok(result.exit_code == 0)
    }

    fn install(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        let cmd = Command::new(&format!("env ASSUME_ALWAYS_YES=YES pkg install {}", name));
        cmd.exec(host)
    }

    fn uninstall(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        let cmd = Command::new(&format!("env ASSUME_ALWAYS_YES=YES pkg delete {}", name));
        cmd.exec(host)
    }
}
