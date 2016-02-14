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
use file::{FileTarget, FileOwner};
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
// File
//

impl FileTarget for Target {
    #[allow(unused_variables)]
    fn file_is_file(host: &mut Host, path: &str) -> Result<bool> {
        default::file_is_file(path)
    }

    #[allow(unused_variables)]
    fn file_exists(host: &mut Host, path: &str) -> Result<bool> {
        default::file_exists(path)
    }

    #[allow(unused_variables)]
    fn file_delete(host: &mut Host, path: &str) -> Result<()> {
        default::file_delete(path)
    }

    #[allow(unused_variables)]
    fn file_get_owner(host: &mut Host, path: &str) -> Result<FileOwner> {
        unix::file_get_owner(path)
    }

    #[allow(unused_variables)]
    fn file_set_owner(host: &mut Host, path: &str, user: &str, group: &str) -> Result<()> {
        default::file_set_owner(path, user, group)
    }

    #[allow(unused_variables)]
    fn file_get_mode(host: &mut Host, path: &str) -> Result<u16> {
        unix::file_get_mode(path)
    }

    #[allow(unused_variables)]
    fn file_set_mode(host: &mut Host, path: &str, mode: u16) -> Result<()> {
        default::file_set_mode(path, mode)
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
        let cpu_brand = try!(unix::get_sysctl_item("machdep\\.cpu\\.brand_string"));
        let hostname = try!(default::hostname());
        let telemetry_version = try!(telemetry_version());

        Ok(Telemetry::new(
            Cpu::new(
                &cpu_vendor,
                &cpu_brand,
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
            &hostname,
            try!(try!(unix::get_sysctl_item("hw\\.memsize")).parse::<u64>()),
            try!(unix::net()),
            Os::new(env::consts::ARCH, "unix", "macos", &telemetry_version),
        ))
    }
}

fn telemetry_version() -> Result<String> {
    let output = try!(process::Command::new("sw_vers").arg("-productVersion").output());

    if output.status.success() == false {
        return Err(Error::Generic("Could not determine version".to_string()));
    }

    Ok(try!(str::from_utf8(&output.stdout)).trim().to_string())
}

#[cfg(test)]
mod tests {
    use Host;
    use package::PackageTarget;
    use target::Target;
    use telemetry::TelemetryTarget;

    #[test]
    fn test_package_default_provider() {
        let mut host = Host::new();
        let result = Target::default_provider(&mut host);
        assert!(result.is_ok());
    }

    #[test]
    fn test_telemetry_init() {
        let mut host = Host::new();
        let result = Target::telemetry_init(&mut host);
        assert!(result.is_ok());
    }
}
