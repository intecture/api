// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use {
    CommandResult,
    Host,
    Providers,
    Result,
    Cpu, Os, Telemetry,
};
use command::CommandTarget;
use error::Error;
use package::PackageTarget;
use std::{env, process, str};
use super::{default_base as default, Target, unix_base as unix};
use telemetry::TelemetryTarget;

//
// Command
//

impl CommandTarget for Target {
    #[allow(unused_variables)]
    fn exec(host: &mut Host, cmd: &str) -> Result<CommandResult> {
        default::command_exec(cmd)
    }
}

//
// Package
//

impl PackageTarget for Target {
    fn default_provider(host: &mut Host) -> Result<Providers> {
        default::default_provider(host, vec![Providers::Homebrew, Providers::Macports])
    }
}

//
// Telemetry
//

impl TelemetryTarget for Target {
    #[allow(unused_variables)]
    fn telemetry_init(host: &mut Host) -> Result<Telemetry> {
        let cpu_vendor = try!(unix::get_sysctl_item("machdep\\.cpu\\.vendor"));
        let cpu_brand_string = try!(unix::get_sysctl_item("machdep\\.cpu\\.brand_string"));
        let telemetry_version = try!(telemetry_version());
        let t = Telemetry::new(
            Cpu::new(
                &cpu_vendor,
                &cpu_brand_string,
                try!(try!(unix::get_sysctl_item("hw\\.physicalcpu")).parse::<u32>())
            ),
            try!(default::parse_fs(vec![
                default::FsFieldOrder::Filesystem,
                default::FsFieldOrder::Size,
                default::FsFieldOrder::Used,
                default::FsFieldOrder::Available,
                default::FsFieldOrder::Capacity,
                default::FsFieldOrder::Blank,
                default::FsFieldOrder::Blank,
                default::FsFieldOrder::Blank,
                default::FsFieldOrder::Mount,
            ])),
            try!(default::hostname()),
            try!(try!(unix::get_sysctl_item("hw\\.memsize")).parse::<u64>()),
            try!(unix::net()),
            Os::new(env::consts::ARCH, "unix", "macos", &telemetry_version),
        );

        Ok(t)
    }
}

fn telemetry_version() -> Result<String> {
    let output = try!(process::Command::new("sw_vers").arg("-productVersion").output());

    if output.status.success() == false {
        return Err(Error::Generic("Could not determine version".to_string()));
    }

    Ok(try!(str::from_utf8(&output.stdout)).trim().to_string())
}
