use ::MissingFrameError;
use libc::{c_char, c_void};
use std::{convert, str};
use std::ffi::{CStr, CString};
use std::ptr;
use zmq;

pub struct Command {
    cmd: String,
}

pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Command {
    pub fn new(cmd: &str) -> Command {
        Command {
            cmd: String::from(cmd),
        }
    }

    pub fn exec(&mut self, sock: &mut zmq::Socket) -> Result<CommandResult, CommandError> {
        try!(sock.send_str("command::exec", zmq::SNDMORE));
        try!(sock.send_str(&self.cmd, 0));

        let status = try!(sock.recv_string(0));
        if status.unwrap() == "Err" {
            if sock.get_rcvmore().unwrap() == false {
                return Err(CommandError::FrameError(MissingFrameError { order: 1, name: "err_msg" }));
            }

            return Err(CommandError::AgentError(AgentErrorError { msg: try!(sock.recv_string(0)).unwrap() }));
        }

        let exit_code = try!(sock.recv_msg(0)).as_str().unwrap().parse::<i32>().unwrap();

        if sock.get_rcvmore().unwrap() == false {
            return Err(CommandError::FrameError(MissingFrameError { order: 1, name: "stdout" }));
        }

        let stdout = try!(sock.recv_string(0)).unwrap();

        if sock.get_rcvmore().unwrap() == false {
            return Err(CommandError::FrameError(MissingFrameError { order: 1, name: "stderr" }));
        }

        let stderr = try!(sock.recv_string(0)).unwrap();

        Ok(CommandResult {
            exit_code: exit_code,
            stdout: stdout,
            stderr: stderr,
        })
    }
}

#[derive(Debug)]
pub enum CommandError {
    AgentError(AgentErrorError),
    FrameError(MissingFrameError),
    ZmqError(zmq::Error),
}

#[derive(Debug)]
pub struct AgentErrorError {
    msg: String
}

impl convert::From<zmq::Error> for CommandError {
	fn from(err: zmq::Error) -> CommandError {
		CommandError::ZmqError(err)
	}
}

//
// FFI
//

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
    let mut cmd = Command::from(unsafe { ptr::read(ffi_cmd_ptr) });
    Ffi__CommandResult::from(cmd.exec(&mut zmq::Socket::new(raw_sock, true)).unwrap())
}