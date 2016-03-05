// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use {CommandResult, Result};
use target::bin_resolver::BinResolver;
use target::default_base as default;

pub fn service_init(name: &str, action: &str) -> Result<CommandResult> {
    match action {
        "enable" | "disable" => default::command_exec(&format!("{} {} {}", &try!(BinResolver::resolve("update-rc.d")), name, action)),
        _ => default::service_action(name, action),
    }
}
