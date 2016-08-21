// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use {CommandResult, Error, Host, Providers, Result};
use command::CommandTarget;
use czmq::ZMsg;
use directory::DirectoryTarget;
use file::{FileTarget, FileOwner};
use package::PackageTarget;
use rustc_serialize::json;
use service::ServiceTarget;
use std::path::Path;
use super::Target;
use telemetry::{Telemetry, TelemetryTarget};

//
// Command
//

impl CommandTarget for Target {
    fn exec(host: &mut Host, cmd: &str) -> Result<CommandResult> {
        let msg = ZMsg::new();
        try!(msg.addstr("command::exec"));
        try!(msg.addstr(cmd));
        try!(host.send(msg));

        let msg = try!(host.recv(3, Some(3)));

        let exit_code = try!(msg.popstr().unwrap().or(Err(Error::HostResponse))).parse::<i32>().unwrap();
        let stdout = try!(msg.popstr().unwrap().or(Err(Error::HostResponse)));
        let stderr = try!(msg.popstr().unwrap().or(Err(Error::HostResponse)));

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

impl <P: AsRef<Path>> DirectoryTarget<P> for Target {
    fn directory_is_directory(host: &mut Host, path: P) -> Result<bool> {
        let msg = ZMsg::new();
        try!(msg.addstr("directory::is_directory"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(host.send(msg));

        let reply = try!(host.recv(1, Some(1)));
        Ok(try!(reply.popstr().unwrap().or(Err(Error::HostResponse))) == "1")
    }

    fn directory_exists(host: &mut Host, path: P) -> Result<bool> {
        let msg = ZMsg::new();
        try!(msg.addstr("directory::exists"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(host.send(msg));

        let reply = try!(host.recv(1, Some(1)));
        Ok(try!(reply.popstr().unwrap().or(Err(Error::HostResponse))) == "1")
    }

    fn directory_create(host: &mut Host, path: P, recursive: bool) -> Result<()> {
        let msg = ZMsg::new();
        try!(msg.addstr("directory::create"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(msg.addstr(if recursive { "1" } else { "0" }));
        try!(host.send(msg));
        try!(host.recv(0, None));
        Ok(())
    }

    fn directory_delete(host: &mut Host, path: P, recursive: bool) -> Result<()> {
        let msg = ZMsg::new();
        try!(msg.addstr("directory::delete"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(msg.addstr(if recursive { "1" } else { "0" }));
        try!(host.send(msg));
        try!(host.recv(0, None));
        Ok(())
    }

    fn directory_mv(host: &mut Host, path: P, new_path: P) -> Result<()> {
        let msg = ZMsg::new();
        try!(msg.addstr("directory::mv"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(msg.addstr(new_path.as_ref().to_str().unwrap()));
        try!(host.send(msg));
        try!(host.recv(0, None));
        Ok(())
    }

    fn directory_get_owner(host: &mut Host, path: P) -> Result<FileOwner> {
        let msg = ZMsg::new();
        try!(msg.addstr("directory::get_owner"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(host.send(msg));

        let reply = try!(host.recv(4, Some(4)));

        Ok(FileOwner {
            user_name: try!(reply.popstr().unwrap().or(Err(Error::HostResponse))),
            user_uid: try!(reply.popstr().unwrap().or(Err(Error::HostResponse))).parse::<u64>().unwrap(),
            group_name: try!(reply.popstr().unwrap().or(Err(Error::HostResponse))),
            group_gid: try!(reply.popstr().unwrap().or(Err(Error::HostResponse))).parse::<u64>().unwrap()
        })
    }

    fn directory_set_owner(host: &mut Host, path: P, user: &str, group: &str) -> Result<()> {
        let msg = ZMsg::new();
        try!(msg.addstr("directory::set_owner"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(msg.addstr(user));
        try!(msg.addstr(group));
        try!(host.send(msg));
        try!(host.recv(0, None));
        Ok(())
    }

    fn directory_get_mode(host: &mut Host, path: P) -> Result<u16> {
        let msg = ZMsg::new();
        try!(msg.addstr("directory::get_mode"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(host.send(msg));

        let reply = try!(host.recv(0, None));
        Ok(try!(reply.popstr().unwrap().or(Err(Error::HostResponse))).parse::<u16>().unwrap())
    }

    fn directory_set_mode(host: &mut Host, path: P, mode: u16) -> Result<()> {
        let msg = ZMsg::new();
        try!(msg.addstr("directory::set_mode"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(msg.addstr(&mode.to_string()));
        try!(host.send(msg));
        try!(host.recv(0, None));
        Ok(())
    }
}

//
// File
//

impl <P: AsRef<Path>> FileTarget<P> for Target {
    fn file_is_file(host: &mut Host, path: P) -> Result<bool> {
        let msg = ZMsg::new();
        try!(msg.addstr("file::is_file"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(host.send(msg));

        let reply = try!(host.recv(0, None));
        Ok(try!(reply.popstr().unwrap().or(Err(Error::HostResponse))) == "1")
    }

    fn file_exists(host: &mut Host, path: P) -> Result<bool> {
        let msg = ZMsg::new();
        try!(msg.addstr("file::exists"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(host.send(msg));

        let reply = try!(host.recv(1, Some(1)));
        Ok(try!(reply.popstr().unwrap().or(Err(Error::HostResponse))) == "1")
    }

    fn file_delete(host: &mut Host, path: P) -> Result<()> {
        let msg = ZMsg::new();
        try!(msg.addstr("file::delete"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(host.send(msg));
        try!(host.recv(0, None));
        Ok(())
    }

    fn file_mv(host: &mut Host, path: P, new_path: P) -> Result<()> {
        let msg = ZMsg::new();
        try!(msg.addstr("file::mv"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(msg.addstr(new_path.as_ref().to_str().unwrap()));
        try!(host.send(msg));
        try!(host.recv(0, None));
        Ok(())
    }

    fn file_copy(host: &mut Host, path: P, new_path: P) -> Result<()> {
        let msg = ZMsg::new();
        try!(msg.addstr("file::copy"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(msg.addstr(new_path.as_ref().to_str().unwrap()));
        try!(host.send(msg));
        try!(host.recv(0, None));
        Ok(())
    }

    fn file_get_owner(host: &mut Host, path: P) -> Result<FileOwner> {
        let msg = ZMsg::new();
        try!(msg.addstr("file::get_owner"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(host.send(msg));

        let reply = try!(host.recv(4, Some(4)));

        Ok(FileOwner {
            user_name: try!(reply.popstr().unwrap().or(Err(Error::HostResponse))),
            user_uid: try!(reply.popstr().unwrap().or(Err(Error::HostResponse))).parse::<u64>().unwrap(),
            group_name: try!(reply.popstr().unwrap().or(Err(Error::HostResponse))),
            group_gid: try!(reply.popstr().unwrap().or(Err(Error::HostResponse))).parse::<u64>().unwrap()
        })
    }

    fn file_set_owner(host: &mut Host, path: P, user: &str, group: &str) -> Result<()> {
        let msg = ZMsg::new();
        try!(msg.addstr("file::set_owner"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(msg.addstr(user));
        try!(msg.addstr(group));
        try!(host.send(msg));
        try!(host.recv(0, None));
        Ok(())
    }

    fn file_get_mode(host: &mut Host, path: P) -> Result<u16> {
        let msg = ZMsg::new();
        try!(msg.addstr("file::get_mode"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(host.send(msg));

        let reply = try!(host.recv(0, None));
        Ok(try!(reply.popstr().unwrap().or(Err(Error::HostResponse))).parse::<u16>().unwrap())
    }

    fn file_set_mode(host: &mut Host, path: P, mode: u16) -> Result<()> {
        let msg = ZMsg::new();
        try!(msg.addstr("file::set_mode"));
        try!(msg.addstr(path.as_ref().to_str().unwrap()));
        try!(msg.addstr(&mode.to_string()));
        try!(host.send(msg));
        try!(host.recv(0, None));
        Ok(())
    }
}

//
// Package
//

impl PackageTarget for Target {
    fn default_provider(host: &mut Host) -> Result<Providers> {
        let msg = ZMsg::new();
        try!(msg.addstr("package::default_provider"));
        try!(host.send(msg));

        let reply = try!(host.recv(1, Some(1)));
        Ok(Providers::from(try!(reply.popstr().unwrap().or(Err(Error::HostResponse)))))
    }
}

//
// Service
//

impl ServiceTarget for Target {
    fn service_action(host: &mut Host, name: &str, action: &str) -> Result<Option<CommandResult>> {
        let msg = ZMsg::new();
        try!(msg.addstr("service::action"));
        try!(msg.addstr(name));
        try!(msg.addstr(action));
        try!(host.send(msg));

        let msg = try!(host.recv(0, Some(3)));

        if msg.size() == 0 {
            Ok(None)
        }
        else if msg.size() == 3 {
            Ok(Some(CommandResult {
                exit_code: try!(msg.popstr().unwrap().or(Err(Error::HostResponse))).parse::<i32>().unwrap(),
                stdout: try!(msg.popstr().unwrap().or(Err(Error::HostResponse))),
                stderr: try!(msg.popstr().unwrap().or(Err(Error::HostResponse))),
            }))
        } else {
            Err(Error::HostResponse)
        }
    }
}

//
// Telemetry
//

impl TelemetryTarget for Target {
    fn telemetry_init(host: &mut Host) -> Result<Telemetry> {
        let msg = ZMsg::new();
        try!(msg.addstr("telemetry"));
        try!(host.send(msg));

        let msg = try!(host.recv(1, Some(1)));
        let telemetry = try!(msg.popstr().unwrap().or(Err(Error::HostResponse)));
        Ok(try!(json::decode(&telemetry)))
    }
}
