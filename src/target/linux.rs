// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
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
use service::ServiceTarget;
use std::fs;
use std::path::Path;
use std::sync::{Once, ONCE_INIT};
use super::Target;
use super::centos::CentosTarget;
use super::debian::DebianTarget;
use super::fedora::FedoraTarget;
use super::redhat::RedhatTarget;
use super::ubuntu::UbuntuTarget;
use telemetry::{Telemetry, TelemetryTarget};

static mut LINUX_PLATFORM: LinuxPlatform = LinuxPlatform::Centos;
static INIT_FINGERPRINT: Once = ONCE_INIT;

enum LinuxPlatform {
    Centos,
    Debian,
    Fedora,
    Redhat,
    Ubuntu,
}

//
// Command
//

impl CommandTarget for Target {
    fn exec(host: &mut Host, cmd: &str) -> Result<CommandResult> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::exec(host, cmd),
            &LinuxPlatform::Debian => DebianTarget::exec(host, cmd),
            &LinuxPlatform::Fedora => FedoraTarget::exec(host, cmd),
            &LinuxPlatform::Redhat => RedhatTarget::exec(host, cmd),
            &LinuxPlatform::Ubuntu => UbuntuTarget::exec(host, cmd),
        }
    }
}

//
// Directory
//

impl<P: AsRef<Path>> DirectoryTarget<P> for Target {
    fn directory_is_directory(host: &mut Host, path: P) -> Result<bool> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::directory_is_directory(host, path),
            &LinuxPlatform::Debian => DebianTarget::directory_is_directory(host, path),
            &LinuxPlatform::Fedora => FedoraTarget::directory_is_directory(host, path),
            &LinuxPlatform::Redhat => RedhatTarget::directory_is_directory(host, path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::directory_is_directory(host, path),
        }
    }

    fn directory_exists(host: &mut Host, path: P) -> Result<bool> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::directory_exists(host, path),
            &LinuxPlatform::Debian => DebianTarget::directory_exists(host, path),
            &LinuxPlatform::Fedora => FedoraTarget::directory_exists(host, path),
            &LinuxPlatform::Redhat => RedhatTarget::directory_exists(host, path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::directory_exists(host, path),
        }
    }

    fn directory_create(host: &mut Host, path: P, recursive: bool) -> Result<()> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::directory_create(host, path, recursive),
            &LinuxPlatform::Debian => DebianTarget::directory_create(host, path, recursive),
            &LinuxPlatform::Fedora => FedoraTarget::directory_create(host, path, recursive),
            &LinuxPlatform::Redhat => RedhatTarget::directory_create(host, path, recursive),
            &LinuxPlatform::Ubuntu => UbuntuTarget::directory_create(host, path, recursive),
        }
    }

    fn directory_delete(host: &mut Host, path: P, recursive: bool) -> Result<()> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::directory_delete(host, path, recursive),
            &LinuxPlatform::Debian => DebianTarget::directory_delete(host, path, recursive),
            &LinuxPlatform::Fedora => FedoraTarget::directory_delete(host, path, recursive),
            &LinuxPlatform::Redhat => RedhatTarget::directory_delete(host, path, recursive),
            &LinuxPlatform::Ubuntu => UbuntuTarget::directory_delete(host, path, recursive),
        }
    }

    fn directory_mv(host: &mut Host, path: P, new_path: P) -> Result<()> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::directory_mv(host, path, new_path),
            &LinuxPlatform::Debian => DebianTarget::directory_mv(host, path, new_path),
            &LinuxPlatform::Fedora => FedoraTarget::directory_mv(host, path, new_path),
            &LinuxPlatform::Redhat => RedhatTarget::directory_mv(host, path, new_path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::directory_mv(host, path, new_path),
        }
    }

    fn directory_get_owner(host: &mut Host, path: P) -> Result<FileOwner> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::directory_get_owner(host, path),
            &LinuxPlatform::Debian => DebianTarget::directory_get_owner(host, path),
            &LinuxPlatform::Fedora => FedoraTarget::directory_get_owner(host, path),
            &LinuxPlatform::Redhat => RedhatTarget::directory_get_owner(host, path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::directory_get_owner(host, path),
        }
    }

    fn directory_set_owner(host: &mut Host, path: P, user: &str, group: &str) -> Result<()> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::directory_set_owner(host, path, user, group),
            &LinuxPlatform::Debian => DebianTarget::directory_set_owner(host, path, user, group),
            &LinuxPlatform::Fedora => FedoraTarget::directory_set_owner(host, path, user, group),
            &LinuxPlatform::Redhat => RedhatTarget::directory_set_owner(host, path, user, group),
            &LinuxPlatform::Ubuntu => UbuntuTarget::directory_set_owner(host, path, user, group),
        }
    }

    fn directory_get_mode(host: &mut Host, path: P) -> Result<u16> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::directory_get_mode(host, path),
            &LinuxPlatform::Debian => DebianTarget::directory_get_mode(host, path),
            &LinuxPlatform::Fedora => FedoraTarget::directory_get_mode(host, path),
            &LinuxPlatform::Redhat => RedhatTarget::directory_get_mode(host, path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::directory_get_mode(host, path),
        }
    }

    fn directory_set_mode(host: &mut Host, path: P, mode: u16) -> Result<()> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::directory_set_mode(host, path, mode),
            &LinuxPlatform::Debian => DebianTarget::directory_set_mode(host, path, mode),
            &LinuxPlatform::Fedora => FedoraTarget::directory_set_mode(host, path, mode),
            &LinuxPlatform::Redhat => RedhatTarget::directory_set_mode(host, path, mode),
            &LinuxPlatform::Ubuntu => UbuntuTarget::directory_set_mode(host, path, mode),
        }
    }
}

