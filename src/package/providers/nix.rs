// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Nix package provider

use command::{Command, CommandResult};
use error::Result;
use host::Host;
use super::*;

pub struct Nix;

impl Provider for Nix {
    fn get_providers(&self) -> Providers {
        Providers::Nix
    }

    fn is_active(&self, host: &mut Host) -> Result<bool> {
        let cmd = Command::new("which nix-env");
        let result = try!(cmd.exec(host));

        Ok(result.exit_code == 0)
    }

    fn is_installed(&self, host: &mut Host, name: &str) -> Result<bool> {
        let cmd = Command::new(&format!("nix-env --install --dry-run {}", name));
        let result = try!(cmd.exec(host));

        if result.exit_code != 0 {
            return Err(Error::Agent(result.stderr));
        }

        Ok(!result.stderr.contains("these paths will be fetched"))
    }

    fn install(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        let cmd = Command::new(&format!("nix-env --install {}", name));
        cmd.exec(host)
    }

    fn uninstall(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        let cmd = Command::new(&format!("nix-env --uninstall {}", name));
        cmd.exec(host)
    }
}
