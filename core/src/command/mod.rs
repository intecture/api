// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Endpoint for running shell commands.
//!
//! A shell command is represented by the `Command` struct, which is not
//! idempotent.

pub mod providers;

use errors::*;
use futures::{future, Future, Poll};
use futures::stream::Stream;
use futures::sync::oneshot;
use host::Host;
use remote::{Request, Response};
use std::io;
use self::providers::CommandProvider;
use serde_json;
use tokio_proto::streaming::{Body, Message};

#[cfg(not(windows))]
const DEFAULT_SHELL: [&'static str; 2] = ["/bin/sh", "-c"];
#[cfg(windows)]
const DEFAULT_SHELL: [&'static str; 1] = ["yeah...we don't currently support windows :("];

/// Represents a shell command to be executed on a host.
///
///## Examples
///
/// Here's an example `ls` command that lists the directory `foo/`.
///
///```
///extern crate futures;
///extern crate intecture_api;
///extern crate tokio_core;
///
///use futures::{Future, Stream};
///use intecture_api::prelude::*;
///use tokio_core::reactor::Core;
///
///# fn main() {
///let mut core = Core::new().unwrap();
///let handle = core.handle();
///
///let host = Local::new(&handle).wait().unwrap();
///
///let cmd = Command::new(&host, "ls /path/to/foo", None);
///let result = cmd.exec().and_then(|mut status| {
///    // Print the command's stdout/stderr to stdout
///    status.take_stream().unwrap()
///        .for_each(|line| { println!("{}", line); Ok(()) })
///        // On its own, the stream will not do anything, so we need to make
///        // sure it gets returned along with the status future. `join()` will
///        // mash the two together so we can return them as one.
///        .join(status.map(|s| println!("This command {} {}",
///            if s.success { "succeeded" } else { "failed" },
///            if let Some(e) = s.code {
///                format!("with code {}", e)
///            } else {
///                String::new()
///            })))
///});
///
///core.run(result).unwrap();
///# }
///```
///
/// We can also save all output to a string for later use. **Be careful** doing
/// this as you could run out of memory on your heap if the output buffer is
/// too big.
///
///```no_run
///extern crate futures;
///extern crate intecture_api;
///extern crate tokio_core;
///
///use futures::Future;
///use intecture_api::errors::*;
///use intecture_api::prelude::*;
///use tokio_core::reactor::Core;
///
///# fn main() {
///let mut core = Core::new().unwrap();
///let handle = core.handle();
///
///let host = Local::new(&handle).wait().unwrap();
///
///let cmd = Command::new(&host, "ls /path/to/foo", None);
///let result = cmd.exec().and_then(|status| {
///    status.result().unwrap()
///        .map(|_output| {
///            // Our command finished successfully. Now we can do something
///            // with our output here.
///        })
///        .map_err(|e| {
///            // Our command errored out. Let's grab the output and see what
///            // went wrong.
///            match *e.kind() {
///                ErrorKind::Command(ref output) => println!("Oh noes! {}", output),
///                _ => unreachable!(),
///            }
///            e
///        })
///});
///
///core.run(result).unwrap();
///# }
///```
///
/// Finally, we can also ignore the stream entirely if we only care whether the
/// command succeeded or not.
///
///```
///extern crate futures;
///extern crate intecture_api;
///extern crate tokio_core;
///
///use futures::{Future, Stream};
///use intecture_api::prelude::*;
///use tokio_core::reactor::Core;
///
///# fn main() {
///let mut core = Core::new().unwrap();
///let handle = core.handle();
///
///let host = Local::new(&handle).wait().unwrap();
///
///let cmd = Command::new(&host, "ls /path/to/foo", None);
///let result = cmd.exec().and_then(|mut status| {
///    status.map(|exit_status| {
///        if exit_status.success {
///            println!("Huzzah!");
///        } else {
///            println!("Doh!");
///        }
///    })
///});
///
///core.run(result).unwrap();
///# }
///```
pub struct Command<H: Host> {
    host: H,
    provider: Option<Box<CommandProvider>>,
    cmd: Vec<String>,
}

/// Represents the status of a running `Command`, including the output stream
/// and exit status.
pub struct CommandStatus {
    stream: Option<Box<Stream<Item = String, Error = Error>>>,
    exit_status: Option<Box<Future<Item = ExitStatus, Error = Error>>>,
}

/// Represents the exit status of a `Command` as a `Result`-like `Future`. If
/// the command succeeded, the command output is returned. If it failed, an
/// error containing the command's output is returned.
pub struct CommandResult {
    inner: Box<Future<Item = String, Error = Error>>,
}

/// The status of a finished command.
///
/// This is a serializable replica of
/// [`std::process::ExitStatus`](https://doc.rust-lang.org/std/process/struct.ExitStatus.html).
#[derive(Debug, Serialize, Deserialize)]
pub struct ExitStatus {
    /// Was termination successful? Signal termination is not considered a
    /// success, and success is defined as a zero exit status.
    pub success: bool,
    /// Returns the exit code of the process, if any.
    ///
    /// On Unix, this will return `None` if the process was terminated by a
    /// signal.
    pub code: Option<i32>,
}

impl<H: Host + 'static> Command<H> {
    /// Create a new `Command` with the default `CommandProvider`.
    ///
    /// By default, `Command` will use `/bin/sh -c` as the shell. You can
    /// override this by providing a value for `shell`. Note that the
    /// underlying implementation of `Command` escapes whitespace, so each
    /// argument needs to be a separate item in the slice. For example, to use
    /// Bash as your shell, you'd provide the value:
    /// `Some(&["/bin/bash", "-c"])`.
    pub fn new(host: &H, cmd: &str, shell: Option<&[&str]>) -> Command<H> {
        let mut args: Vec<String> = shell.unwrap_or(&DEFAULT_SHELL).to_owned()
            .iter().map(|a| (*a).to_owned()).collect();
        args.push(cmd.into());

        Command {
            host: host.clone(),
            provider: None,
            cmd: args,
        }
    }

    /// Create a new `Command` with the specified `CommandProvider`.
    ///
    ///## Example
    ///```
    ///extern crate futures;
    ///extern crate intecture_api;
    ///extern crate tokio_core;
    ///
    ///use futures::Future;
    ///use intecture_api::command::providers::Generic;
    ///use intecture_api::prelude::*;
    ///use tokio_core::reactor::Core;
    ///
    ///# fn main() {
    ///let mut core = Core::new().unwrap();
    ///let handle = core.handle();
    ///
    ///let host = Local::new(&handle).wait().unwrap();
    ///
    ///Command::with_provider(&host, Generic, "ls /path/to/foo", None);
    ///# }
    pub fn with_provider<P>(host: &H, provider: P, cmd: &str, shell: Option<&[&str]>) -> Command<H>
        where P: CommandProvider + 'static
    {
        let mut cmd = Self::new(host, cmd, shell);
        cmd.provider = Some(Box::new(provider));
        cmd
    }

    /// Execute the command.
    ///
    ///## Returns
    ///
    /// This function returns a `Future` that represents the delay between
    /// now and the time it takes to start execution. This `Future` yields a
    /// tuple with a `Stream` and a `Future` inside. The `Stream` is the
    /// command's output stream, including both stdout and stderr. The `Future`
    /// yields the command's `ExitStatus`.
    ///
    /// **WARNING!** For remote `Host` types, you _MUST_ consume the output
    /// `Stream` if you want to access the `ExitStatus`. This is due to the
    /// plumbing between the API and the remote host, which relies on a single
    /// streaming pipe. First we stream the command output, then tack the
    /// `ExitStatus` on as the last frame. Without consuming the output buffer,
    /// we would never be able to get to the last frame, and `ExitStatus` could
    /// never be resolved.
    ///
    ///# Errors
    ///
    ///>Error: Buffer dropped before ExitStatus was sent
    ///
    ///>Caused by: oneshot canceled
    ///
    /// This is the error you'll see if you prematurely drop the output `Stream`
    /// while trying to resolve the `Future<Item = ExitStatus, ...>`.
    pub fn exec(&self) -> Box<Future<Item = CommandStatus, Error = Error>> {
        let request = Request::CommandExec(self.provider.as_ref().map(|p| p.name()), self.cmd.clone());
        Box::new(self.host.request(request)
            .chain_err(|| ErrorKind::Request { endpoint: "Command", func: "exec" })
            .map(|msg| {
                CommandStatus::new(msg)
            }))
    }
}

impl CommandStatus {
    #[doc(hidden)]
    pub fn new(mut msg: Message<Response, Body<Vec<u8>, io::Error>>) -> CommandStatus {
        let (tx, rx) = oneshot::channel::<ExitStatus>();
        let mut tx = Some(tx);
        let stream = msg.take_body()
            .expect("Command::exec reply missing body stream")
            .filter_map(move |v| {
                let s = String::from_utf8_lossy(&v).to_string();

                // @todo This is a heuristical approach which is fallible
                if s.starts_with("ExitStatus:") {
                    let (_, json) = s.split_at(11);
                    match serde_json::from_str(json) {
                        Ok(status) => {
                            // @todo What should happen if this fails?
                            let _ = tx.take().unwrap().send(status);
                            return None;
                        },
                        _ => (),
                    }
                }

                Some(s)
            })
            .then(|r| r.chain_err(|| "Command execution failed"));

        let exit_status = rx.chain_err(|| "Buffer dropped before ExitStatus was sent");

        CommandStatus {
            stream: Some(Box::new(stream)),
            exit_status: Some(Box::new(exit_status)),
        }
    }

    /// Take ownership of the output stream.
    ///
    /// The stream is guaranteed to be present only if this is the first call
    /// to `take_stream()` and the future has not yet been polled.
    pub fn take_stream(&mut self) -> Option<Box<Stream<Item = String, Error = Error>>> {
        self.stream.take()
    }

    /// Convert this to a `CommandResult`, which returns the output string on
    /// success and an error containing the command's output on failure. If the
    /// stream has already been taken by `take_stream()` then this function
    /// will return `None`.
    ///
    /// Note that "success" is determined by examining the `ExitStatus::success`
    /// bool. See `ExitStatus` docs for details.
    pub fn result(self) -> Option<CommandResult> {
        if let Some(stream) = self.stream {
            let inner = stream.fold(String::new(), |mut acc, line| {
                    acc.push_str(&line);
                    future::ok::<_, Error>(acc)
                })
                .join(self.exit_status.unwrap())
                .and_then(|(output, status)| if status.success {
                    future::ok(output)
                } else {
                    future::err(ErrorKind::Command(output).into())
                });

            Some(CommandResult {
                inner: Box::new(inner) as Box<Future<Item = String, Error = Error>>
            })
        } else {
            None
        }
    }
}

impl Future for CommandStatus {
    type Item = ExitStatus;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if let Some(stream) = self.stream.take() {
            self.exit_status = Some(Box::new(stream.for_each(|_| Ok(()))
                .join(self.exit_status.take().unwrap())
                .map(|(_, status)| status)));
        }

        self.exit_status.as_mut().unwrap().poll()
    }
}

impl Future for CommandResult {
    type Item = String;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}