//
// File
//

impl<P: AsRef<Path>> FileTarget<P> for Target {
    fn file_is_file(host: &mut Host, path: P) -> Result<bool> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::file_is_file(host, path),
            &LinuxPlatform::Debian => DebianTarget::file_is_file(host, path),
            &LinuxPlatform::Fedora => FedoraTarget::file_is_file(host, path),
            &LinuxPlatform::Redhat => RedhatTarget::file_is_file(host, path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::file_is_file(host, path),
        }
    }

    fn file_exists(host: &mut Host, path: P) -> Result<bool> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::file_exists(host, path),
            &LinuxPlatform::Debian => DebianTarget::file_exists(host, path),
            &LinuxPlatform::Fedora => FedoraTarget::file_exists(host, path),
            &LinuxPlatform::Redhat => RedhatTarget::file_exists(host, path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::file_exists(host, path),
        }
    }

    fn file_delete(host: &mut Host, path: P) -> Result<()> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::file_delete(host, path),
            &LinuxPlatform::Debian => DebianTarget::file_delete(host, path),
            &LinuxPlatform::Fedora => FedoraTarget::file_delete(host, path),
            &LinuxPlatform::Redhat => RedhatTarget::file_delete(host, path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::file_delete(host, path),
        }
    }

    fn file_mv(host: &mut Host, path: P, new_path: P) -> Result<()> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::file_mv(host, path, new_path),
            &LinuxPlatform::Debian => DebianTarget::file_mv(host, path, new_path),
            &LinuxPlatform::Fedora => FedoraTarget::file_mv(host, path, new_path),
            &LinuxPlatform::Redhat => RedhatTarget::file_mv(host, path, new_path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::file_mv(host, path, new_path),
        }
    }

    fn file_copy(host: &mut Host, path: P, new_path: P) -> Result<()> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::file_copy(host, path, new_path),
            &LinuxPlatform::Debian => DebianTarget::file_copy(host, path, new_path),
            &LinuxPlatform::Fedora => FedoraTarget::file_copy(host, path, new_path),
            &LinuxPlatform::Redhat => RedhatTarget::file_copy(host, path, new_path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::file_copy(host, path, new_path),
        }
    }

    fn file_get_owner(host: &mut Host, path: P) -> Result<FileOwner> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::file_get_owner(host, path),
            &LinuxPlatform::Debian => DebianTarget::file_get_owner(host, path),
            &LinuxPlatform::Fedora => FedoraTarget::file_get_owner(host, path),
            &LinuxPlatform::Redhat => RedhatTarget::file_get_owner(host, path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::file_get_owner(host, path),
        }
    }

    fn file_set_owner(host: &mut Host, path: P, user: &str, group: &str) -> Result<()> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::file_set_owner(host, path, user, group),
            &LinuxPlatform::Debian => DebianTarget::file_set_owner(host, path, user, group),
            &LinuxPlatform::Fedora => FedoraTarget::file_set_owner(host, path, user, group),
            &LinuxPlatform::Redhat => RedhatTarget::file_set_owner(host, path, user, group),
            &LinuxPlatform::Ubuntu => UbuntuTarget::file_set_owner(host, path, user, group),
        }
    }

    fn file_get_mode(host: &mut Host, path: P) -> Result<u16> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::file_get_mode(host, path),
            &LinuxPlatform::Debian => DebianTarget::file_get_mode(host, path),
            &LinuxPlatform::Fedora => FedoraTarget::file_get_mode(host, path),
            &LinuxPlatform::Redhat => RedhatTarget::file_get_mode(host, path),
            &LinuxPlatform::Ubuntu => UbuntuTarget::file_get_mode(host, path),
        }
    }

    fn file_set_mode(host: &mut Host, path: P, mode: u16) -> Result<()> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::file_set_mode(host, path, mode),
            &LinuxPlatform::Debian => DebianTarget::file_set_mode(host, path, mode),
            &LinuxPlatform::Fedora => FedoraTarget::file_set_mode(host, path, mode),
            &LinuxPlatform::Redhat => RedhatTarget::file_set_mode(host, path, mode),
            &LinuxPlatform::Ubuntu => UbuntuTarget::file_set_mode(host, path, mode),
        }
    }
}

