// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Command

use ffi_helpers::Leaky;
use host::Host;
use libc::{c_char, uint8_t};
use std::convert;
use std::ffi::CString;
use std::panic::catch_unwind;
use super::{Command, CommandResult};

#[repr(C)]
pub struct Ffi__CommandResult {
    pub exit_code: i32,
    pub stdout: *mut c_char,
    pub stderr: *mut c_char,
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
pub extern "C" fn command_new(cmd_ptr: *const c_char) -> *mut Command {
    let cmd_string = trynull!(ptrtostr!(cmd_ptr, "command string"));
    Box::into_raw(Box::new(Command::new(cmd_string)))
}

#[no_mangle]
pub extern "C" fn command_exec(cmd_ptr: *mut Command, host_ptr: *mut Host) -> *mut Ffi__CommandResult {
    let cmd = Leaky::new(trynull!(readptr!(cmd_ptr, "Command pointer")));
    let mut host = Leaky::new(trynull!(readptr!(host_ptr, "Host pointer")));

    let result = trynull!(cmd.exec(&mut host));
    let ffi_result: Ffi__CommandResult = trynull!(catch_unwind(|| result.into()));

    Box::into_raw(Box::new(ffi_result))
}

#[no_mangle]
pub extern "C" fn command_free(cmd_ptr: *mut Command) -> uint8_t {
    tryrc!(boxptr!(cmd_ptr, "Command pointer"));
    0
}

#[no_mangle]
pub extern "C" fn command_result_free(result_ptr: *mut Ffi__CommandResult) -> uint8_t {
    let result = tryrc!(boxptr!(result_ptr, "CommandResult pointer"));
    if !result.stdout.is_null() {
        unsafe { CString::from_raw(result.stdout) };
    }
    if !result.stderr.is_null() {
        unsafe { CString::from_raw(result.stderr) };
    }
    0
}

#[cfg(test)]
mod tests {
    use command::CommandResult;
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    use error::ERRMSG;
    #[cfg(feature = "remote-run")]
    use host::ffi::host_close;
    use host::Host;
    #[cfg(feature = "remote-run")]
    use std::{str, thread};
    use std::ffi::CStr;
    use std::ffi::CString;
    use std::ptr;
    use super::*;

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
    fn test_new() {
        let cmd = readptr!(command_new(CString::new("moo").unwrap().into_raw()), "Command pointer").unwrap();
        assert_eq!(cmd.cmd, "moo");

        assert!(command_new(ptr::null()).is_null());
        assert_eq!(unsafe { CStr::from_ptr(ERRMSG).to_str().unwrap() }, "Received null when we expected a command string pointer");
    }

    #[cfg(feature = "local-run")]
    #[test]
    fn test_exec() {
        let path: Option<String> = None;
        let host = Box::into_raw(Box::new(Host::local(path).unwrap()));

        let whoami = CString::new("whoami").unwrap();
        let cmd = command_new(whoami.into_raw());
        assert!(!cmd.is_null());

        let result = readptr!(command_exec(cmd, host), "CommandResult pointer").unwrap();
        assert_eq!(result.exit_code, 0);

        assert_eq!(command_free(cmd), 0);
        unsafe { Box::from_raw(host) };
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_exec() {
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

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));
        let whoami = CString::new("moo").unwrap();
        let command = command_new(whoami.into_raw());

        let result = readptr!(command_exec(command, host), "CommandResult pointer").unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(ptrtostr!(result.stdout, "stdout").unwrap(), "cow");
        assert_eq!(ptrtostr!(result.stderr, "stderr").unwrap(), "err");

        assert_eq!(command_free(command), 0);
        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }
}
