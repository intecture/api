// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Command

use host::ffi::Ffi__Host;
use host::Host;
use libc::c_char;
use std::convert;
use std::ffi::CString;
use std::panic::catch_unwind;
use super::{Command, CommandResult};

#[repr(C)]
pub struct Ffi__Command {
    cmd: *const c_char,
}

impl convert::From<Command> for Ffi__Command {
    fn from(command: Command) -> Ffi__Command {
        Ffi__Command {
            cmd: CString::new(command.cmd).unwrap().into_raw(),
        }
    }
}

impl convert::Into<Command> for Ffi__Command {
    fn into(self) -> Command {
        let cmd = trypanic!(ptrtostr!(self.cmd, "Command.cmd string"));
        Command::new(cmd)
    }
}

#[repr(C)]
pub struct Ffi__CommandResult {
    pub exit_code: i32,
    pub stdout: *const c_char,
    pub stderr: *const c_char,
}

impl convert::From<CommandResult> for Ffi__CommandResult {
    fn from(result: CommandResult) -> Ffi__CommandResult {
        Ffi__CommandResult {
            exit_code: result.exit_code,
            stdout: CString::new(result.stdout).unwrap().into_raw(),
            stderr: CString::new(result.stderr).unwrap().into_raw(),
        }
    }
}

#[no_mangle]
pub extern "C" fn command_new(cmd_ptr: *const c_char) -> *mut Ffi__Command {
    let cmd = trynull!(ptrtostr!(cmd_ptr, "path string"));

    let ffi_command: Ffi__Command = trynull!(catch_unwind(|| Command::new(cmd).into()));
    Box::into_raw(Box::new(ffi_command))
}

#[no_mangle]
pub extern "C" fn command_exec(ffi_cmd_ptr: *mut Ffi__Command, ffi_host_ptr: *mut Ffi__Host) -> *mut Ffi__CommandResult {
    let cmd: Command = trynull!(readptr!(ffi_cmd_ptr, "Command struct"));
    let mut host: Host = trynull!(readptr!(ffi_host_ptr, "Host struct"));

    let result = trynull!(cmd.exec(&mut host));
    let ffi_result: Ffi__CommandResult = trynull!(catch_unwind(|| result.into()));
    Box::into_raw(Box::new(ffi_result))
}

#[cfg(test)]
mod tests {
    use {Command, CommandResult};
    #[cfg(feature = "remote-run")]
    use Host;
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    use error::ERRMSG;
    use host::ffi::Ffi__Host;
    #[cfg(feature = "remote-run")]
    use host::ffi::host_close;
    #[cfg(feature = "remote-run")]
    use std::{str, thread};
    use std::ffi::CStr;
    use std::ffi::CString;
    use std::ptr;
    use super::*;

    #[test]
    fn test_convert_command() {
        let command = Command {
            cmd: "whoami".to_string(),
        };
        Ffi__Command::from(command);
    }

    #[test]
    fn test_convert_ffi_command() {
        let ffi_command = Ffi__Command {
            cmd: CString::new("whoami").unwrap().as_ptr(),
        };
        let _: Command = ffi_command.into();
    }

    #[test]
    fn test_convert_command_result() {
        let result = CommandResult {
            exit_code: 0,
            stdout: "moo".to_string(),
            stderr: "cow".to_string(),
        };
        Ffi__CommandResult::from(result);
    }

    #[test]
    fn test_command_new() {
        let cmd_cstr = CString::new("moo").unwrap().as_ptr();
        let ffi_cmd = unsafe { ptr::read(command_new(cmd_cstr)) };
        assert_eq!(ffi_cmd.cmd, cmd_cstr);

        assert!(command_new(ptr::null()).is_null());
        assert_eq!(unsafe { CStr::from_ptr(ERRMSG).to_str().unwrap() }, "Received null when we expected a path string pointer");
    }

    #[cfg(feature = "local-run")]
    #[test]
    fn test_command_exec() {
        let mut host = Ffi__Host;
        let cmd = command_new(CString::new("whoami").unwrap().as_ptr());
        let result = unsafe { ptr::read(command_exec(cmd, &mut host)) };
        assert_eq!(result.exit_code, 0);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_command_exec() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();
        client.set_sndtimeo(Some(500));
        server.set_rcvtimeo(Some(500));

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("command::exec", req.popstr().unwrap().unwrap());
            assert_eq!("moo", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("cow").unwrap();
            rep.addstr("err").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut ffi_host = Ffi__Host::from(Host::test_new(None, Some(client), None));
        let mut ffi_command = Ffi__Command {
            cmd: CString::new("moo").unwrap().into_raw(),
        };

        let result = unsafe { ptr::read(command_exec(&mut ffi_command, &mut ffi_host)) };

        assert_eq!(host_close(&mut ffi_host), 0);
        assert_eq!(result.exit_code, 0);

        let stdout = ptrtostr!(result.stdout, "stdout").unwrap();
        assert_eq!(stdout, "cow");

        let stderr = ptrtostr!(result.stderr, "stderr").unwrap();
        assert_eq!(stderr, "err");

        agent_mock.join().unwrap();
    }
}
