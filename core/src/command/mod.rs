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
use futures::{future, Future};
use futures::stream::Stream;
use host::Host;
use self::providers::CommandProvider;
use tokio_core::reactor::Handle;

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
///let host = Local::new().wait().unwrap();
///
///let cmd = Command::new(&host, "ls /path/to/foo", None)
///    // Remember that `Command::new()` returns a `Future`, so we need
///    // to wait for it to resolve before using it.
///    .and_then(|cmd| {
///        // Now that we have our `Command` instance, let's run it.
///        cmd.exec(&handle).and_then(|(stream, status)| {
///            // Print the command's stdout/stderr to stdout
///            stream.for_each(|line| { println!("{}", line); Ok(()) })
///                // When it's ready, also print the exit status
///                .join(status.map(|s| println!("This command {} {}",
///                    if s.success { "succeeded" } else { "failed" },
///                    if let Some(e) = s.code { format!("with code {}", e) } else { String::new() })))
///        })
///    });
///
/// core.run(cmd).unwrap();
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
///let host = Local::new().wait().unwrap();
///
///let cmd = Command::new(&host, "ls /path/to/foo", None).and_then(|cmd| {
///    cmd.exec(&handle).and_then(|(stream, _)| {
///        // Concatenate the buffer into a `String`
///        stream.fold(String::new(), |mut acc, line| {
///                acc.push_str(&line);
///                future::ok::<_, Error>(acc)
///            })
///            .map(|_output| {
///                // The binding `output` is our accumulated buffer
///            })
///    })
///});
///
/// core.run(cmd).unwrap();
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
///let host = Local::new().wait().unwrap();
///
///let cmd = Command::new(&host, "ls /path/to/foo", None).and_then(|cmd| {
///    cmd.exec(&handle).and_then(|(stream, status)| {
///        // Concatenate the buffer into a `String`
///        stream.for_each(|_| Ok(()))
///            .join(status.map(|_status| {
///                // Enjoy the status, baby...
///            }))
///    })
///});
///
/// core.run(cmd).unwrap();
///# }
///```
pub struct Command<H: Host> {
    host: H,
    inner: Box<CommandProvider<H>>,
    shell: Vec<String>,
    cmd: String,
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
    /// Create a new `Command` with the default `Provider`.
    ///
    /// By default, `Command` will use `/bin/sh -c` as the shell. You can
    /// override this by providing a value for `shell`. Note that the
    /// underlying implementation of `Command` escapes whitespace, so each
    /// argument needs to be a separate item in the slice. For example, to use
    /// Bash as your shell, you'd provide the value:
    /// `Some(&["/bin/bash", "-c"])`.
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

    /// Create a new `Command` with the specified `Provider`.
    ///
    ///## Example
    ///```
    ///extern crate futures;
    ///extern crate intecture_api;
    ///
    ///use futures::Future;
    ///use intecture_api::command::providers::Generic;
    ///use intecture_api::prelude::*;
    ///
    ///# fn main() {
    ///let host = Local::new().wait().unwrap();
    ///
    ///Generic::try_new(&host).map(|generic| {
    ///    Command::with_provider(&host, generic.unwrap(), "ls /path/to/foo", None)
    ///});
    ///# }
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
    pub fn exec(&self, handle: &Handle) ->
        Box<Future<Item = (
            Box<Stream<Item = String, Error = Error>>,
            Box<Future<Item = ExitStatus, Error = Error>>
        ), Error = Error>>
    {
        self.inner.exec(&self.host, handle, &self.cmd, &self.shell)
    }
}
