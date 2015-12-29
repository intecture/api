// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Pkg package provider

use {/*Command, */CommandResult, Host};
use Result;
use super::*;

pub struct Pkg;

impl Provider for Pkg {
    #[allow(unused_variables)]
    fn is_active(&self, host: &mut Host) -> Result<bool> {
        unimplemented!();
    }

    #[allow(unused_variables)]
    fn is_installed(&self, host: &mut Host, name: &str) -> Result<bool> {
        unimplemented!();
    }

    #[allow(unused_variables)]
    fn install(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        unimplemented!();
    }

    #[allow(unused_variables)]
    fn uninstall(&self, host: &mut Host, name: &str) -> Result<CommandResult> {
        unimplemented!();
    }
}
