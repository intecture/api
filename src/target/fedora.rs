// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::{CommandResult, CommandTarget};
use directory::DirectoryTarget;
use error::Result;
use file::{FileTarget, FileOwner};
use host::Host;
use package::PackageTarget;
use package::providers::Providers;
use serde_json::Value;
use service::ServiceTarget;
use std::env;
use std::path::Path;
use super::{default_base as default, linux_base as linux, redhat_base as redhat};
use host::telemetry::{Cpu, Os, Telemetry, TelemetryTarget};

pub struct FedoraTarget;

//
// Command
//

impl CommandTarget for FedoraTarget {
    #[allow(unused_variables)]
    fn exec(host: &mut Host, cmd: &str) -> Result<CommandResult> {
        default::command_exec(cmd)
    }
}

//
// Directory
//

impl<P: AsRef<Path>> DirectoryTarget<P> for FedoraTarget {
    #[allow(unused_variables)]
    fn directory_is_directory(host: &mut Host, path: P) -> Result<bool> {
        default::directory_is_directory(path)
    }

    #[allow(unused_variables)]
    fn directory_exists(host: &mut Host, path: P) -> Result<bool> {
        default::file_exists(path)
    }

    #[allow(unused_variables)]
    fn directory_create(host: &mut Host, path: P, recursive: bool) -> Result<()> {
        default::directory_create(path, recursive)
    }

    #[allow(unused_variables)]
    fn directory_delete(host: &mut Host, path: P, recursive: bool) -> Result<()> {
        default::directory_delete(path, recursive)
    }

    #[allow(unused_variables)]
    fn directory_mv(host: &mut Host, path: P, new_path: P) -> Result<()> {
        default::file_mv(path, new_path)
    }

    #[allow(unused_variables)]
    fn directory_get_owner(host: &mut Host, path: P) -> Result<FileOwner> {
        linux::file_get_owner(path)
    }

    #[allow(unused_variables)]
    fn directory_set_owner(host: &mut Host, path: P, user: &str, group: &str) -> Result<()> {
        default::file_set_owner(path, user, group)
    }

    #[allow(unused_variables)]
    fn directory_get_mode(host: &mut Host, path: P) -> Result<u16> {
        linux::file_get_mode(path)
    }

    #[allow(unused_variables)]
    fn directory_set_mode(host: &mut Host, path: P, mode: u16) -> Result<()> {
        default::file_set_mode(path, mode)
    }
}

//
// File
//

impl<P: AsRef<Path>> FileTarget<P> for FedoraTarget {
    #[allow(unused_variables)]
    fn file_is_file(host: &mut Host, path: P) -> Result<bool> {
        default::file_is_file(path)
    }

    #[allow(unused_variables)]
    fn file_exists(host: &mut Host, path: P) -> Result<bool> {
        default::file_exists(path)
    }

    #[allow(unused_variables)]
    fn file_delete(host: &mut Host, path: P) -> Result<()> {
        default::file_delete(path)
    }

    #[allow(unused_variables)]
    fn file_mv(host: &mut Host, path: P, new_path: P) -> Result<()> {
        default::file_mv(path, new_path)
    }

    #[allow(unused_variables)]
    fn file_copy(host: &mut Host, path: P, new_path: P) -> Result<()> {
        default::file_copy(path, new_path)
    }

    #[allow(unused_variables)]
    fn file_get_owner(host: &mut Host, path: P) -> Result<FileOwner> {
        linux::file_get_owner(path)
    }

    #[allow(unused_variables)]
    fn file_set_owner(host: &mut Host, path: P, user: &str, group: &str) -> Result<()> {
        default::file_set_owner(path, user, group)
    }

    #[allow(unused_variables)]
    fn file_get_mode(host: &mut Host, path: P) -> Result<u16> {
        linux::file_get_mode(path)
    }

    #[allow(unused_variables)]
    fn file_set_mode(host: &mut Host, path: P, mode: u16) -> Result<()> {
        default::file_set_mode(path, mode)
    }
}

//
// Package
//

impl PackageTarget for FedoraTarget {
    fn default_provider(host: &mut Host) -> Result<Providers> {
        default::default_provider(host, vec![Providers::Dnf, Providers::Yum])
    }
}

//
// Service
//

impl ServiceTarget for FedoraTarget {
    #[allow(unused_variables)]
    fn service_action(host: &mut Host, name: &str, action: &str) -> Result<Option<CommandResult>> {
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

impl TelemetryTarget for FedoraTarget {
    #[allow(unused_variables)]
    fn telemetry_init(host: &mut Host) -> Result<Value> {
        let cpu_vendor = try!(linux::cpu_vendor());
        let cpu_brand = try!(linux::cpu_brand_string());
        let hostname = try!(default::hostname());
        let (version_str, version_maj, version_min, version_patch) = try!(redhat::version());

        let telemetry = Telemetry::new(
            Cpu::new(
                &cpu_vendor,
                &cpu_brand,
                try!(linux::cpu_cores())
            ),
            try!(default::fs()),
            &hostname,
            try!(linux::memory()),
            try!(linux::net()),
            Os::new(
                env::consts::ARCH,
                "redhat",
                "fedora",
                &version_str,
                version_maj,
                version_min,
                version_patch
            ),
        );

        Ok(telemetry.into_value())
    }
}
