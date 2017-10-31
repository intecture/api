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
use futures::Future;
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

pub type ExecResult = Box<Future<Item = (
    Box<Stream<Item = String, Error = Error>>,
    Box<Future<Item = ExitStatus, Error = Error>>
), Error = Error>>;

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
///let result = cmd.exec().and_then(|(stream, status)| {
///    // Print the command's stdout/stderr to stdout
///    stream.for_each(|line| { println!("{}", line); Ok(()) })
///        // When it's ready, also print the exit status
///        .join(status.map(|s| println!("This command {} {}",
///            if s.success { "succeeded" } else { "failed" },
///            if let Some(e) = s.code { format!("with code {}", e) } else { String::new() })))
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
///```
///extern crate futures;
///extern crate intecture_api;
///extern crate tokio_core;
///
///use futures::{future, Future, Stream};
///use intecture_api::errors::Error;
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
///let result = cmd.exec().and_then(|(stream, _)| {
///    // Concatenate the buffer into a `String`
///    stream.fold(String::new(), |mut acc, line| {
///        acc.push_str(&line);
///        future::ok::<_, Error>(acc)
///    })
///        .map(|_output| {
///            // The binding `output` is our accumulated buffer
///        })
///});
///
///core.run(result).unwrap();
///# }
///```
///
/// Finally, we can also discard the command's output altogether if we only
/// care whether the command succeeded or not.
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
///let result = cmd.exec().and_then(|(stream, status)| {
///    // Discard the buffer
///    stream.for_each(|_| Ok(()))
///        .join(status.map(|_status| {
///            // Enjoy the status, baby...
///        }))
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
    pub fn exec(&self) -> ExecResult {
        let request = Request::CommandExec(self.provider.as_ref().map(|p| p.name()), self.cmd.clone());
        Box::new(self.host.request(request)
            .chain_err(|| ErrorKind::Request { endpoint: "Command", func: "exec" })
            .map(|msg| {
                parse_body_stream(msg)
            }))
    }
}

// Abstract this logic so other endpoints can parse CommandProvider::exec()
// streams too.
#[doc(hidden)]
pub fn parse_body_stream(mut msg: Message<Response, Body<Vec<u8>, io::Error>>) ->
    (
        Box<Stream<Item = String, Error = Error>>,
        Box<Future<Item = ExitStatus, Error = Error>>
    )
{
    let (tx, rx) = oneshot::channel::<ExitStatus>();
    let mut tx_share = Some(tx);
    let mut found = false;
    (
        Box::new(msg.take_body()
            .expect("Command::exec reply missing body stream")
            .filter_map(move |v| {
                let s = String::from_utf8_lossy(&v).to_string();

                // @todo This is a heuristical approach which is fallible
                if !found && s.starts_with("ExitStatus:") {
                    let (_, json) = s.split_at(11);
                    match serde_json::from_str(json) {
                        Ok(status) => {
                            // @todo What should happen if this fails?
                            let _ = tx_share.take().unwrap().send(status);
                            found = true;
                            return None;
                        },
                        _ => (),
                    }
                }

                Some(s)
            })
            .then(|r| r.chain_err(|| "Command execution failed"))
        ) as Box<Stream<Item = String, Error = Error>>,
        Box::new(rx.chain_err(|| "Buffer dropped before ExitStatus was sent"))
            as Box<Future<Item = ExitStatus, Error = Error>>
    )
}
