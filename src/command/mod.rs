// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
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
#![cfg_attr(feature = "local-run", doc = "let path: Option<String> = None;")]
#![cfg_attr(feature = "local-run", doc = "let mut host = Host::local(path).unwrap();")]
#![cfg_attr(feature = "remote-run", doc = "let mut host = Host::connect(\"nodes/mynode.json\").unwrap();")]
//! ```
//!
//! Now run your command and get the result:
//!
//! ```no_run
//! # use inapi::{Command, Host};
#![cfg_attr(feature = "local-run", doc = "let path: Option<String> = None;")]
#![cfg_attr(feature = "local-run", doc = "let mut host = Host::local(path).unwrap();")]
#![cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"nodes/mynode.json\").unwrap();")]
//! let cmd = Command::new("whoami");
//! let result = cmd.exec(&mut host).unwrap();
//! println!("Exit: {}, Stdout: {}, Stderr: {}", result.exit_code, result.stdout, result.stderr);
//! ```
//!
//! If all goes well, this will output:
//!
//! > Exit: 0, Stdout: <agent_runtime_user>, Stderr:

pub mod ffi;

use error::Result;
use host::Host;
use target::Target;

/// Reusable container for sending commands to managed hosts.
pub struct Command {
    /// The shell command
    cmd: String,
}

/// Result attributes returned from the managed host.
#[derive(Debug)]
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
            cmd: cmd.to_string(),
        }
    }

    /// Execute command on shell.
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
    #[cfg_attr(feature = "local-run", doc = "let path: Option<String> = None;")]
    #[cfg_attr(feature = "local-run", doc = "let mut web1 = Host::local(path).unwrap();")]
    #[cfg_attr(feature = "remote-run", doc = "let mut web1 = Host::connect(\"data/nodes/web1.json\").unwrap();")]
    /// let w1_result = cmd.exec(&mut web1).unwrap();
    ///
    #[cfg_attr(feature = "local-run", doc = "let path: Option<String> = None;")]
    #[cfg_attr(feature = "local-run", doc = "let mut web2 = Host::local(path).unwrap();")]
    #[cfg_attr(feature = "remote-run", doc = "let mut web2 = Host::connect(\"data/nodes/web2.json\").unwrap();")]
    /// let w2_result = cmd.exec(&mut web2).unwrap();
    /// ```
    #[allow(unused_variables)]
    pub fn exec(&self, host: &mut Host) -> Result<CommandResult> {
        Target::exec(host, &self.cmd)
    }
}

pub trait CommandTarget {
    fn exec(host: &mut Host, cmd: &str) -> Result<CommandResult>;
}

#[cfg(test)]
mod tests {
    use Host;
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    #[cfg(feature = "local-run")]
    use std::{process, str};
    #[cfg(feature = "remote-run")]
    use std::thread;
    use super::*;

    #[cfg(feature = "local-run")]
    #[test]
    fn test_exec() {
        let path: Option<String> = None;
        let mut host = Host::local(path).unwrap();
        let cmd = Command::new("whoami");
        let result = cmd.exec(&mut host).unwrap();

        let output = process::Command::new("sh").arg("-c").arg(&cmd.cmd).output().unwrap();

        assert_eq!(result.exit_code, output.status.code().unwrap());
        assert_eq!(result.stdout, str::from_utf8(&output.stdout).unwrap().trim().to_string());
        assert_eq!(result.stderr, str::from_utf8(&output.stderr).unwrap().trim().to_string());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_exec() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

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

        let mut host = Host::test_new(None, Some(client), None, None);

        let cmd = Command::new("moo");
        let result = cmd.exec(&mut host).unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "cow");
        assert_eq!(result.stderr, "err");

        agent_mock.join().unwrap();
    }
}
