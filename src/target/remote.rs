// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
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
use directory::DirectoryTarget;
use file::{FileTarget, FileOwner};
use package::PackageTarget;
use rustc_serialize::json;
use service::ServiceTarget;
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
// Directory
//

impl DirectoryTarget for Target {
    fn directory_is_directory(host: &mut Host, path: &str) -> Result<bool> {
        try!(host.send("directory::is_directory", zmq::SNDMORE));
        try!(host.send(path, 0));

        try!(host.recv_header());
        let result = try!(host.expect_recv("is_directory", 1));
        Ok(result == "1")
    }

    fn directory_exists(host: &mut Host, path: &str) -> Result<bool> {
        try!(host.send("directory::exists", zmq::SNDMORE));
        try!(host.send(path, 0));

        try!(host.recv_header());
        let result = try!(host.expect_recv("exists", 1));
        Ok(result == "1")
    }

    fn directory_create(host: &mut Host, path: &str, recursive: bool) -> Result<()> {
        try!(host.send("directory::create", zmq::SNDMORE));
        try!(host.send(path, zmq::SNDMORE));
        try!(host.send(if recursive { "1" } else { "0" }, 0));
        try!(host.recv_header());
        Ok(())
    }

    fn directory_delete(host: &mut Host, path: &str, recursive: bool) -> Result<()> {
        try!(host.send("directory::delete", zmq::SNDMORE));
        try!(host.send(path, zmq::SNDMORE));
        try!(host.send(if recursive { "1" } else { "0" }, 0));
        try!(host.recv_header());
        Ok(())
    }

    fn directory_mv(host: &mut Host, path: &str, new_path: &str) -> Result<()> {
        try!(host.send("directory::mv", zmq::SNDMORE));
        try!(host.send(path, zmq::SNDMORE));
        try!(host.send(new_path, 0));
        try!(host.recv_header());
        Ok(())
    }

    fn directory_get_owner(host: &mut Host, path: &str) -> Result<FileOwner> {
        try!(host.send("directory::get_owner", zmq::SNDMORE));
        try!(host.send(path, 0));
        try!(host.recv_header());

        Ok(FileOwner {
            user_name: try!(host.expect_recv("user_name", 1)),
            user_uid: try!(host.expect_recv("user_uid", 2)).parse::<u64>().unwrap(),
            group_name: try!(host.expect_recv("group_name", 3)),
            group_gid: try!(host.expect_recv("group_gid", 4)).parse::<u64>().unwrap()
        })
    }

    fn directory_set_owner(host: &mut Host, path: &str, user: &str, group: &str) -> Result<()> {
        try!(host.send("directory::set_owner", zmq::SNDMORE));
        try!(host.send(path, zmq::SNDMORE));
        try!(host.send(user, zmq::SNDMORE));
        try!(host.send(group, 0));
        try!(host.recv_header());
        Ok(())
    }

    fn directory_get_mode(host: &mut Host, path: &str) -> Result<u16> {
        try!(host.send("directory::get_mode", zmq::SNDMORE));
        try!(host.send(path, 0));
        try!(host.recv_header());
        Ok(try!(host.expect_recv("mode", 1)).parse::<u16>().unwrap())
    }

    fn directory_set_mode(host: &mut Host, path: &str, mode: u16) -> Result<()> {
        try!(host.send("directory::set_mode", zmq::SNDMORE));
        try!(host.send(path, zmq::SNDMORE));
        try!(host.send(&mode.to_string(), 0));
        try!(host.recv_header());
        Ok(())
    }
}

//
// File
//

impl FileTarget for Target {
    fn file_is_file(host: &mut Host, path: &str) -> Result<bool> {
        try!(host.send("file::is_file", zmq::SNDMORE));
        try!(host.send(path, 0));

        try!(host.recv_header());
        let result = try!(host.expect_recv("is_file", 1));
        Ok(result == "1")
    }

    fn file_exists(host: &mut Host, path: &str) -> Result<bool> {
        try!(host.send("file::exists", zmq::SNDMORE));
        try!(host.send(path, 0));

        try!(host.recv_header());
        let result = try!(host.expect_recv("exists", 1));
        Ok(result == "1")
    }

    fn file_delete(host: &mut Host, path: &str) -> Result<()> {
        try!(host.send("file::delete", zmq::SNDMORE));
        try!(host.send(path, 0));
        try!(host.recv_header());
        Ok(())
    }

    fn file_mv(host: &mut Host, path: &str, new_path: &str) -> Result<()> {
        try!(host.send("file::mv", zmq::SNDMORE));
        try!(host.send(path, zmq::SNDMORE));
        try!(host.send(new_path, 0));
        try!(host.recv_header());
        Ok(())
    }

    fn file_copy(host: &mut Host, path: &str, new_path: &str) -> Result<()> {
        try!(host.send("file::copy", zmq::SNDMORE));
        try!(host.send(path, zmq::SNDMORE));
        try!(host.send(new_path, 0));
        try!(host.recv_header());
        Ok(())
    }

    fn file_get_owner(host: &mut Host, path: &str) -> Result<FileOwner> {
        try!(host.send("file::get_owner", zmq::SNDMORE));
        try!(host.send(path, 0));
        try!(host.recv_header());

        Ok(FileOwner {
            user_name: try!(host.expect_recv("user_name", 1)),
            user_uid: try!(host.expect_recv("user_uid", 2)).parse::<u64>().unwrap(),
            group_name: try!(host.expect_recv("group_name", 3)),
            group_gid: try!(host.expect_recv("group_gid", 4)).parse::<u64>().unwrap()
        })
    }

    fn file_set_owner(host: &mut Host, path: &str, user: &str, group: &str) -> Result<()> {
        try!(host.send("file::set_owner", zmq::SNDMORE));
        try!(host.send(path, zmq::SNDMORE));
        try!(host.send(user, zmq::SNDMORE));
        try!(host.send(group, 0));
        try!(host.recv_header());
        Ok(())
    }

    fn file_get_mode(host: &mut Host, path: &str) -> Result<u16> {
        try!(host.send("file::get_mode", zmq::SNDMORE));
        try!(host.send(path, 0));
        try!(host.recv_header());
        Ok(try!(host.expect_recv("mode", 1)).parse::<u16>().unwrap())
    }

    fn file_set_mode(host: &mut Host, path: &str, mode: u16) -> Result<()> {
        try!(host.send("file::set_mode", zmq::SNDMORE));
        try!(host.send(path, zmq::SNDMORE));
        try!(host.send(&mode.to_string(), 0));
        try!(host.recv_header());
        Ok(())
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
// Service
//

impl ServiceTarget for Target {
    fn service_action(host: &mut Host, name: &str, action: &str) -> Result<CommandResult> {
        try!(host.send("service::action", zmq::SNDMORE));
        try!(host.send(name, zmq::SNDMORE));
        try!(host.send(action, 0));
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
