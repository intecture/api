// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Endpoint for running shell commands.

pub mod providers;

use errors::*;
use futures::{future, Future};
use host::Host;
use self::providers::CommandProvider;

#[cfg(not(windows))]
const DEFAULT_SHELL: [&'static str; 2] = ["/bin/sh", "-c"];
#[cfg(windows)]
const DEFAULT_SHELL: [&'static str; 1] = ["yeah...we don't currently support windows :("];

pub struct Command<H: Host> {
    host: H,
    inner: Box<CommandProvider<H>>,
    shell: Vec<String>,
    cmd: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl<H: Host + 'static> Command<H> {
    pub fn new(host: &H, cmd: &str, shell: Option<&[&str]>) -> Box<Future<Item = Command<H>, Error = Error>> {
        let host = host.clone();
        let cmd = cmd.to_owned();
        let shell: Vec<String> = shell.unwrap_or(&DEFAULT_SHELL)
            .to_owned()
            .iter()
            .map(|s| s.to_string())
            .collect();

        Box::new(host.command_provider()
            .and_then(|provider| future::ok(Command {
                host: host,
                inner: provider,
                shell: shell,
                cmd: cmd,
            })))
    }

    pub fn with_provider<P>(host: &H, provider: P, cmd: &str, shell: Option<&[&str]>) -> Command<H>
        where P: CommandProvider<H> + 'static
    {
        Command {
            host: host.clone(),
            inner: Box::new(provider),
            shell: shell.unwrap_or(&DEFAULT_SHELL)
                        .to_owned()
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
            cmd: cmd.into(),
        }
    }

    pub fn exec(&mut self) -> Box<Future<Item = CommandResult, Error = Error>> {
        self.inner.exec(&self.host, &self.cmd, &self.shell)
    }
}
