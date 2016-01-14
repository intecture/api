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
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::Read;
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
        default::default_provider(host, vec![Providers::Pkg, Providers::Ports])
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