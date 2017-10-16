// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::CommandResult;
use erased_serde::Serialize;
use errors::*;
use futures::{future, Future};
use host::{Host, HostType};
use host::local::Local;
use host::remote::Plain;
use provider::Provider;
use remote::{Executable, Runnable};
use std::process;
use super::{CommandProvider, CommandRunnable};

#[derive(Clone)]
pub struct Generic<H: Host> {
    host: H
}

struct LocalGeneric;
struct RemoteGeneric;

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum GenericRunnable {
    Available,
    Exec(String, Vec<String>),
}

impl<H: Host + 'static> Provider<H> for Generic<H> {
    fn available(host: &H) -> Box<Future<Item = bool, Error = Error>> {
        match host.get_type() {
            HostType::Local(l) => LocalGeneric::available(l),
            HostType::Remote(r) => RemoteGeneric::available(r),
        }
    }

    fn try_new(host: &H) -> Box<Future<Item = Option<Generic<H>>, Error = Error>> {
        let host = host.clone();
        Box::new(Self::available(&host)
            .and_then(|available| {
                if available {
                    future::ok(Some(Generic { host }))
                } else {
                    future::ok(None)
                }
            }))
    }
}

impl<H: Host + 'static> CommandProvider<H> for Generic<H> {
    fn exec(&mut self, cmd: &str, shell: &[String]) -> Box<Future<Item = CommandResult, Error = Error>> {
        match self.host.get_type() {
            HostType::Local(l) => LocalGeneric::exec(l, cmd, shell),
            HostType::Remote(r) => RemoteGeneric::exec(r, cmd, shell),
        }
    }
}

impl LocalGeneric {
    fn available(_: &Local) -> Box<Future<Item = bool, Error = Error>> {
        Box::new(future::ok(cfg!(unix)))
    }

    fn exec(_: &Local, cmd: &str, shell: &[String]) -> Box<Future<Item = CommandResult, Error = Error>> {
        let cmd_owned = cmd.to_owned();
        let shell_owned = shell.to_owned();

        Box::new(future::lazy(move || -> future::FutureResult<CommandResult, Error> {
            let (shell, shell_args) = match shell_owned.split_first() {
                Some((s, a)) => (s, a),
                None => return future::err("Invalid shell provided".into()),
            };

            let out = process::Command::new(shell)
                                       .args(shell_args)
                                       .arg(&cmd_owned)
                                       .output()
                                       .chain_err(|| "Command execution failed");
            match out {
                Ok(output) => future::ok(CommandResult {
                    success: output.status.success(),
                    exit_code: output.status.code(),
                    stdout: output.stdout,
                    stderr: output.stderr,
                }),
                Err(e) => future::err(e),
            }
        }))
    }
}

impl RemoteGeneric {
    fn available(host: &Plain) -> Box<Future<Item = bool, Error = Error>> {
        let runnable = Runnable::Command(
                          CommandRunnable::Generic(
                              GenericRunnable::Available));
        host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Command::Generic", func: "available" })
    }

    fn exec(host: &Plain, cmd: &str, shell: &[String]) -> Box<Future<Item = CommandResult, Error = Error>> {
        let runnable = Runnable::Command(
                          CommandRunnable::Generic(
                              GenericRunnable::Exec(cmd.into(), shell.to_owned())));
        host.run(runnable)
            .chain_err(|| ErrorKind::Runnable { endpoint: "Command::Generic", func: "exec" })
    }
}

impl Executable for GenericRunnable {
    fn exec(self, host: &Local) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            GenericRunnable::Available => Box::new(LocalGeneric::available(host).map(|b| Box::new(b) as Box<Serialize>)),
            GenericRunnable::Exec(cmd, shell) => Box::new(LocalGeneric::exec(host, &cmd, &shell).map(|r| Box::new(r) as Box<Serialize>)),
        }
    }
}
