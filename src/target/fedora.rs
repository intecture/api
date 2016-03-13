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
use directory::DirectoryTarget;
use file::{FileTarget, FileOwner};
use package::PackageTarget;
use service::ServiceTarget;
use std::env;
use super::{default_base as default, linux_base as linux, redhat_base as redhat, Target};
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
// Directory
//

impl DirectoryTarget for Target {
    #[allow(unused_variables)]
    fn directory_is_directory(host: &mut Host, path: &str) -> Result<bool> {
        default::directory_is_directory(path)
    }

    #[allow(unused_variables)]
    fn directory_exists(host: &mut Host, path: &str) -> Result<bool> {
        default::file_exists(path)
    }

    #[allow(unused_variables)]
    fn directory_create(host: &mut Host, path: &str, recursive: bool) -> Result<()> {
        default::directory_create(path, recursive)
    }

    #[allow(unused_variables)]
    fn directory_delete(host: &mut Host, path: &str, recursive: bool) -> Result<()> {
        default::directory_delete(path, recursive)
    }

    #[allow(unused_variables)]
    fn directory_mv(host: &mut Host, path: &str, new_path: &str) -> Result<()> {
        default::file_mv(path, new_path)
    }

    #[allow(unused_variables)]
    fn directory_get_owner(host: &mut Host, path: &str) -> Result<FileOwner> {
        linux::file_get_owner(path)
    }

    #[allow(unused_variables)]
    fn directory_set_owner(host: &mut Host, path: &str, user: &str, group: &str) -> Result<()> {
        default::file_set_owner(path, user, group)
    }

    #[allow(unused_variables)]
    fn directory_get_mode(host: &mut Host, path: &str) -> Result<u16> {
        linux::file_get_mode(path)
    }

    #[allow(unused_variables)]
    fn directory_set_mode(host: &mut Host, path: &str, mode: u16) -> Result<()> {
        default::file_set_mode(path, mode)
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
    fn file_mv(host: &mut Host, path: &str, new_path: &str) -> Result<()> {
        default::file_mv(path, new_path)
    }

    #[allow(unused_variables)]
    fn file_copy(host: &mut Host, path: &str, new_path: &str) -> Result<()> {
        default::file_copy(path, new_path)
    }

    #[allow(unused_variables)]
    fn file_get_owner(host: &mut Host, path: &str) -> Result<FileOwner> {
        linux::file_get_owner(path)
    }

    #[allow(unused_variables)]
    fn file_set_owner(host: &mut Host, path: &str, user: &str, group: &str) -> Result<()> {
        default::file_set_owner(path, user, group)
    }

    #[allow(unused_variables)]
    fn file_get_mode(host: &mut Host, path: &str) -> Result<u16> {
        linux::file_get_mode(path)
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
        default::default_provider(host, vec![Providers::Dnf, Providers::Yum])
    }
}

//
// Service
//

impl ServiceTarget for Target {
    #[allow(unused_variables)]
    fn service_action(host: &mut Host, name: &str, action: &str) -> Result<CommandResult> {
        if try!(linux::using_systemd()) {
            linux::service_systemd(name, action)
        } else {
            redhat::service_init(name, action)
        }
    }
}

//
// Telemetry
//

impl TelemetryTarget for Target {
    #[allow(unused_variables)]
    fn telemetry_init(host: &mut Host) -> Result<Telemetry> {
        let cpu_vendor = try!(linux::cpu_vendor());
        let cpu_brand = try!(linux::cpu_brand_string());
        let hostname = try!(default::hostname());
        let os_version = try!(redhat::version());

        Ok(Telemetry::new(
            Cpu::new(
                &cpu_vendor,
                &cpu_brand,
                try!(linux::cpu_cores())
            ),
            try!(default::fs()),
            &hostname,
            try!(linux::memory()),
            try!(linux::net()),
            Os::new(env::consts::ARCH, "redhat", "fedora", &os_version),
        ))
    }
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
