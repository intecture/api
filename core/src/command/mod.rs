// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Endpoint for running shell commands.

pub mod providers;

use erased_serde::Serialize;
use errors::*;
use Executable;
use futures::{future, Future};
use host::Host;
use self::providers::{Nix, NixRunnable};
use std::sync::Arc;

pub trait CommandProvider<H: Host> {
    fn available(&Arc<H>) -> Box<Future<Item = bool, Error = Error>> where Self: Sized;
    fn try_new(&Arc<H>, &str, Option<&[&str]>) -> Box<Future<Item = Option<Self>, Error = Error>> where Self: Sized;
    fn exec(&mut self) -> Box<Future<Item = CommandResult, Error = Error>>;
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum CommandRunnable {
    Nix(NixRunnable)
}

#[doc(hidden)]
#[derive(Clone, Serialize, Deserialize)]
pub struct Command {
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

impl Executable for CommandRunnable {
    fn exec(self) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            CommandRunnable::Nix(p) => p.exec()
        }
    }
}

pub fn factory<H: Host + 'static>(host: &Arc<H>, cmd: &str, shell: Option<&[&str]>) -> Box<Future<Item = Box<CommandProvider<H>>, Error = Error>> {
    Box::new(Nix::try_new(host, cmd, shell)
                     .and_then(|opt| match opt {
                         Some(provider) => future::ok(Box::new(provider) as Box<CommandProvider<H>>),
                         None => future::err(ErrorKind::ProviderUnavailable("Command").into())
                     }))
}
