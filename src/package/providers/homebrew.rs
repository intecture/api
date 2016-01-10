// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Homebrew package provider

use {Command, CommandResult, Host};
use Result;
use super::*;

pub struct Homebrew;

impl Provider for Homebrew {
    fn get_providers(&self) -> Providers {
        Providers::Homebrew
    }

    fn is_active(&self, host: &mut Host) -> Result<bool> {
        let cmd = Command::new("which brew");
        let result = try!(cmd.exec(host));

        Ok(result.exit_code == 0)
    }

    fn is_installed(&self, host: &mut Host, name: &str) -> Result<bool> {
        let cmd = Command::new(&format!("brew list | grep {}", name));
        let result = try!(cmd.exec(host));

        Ok(result.exit_code == 0)
    }

    fn install(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        let cmd = Command::new(&format!("brew install {}", name));
        cmd.exec(host)
    }

    fn uninstall(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        let cmd = Command::new(&format!("brew uninstall {}", name));
        cmd.exec(host)
    }
}
