// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The shell command primitive for running commands on a managed
//! host.
//!
//! # Examples
//!
//! Setup your ZMQ socket, using your managed host's IP address in
//! place of *127.0.0.1*:
//!
//! ```no_run
//! # #[macro_use] extern crate zmq;
//! # fn main() {
//! let mut ctx = zmq::Context::new();
//! let mut zmq_socket = ctx.socket(zmq::REQ).unwrap();
//! zmq_socket.connect("tcp://127.0.0.1:7101").unwrap();
//! # }
//! ```
//!
//! Now run your command and get the result:
//!
//! ```no_run
//! # #[macro_use] extern crate zmq;
//! # #[macro_use] extern crate inapi;
//! # use inapi::Command;
//! # fn main() {
//! # let mut ctx = zmq::Context::new();
//! # let mut zmq_socket = ctx.socket(zmq::REQ).unwrap();
//! let cmd = Command::new("whoami");
//! let result = cmd.exec(&mut zmq_socket).unwrap();
//! println!("Exit: {}, Stdout: {}, Stderr: {}", result.exit_code, result.stdout, result.stderr);
//! # }
//! ```
//!
//! If all goes well, this will output:
//!
//! > Exit: 0, Stdout: root, Stderr:

use ::MissingFrameError;
use std::convert;
use zmq;

/// Reusable container for sending commands to managed hosts.
#[derive(Debug)]
pub struct Command {
    /// The shell command
    pub cmd: String,
}

/// Result attributes returned from the managed host.
pub struct CommandResult {
    /// Exit code for the shell command's process
    pub exit_code: i32,
    /// Process's standard output
    pub stdout: String,
    /// Process's standard error
    pub stderr: String,
}

impl Command {
    /// Create a new Command to represent your shell command.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::Command;
    /// let cmd = Command::new("your shell command goes here");
    /// ```
    pub fn new(cmd: &str) -> Command {
        Command {
            cmd: String::from(cmd),
        }
    }

    /// Send request to the Agent to run your shell command.
    ///
    /// Command structs are reusable accross multiple hosts, which is
    /// helpful if you are configuring a group of servers
    /// simultaneously.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[macro_use] extern crate zmq;
    /// # #[macro_use] extern crate inapi;
    /// # use inapi::Command;
    /// # fn main() {
    /// # let mut ctx = zmq::Context::new();
    /// # let mut web1_sock = ctx.socket(zmq::REQ).unwrap();
    /// # let mut web2_sock = ctx.socket(zmq::REQ).unwrap();
    /// let cmd = Command::new("whoami");
    /// let result_web1 = cmd.exec(&mut web1_sock).unwrap();
    /// let result_web2 = cmd.exec(&mut web2_sock).unwrap();
    /// # }
    /// ```
    pub fn exec(&self, sock: &mut zmq::Socket) -> Result<CommandResult, CommandError> {
        try!(sock.send_str("command::exec", zmq::SNDMORE));
        try!(sock.send_str(&self.cmd, 0));

        let status = try!(sock.recv_string(0));
        if status.unwrap() == "Err" {
            if sock.get_rcvmore().unwrap() == false {
                return Err(CommandError::FrameError(MissingFrameError { order: 1, name: "err_msg" }));
            }

            return Err(CommandError::AgentError(try!(sock.recv_string(0)).unwrap()));
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
    /// An error string returned from the host's Intecture Agent
    AgentError(String),
    /// Message frames missing in the response from host's Intecture Agent
    FrameError(MissingFrameError),
    /// ZMQ transmission error
    ZmqError(zmq::Error),
}

impl convert::From<zmq::Error> for CommandError {
	fn from(err: zmq::Error) -> CommandError {
		CommandError::ZmqError(err)
	}
}

#[cfg(test)]
mod tests {
    use super::*;
    use zmq;
    use std::thread;

    #[test]
    fn test_exec() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();

        let agent_mock = thread::spawn(move || {
            agent_sock.bind("inproc://test_exec").unwrap();

            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert!(agent_sock.get_rcvmore().unwrap());
            assert_eq!("moo", agent_sock.recv_string(0).unwrap().unwrap());

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("cow", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();
        });

        let mut req_sock = ctx.socket(zmq::REQ).unwrap();
        req_sock.connect("inproc://test_exec").unwrap();

        let cmd = Command::new("moo");
        cmd.exec(&mut req_sock).unwrap();
        
        agent_mock.join().unwrap();
    }
}