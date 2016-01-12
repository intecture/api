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
};
use command::CommandTarget;
use package::PackageTarget;
use rustc_serialize::json;
use super::Target;
use telemetry::{Telemetry, TelemetryTarget};
use zmq;

//
// Command
//

impl CommandTarget for Target {
    fn exec(host: &mut Host, cmd: &str) -> Result<CommandResult> {
        try!(host.send("command::exec", zmq::SNDMORE));
        try!(host.send(cmd, 0));
        try!(host.recv_header());

        let exit_code = try!(host.expect_recvmsg("exit_code", 1)).as_str().unwrap().parse::<i32>().unwrap();
        let stdout = try!(host.expect_recv("stdout", 2));
        let stderr = try!(host.expect_recv("stderr", 3));

        Ok(CommandResult {
            exit_code: exit_code,
            stdout: stdout,
            stderr: stderr,
        })
    }
}

//
// Package
//

impl PackageTarget for Target {
    fn default_provider(host: &mut Host) -> Result<Providers> {
        try!(host.send("package::default_provider", 0));
        try!(host.recv_header());

        let provider = try!(host.expect_recv("provider", 1));

        Ok(Providers::from(provider))
    }
}

//
// Telemetry
//

impl TelemetryTarget for Target {
    fn telemetry_init(host: &mut Host) -> Result<Telemetry> {
        try!(host.send("telemetry", 0));
        try!(host.recv_header());

        let telemetry = try!(host.expect_recv("telemetry", 1));
        Ok(try!(json::decode(&telemetry)))
    }
}
