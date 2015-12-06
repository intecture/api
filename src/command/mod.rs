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
//! Initialise a new Host using your managed host's IP address and
//! port number:
//!
//! ```no_run
//! # use inapi::Host;
//! let host = Host::new("127.0.0.1", 7101);
//! ```
//!
//! Now run your command and get the result:
//!
//! ```no_run
//! # use inapi::{Command, Host};
//! # let mut host = Host::new("127.0.0.1", 7101).unwrap();
//! let cmd = Command::new("whoami");
//! let result = cmd.exec(&mut host).unwrap();
//! println!("Exit: {}, Stdout: {}, Stderr: {}", result.exit_code, result.stdout, result.stderr);
//! ```
//!
//! If all goes well, this will output:
//!
//! > Exit: 0, Stdout: root, Stderr:

pub mod ffi;

use error::Result;
use host::Host;
use zmq;

/// Reusable container for sending commands to managed hosts.
pub struct Command {
    /// The shell command
    cmd: String,
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
    /// # use inapi::{Command, Host};
    /// let cmd = Command::new("whoami");
    ///
    /// let mut web1 = Host::new("web1.example.com", 7101).unwrap();
    /// let w1_result = cmd.exec(&mut web1).unwrap();
    ///
    /// let mut web2 = Host::new("web2.example.com", 7101).unwrap();
    /// let w2_result = cmd.exec(&mut web2).unwrap();
    /// ```
    pub fn exec(&self, host: &mut Host) -> Result<CommandResult> {
        try!(host.send("command::exec", zmq::SNDMORE));
        try!(host.send(&self.cmd, 0));

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

#[cfg(test)]
mod tests {
    use host::Host;
    use std::thread;
    use super::*;
    use zmq;

    #[test]
    fn test_exec() {
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

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_exec").unwrap();

        let mut host = Host::test_new(sock);

        let cmd = Command::new("moo");
        let result = cmd.exec(&mut host).unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "cow");
        assert_eq!(result.stderr, "err");

        agent_mock.join().unwrap();
    }
}