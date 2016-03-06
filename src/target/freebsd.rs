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
use regex::Regex;
use service::ServiceTarget;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
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
    fn file_mv(host: &mut Host, path: &str, new_path: &str) -> Result<()> {
        default::file_mv(path, new_path)
    }

    #[allow(unused_variables)]
    fn file_copy(host: &mut Host, path: &str, new_path: &str) -> Result<()> {
        default::file_copy(path, new_path)
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
        default::default_provider(host, vec![Providers::Pkg, Providers::Ports])
    }
}

//
// Service
//

impl ServiceTarget for Target {
    #[allow(unused_variables)]
    fn service_action(host: &mut Host, name: &str, action: &str) -> Result<CommandResult> {
        let mut rc_conf = try!(OpenOptions::new().read(true).write(true).open("/etc/rc.conf"));
        let mut rc = String::new();
        try!(rc_conf.read_to_string(&mut rc));

        let match_daemon = Regex::new(&format!("(?m)^\\s*{}_enable\\s*=\\s*[\"']{{0,1}}(?:YES|yes)[\"']{{0,1}}\n?", name)).unwrap();

        match action {
            "enable" => {
                if ! match_daemon.is_match(&rc) {
                    let newline = if rc.ends_with("\n") { "" } else { "\n" };
                    try!(rc_conf.write_all(&format!("{}{}_enable=\"YES\"\n", newline, name).into_bytes()));
                    try!(rc_conf.sync_data());
                }

                Ok(CommandResult{
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                })
            },
            "disable" => {
                if match_daemon.is_match(&rc) {
                    let replace = match_daemon.replace(&rc, "").trim().to_string();
                    try!(rc_conf.seek(SeekFrom::Start(0)));
                    try!(rc_conf.set_len(replace.len() as u64));
                    try!(rc_conf.write_all(replace.as_bytes()));
                    try!(rc_conf.sync_data());
                }

                Ok(CommandResult{
                    exit_code: 0,
                    stdout: String::new(),
                    stderr: String::new(),
                })
            },
            "start" | "stop" | "restart" => {
                if ! match_daemon.is_match(&rc) {
                    default::service_action(name, &format!("one{}", action))
                } else {
                    default::service_action(name, action)
                }
            },
            _ => default::service_action(name, action),
        }
    }
}

//
// Telemetry
//

impl TelemetryTarget for Target {
    #[allow(unused_variables)]
    fn telemetry_init(host: &mut Host) -> Result<Telemetry> {
        let cpu_vendor = try!(telemetry_cpu_vendor());
        let cpu_brand = try!(unix::get_sysctl_item("hw\\.model"));
        let hostname = try!(default::hostname());
        let os_version = try!(unix::version());

        Ok(Telemetry::new(
            Cpu::new(
                &cpu_vendor,
                &cpu_brand,
                try!(try!(unix::get_sysctl_item("hw\\.ncpu")).parse::<u32>()),
            ),
            try!(default::fs()),
            &hostname,
            try!(try!(unix::get_sysctl_item("hw\\.physmem")).parse::<u64>()),
            try!(unix::net()),
            Os::new(env::consts::ARCH, "unix", "freebsd", &os_version),
        ))
    }
}

fn telemetry_cpu_vendor() -> Result<String> {
    let mut fh = try!(File::open("/var/run/dmesg.boot"));
    let mut fc = String::new();
    fh.read_to_string(&mut fc).unwrap();

    let regex = Regex::new(r#"(?m)^CPU:.+$\n\s+Origin="([A-Za-z]+)""#).unwrap();
    if let Some(cap) = regex.captures(&fc) {
        Ok(cap.at(1).unwrap().to_string())
    } else {
        Err(Error::Generic("Could not match CPU vendor".to_string()))
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
