// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::{Command, CommandResult};
use libc::{c_char, c_void};
use std::{convert, str};
use std::ffi::{CStr, CString};
use std::ptr;
use zmq;

#[repr(C)]
pub struct Ffi__Command {
    cmd: *const c_char,
}

impl convert::From<Command> for Ffi__Command {
    fn from(command: Command) -> Ffi__Command {
        Ffi__Command {
            cmd: CString::new(command.cmd).unwrap().as_ptr(),
        }
    }
}

impl convert::From<Ffi__Command> for Command {
    fn from(ffi_cmd: Ffi__Command) -> Command {
        let slice = unsafe { CStr::from_ptr(ffi_cmd.cmd) };
        let cmd_str = str::from_utf8(slice.to_bytes()).unwrap();

        Command {
            cmd: cmd_str.to_string(),
        }
    }
}

#[repr(C)]
pub struct Ffi__CommandResult {
    exit_code: i32,
    stdout: *const c_char,
    stderr: *const c_char,
}

impl convert::From<CommandResult> for Ffi__CommandResult {
    fn from(result: CommandResult) -> Ffi__CommandResult {
        Ffi__CommandResult {
            exit_code: result.exit_code,
            stdout: CString::new(result.stdout).unwrap().as_ptr(),
            stderr: CString::new(result.stderr).unwrap().as_ptr(),
        }
    }
}

#[no_mangle]
pub extern "C" fn command_new(cmd: *const c_char) -> Ffi__Command {
    let slice = unsafe { CStr::from_ptr(cmd) };
    let cmd_str = str::from_utf8(slice.to_bytes()).unwrap();
    Ffi__Command::from(Command::new(cmd_str))
}

#[no_mangle]
pub extern "C" fn command_exec(ffi_cmd_ptr: *const Ffi__Command, raw_sock: *mut c_void) -> Ffi__CommandResult {
    let cmd = Command::from(unsafe { ptr::read(ffi_cmd_ptr) });
    Ffi__CommandResult::from(cmd.exec(&mut zmq::Socket::from_raw(raw_sock, true)).unwrap())
}

#[cfg(test)]
mod tests {
    extern crate zmq_sys;

    use std::ffi::{CString, CStr};
    use std::{str, thread};
    use zmq;

    #[test]
    fn test_command_new() {
        let cmd_cstr = CString::new("moo").unwrap().as_ptr();
        let ffi_cmd = super::command_new(cmd_cstr);
        assert_eq!(ffi_cmd.cmd, cmd_cstr);
    }

    #[test]
    fn test_command_exec() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_exec").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert!(agent_sock.get_rcvmore().unwrap());
            assert_eq!("moo", agent_sock.recv_string(0).unwrap().unwrap());

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("cow", zmq::SNDMORE).unwrap();
            agent_sock.send_str("err", 0).unwrap();
        });

        let mut req_sock = ctx.socket(zmq::REQ).unwrap();
        req_sock.connect("inproc://test_exec").unwrap();

        let ffi_command = super::Ffi__Command {
            cmd: CString::new("moo").unwrap().as_ptr(),
        };
        let result = super::command_exec(&ffi_command, req_sock.to_raw());

        assert_eq!(result.exit_code, 0);

        let stdout_slice = unsafe { CStr::from_ptr(result.stdout) };
        let stdout = str::from_utf8(stdout_slice.to_bytes()).unwrap();
        assert_eq!(stdout, "cow");

        let stderr_slice = unsafe { CStr::from_ptr(result.stderr) };
        let stderr = str::from_utf8(stderr_slice.to_bytes()).unwrap();
        assert_eq!(stderr, "err");

        agent_mock.join().unwrap();
    }
}