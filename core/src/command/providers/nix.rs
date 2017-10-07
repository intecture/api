// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::{Command, CommandProvider, CommandResult, CommandRunnable};
use erased_serde::Serialize;
use errors::*;
use Executable;
use futures::{future, Future};
use host::Host;
use Runnable;
use std::process;
use std::sync::Arc;

const DEFAULT_SHELL: [&'static str; 2] = ["/bin/sh", "-c"];

pub struct Nix<H: Host> {
    host: Arc<H>,
    inner: Command
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum NixRunnable {
    Available,
    Exec(Command),
}

impl <H: Host + 'static>CommandProvider<H> for Nix<H> {
    fn available(host: &Arc<H>) -> Box<Future<Item = bool, Error = Error>> {
        host.run(Runnable::Command(CommandRunnable::Nix(NixRunnable::Available)))
            .chain_err(|| ErrorKind::Runnable { endpoint: "Command::Nix", func: "available" })
    }

    fn try_new(host: &Arc<H>, cmd: &str, shell: Option<&[&str]>) -> Box<Future<Item = Option<Nix<H>>, Error = Error>> {
        let cmd_owned = cmd.to_owned();
        let shell_owned: Vec<String> = shell.unwrap_or(&DEFAULT_SHELL).to_owned().iter().map(|s| s.to_string()).collect();
        let host = host.clone();

        Box::new(Self::available(&host).and_then(move |available| {
            if available {
                let inner = Command {
                    shell: shell_owned,
                    cmd: cmd_owned,
                };
                future::ok(Some(Nix { host, inner }))
            } else {
                future::ok(None)
            }
        }))
    }

    fn exec(&mut self) -> Box<Future<Item = CommandResult, Error = Error>> {
        self.host.run(Runnable::Command(CommandRunnable::Nix(NixRunnable::Exec(self.inner.clone()))))
                 .chain_err(|| ErrorKind::Runnable { endpoint: "Command::Nix", func: "exec" })
    }
}

impl Executable for NixRunnable {
    fn exec(self) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            NixRunnable::Available =>
                Box::new(future::ok(Box::new(
                    cfg!(unix)
                ) as Box<Serialize>)),
            NixRunnable::Exec(inner) => {
                Box::new(future::lazy(move || -> future::FutureResult<Box<Serialize>, Error> {
                    let (shell, shell_args) = match inner.shell.split_first() {
                        Some((s, a)) => (s, a),
                        None => return future::err("Invalid shell provided".into()),
                    };

                    let out = process::Command::new(shell)
                                               .args(shell_args)
                                               .arg(&inner.cmd)
                                               .output()
                                               .chain_err(|| "Command execution failed");
                    match out {
                        Ok(output) => future::ok(Box::new(CommandResult {
                            success: output.status.success(),
                            exit_code: output.status.code(),
                            stdout: output.stdout,
                            stderr: output.stderr,
                        }) as Box<Serialize>),
                        Err(e) => future::err(e),
                    }
                }))
            }
        }
    }
}