//
// Package
//

impl PackageTarget for Target {
    fn default_provider(host: &mut Host) -> Result<Providers> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::default_provider(host),
            &LinuxPlatform::Debian => DebianTarget::default_provider(host),
            &LinuxPlatform::Fedora => FedoraTarget::default_provider(host),
            &LinuxPlatform::Redhat => RedhatTarget::default_provider(host),
            &LinuxPlatform::Ubuntu => UbuntuTarget::default_provider(host),
        }
    }
}

//
// Service
//

impl ServiceTarget for Target {
    fn service_action(host: &mut Host, name: &str, action: &str) -> Result<Option<CommandResult>> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::service_action(host, name, action),
            &LinuxPlatform::Debian => DebianTarget::service_action(host, name, action),
            &LinuxPlatform::Fedora => FedoraTarget::service_action(host, name, action),
            &LinuxPlatform::Redhat => RedhatTarget::service_action(host, name, action),
            &LinuxPlatform::Ubuntu => UbuntuTarget::service_action(host, name, action),
        }
    }
}

//
// Telemetry
//

impl TelemetryTarget for Target {
    fn telemetry_init(host: &mut Host) -> Result<Telemetry> {
        match fingerprint_os() {
            &LinuxPlatform::Centos => CentosTarget::telemetry_init(host),
            &LinuxPlatform::Debian => DebianTarget::telemetry_init(host),
            &LinuxPlatform::Fedora => FedoraTarget::telemetry_init(host),
            &LinuxPlatform::Redhat => RedhatTarget::telemetry_init(host),
            &LinuxPlatform::Ubuntu => UbuntuTarget::telemetry_init(host),
        }
    }
}

fn fingerprint_os() -> &'static LinuxPlatform {
    INIT_FINGERPRINT.call_once(|| {
        // CentOS
        if let Ok(_) = fs::metadata("/etc/centos-release") {
            unsafe { LINUX_PLATFORM = LinuxPlatform::Centos; }
        }
        // Ubuntu
        else if let Ok(_) = fs::metadata("/etc/lsb-release") {
            unsafe { LINUX_PLATFORM = LinuxPlatform::Ubuntu; }
        }
        // Debian
        else if let Ok(_) = fs::metadata("/etc/debian_version") {
            unsafe { LINUX_PLATFORM = LinuxPlatform::Debian; }
        }
        // Fedora
        else if let Ok(_) = fs::metadata("/etc/fedora-release") {
            unsafe { LINUX_PLATFORM = LinuxPlatform::Fedora; }
        }
        // RedHat
        else if let Ok(_) = fs::metadata("/etc/redhat-release") {
            unsafe { LINUX_PLATFORM = LinuxPlatform::Redhat; }
        } else {
            panic!("Unknown Linux distro");
        }
    });

    unsafe { &LINUX_PLATFORM }
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
