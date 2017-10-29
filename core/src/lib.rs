// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Intecture is an API for managing your servers. You can think of it as a
//! DevOps tool, but without the complicated ecosystem and proprietary nonsense.
//!
//! The core API is, well, the core of Intecture. It contains all the endpoints
//! used to configure a host, as well as the underlying OS abstractions that
//! they are built on. Generally you'll consume this API via
//! [intecture_proj](../intecture_proj/), which reexports `intecture_api`,
//! though for projects that do not need a formal structure (e.g. an installer
//! program), this API will suffice.
//!
//!## Project structure
//!
//! The core API is organised into a series of modules (known as “endpoints”,
//! e.g. `command`, `package` etc.), which represent basic configuration tasks
//! that you’d normally perform by hand. Within each endpoint is a `providers`
//! module, which houses the OS-specific abstractions that do the heavy lifting
//! on behalf of the endpoint.
//!
//! For example, the [`package`](package/) endpoint has a struct called
//! `Package`. This is a cross-platform abstraction for managing a package on
//! your server. Behind this abstraction is a concrete implementation of a
//! specific package [_provider_](package/providers), e.g. Yum or Apt. If you
//! instantiate a new `Package` instance through the
//! [`Package::new()`](package/struct.Package.html#method.new) function, the
//! best available provider for your server is chosen automatically. This is
//! true of all endpoints.
//!
//!## Hosts
//!
//! So far we’ve talked about using endpoints to automate configuration tasks,
//! but how does Intecture know which server we want to talk to? This is where
//! we need the [`host`](host/) endpoint. All things start with a host! Side
//! note - if we were ever to do ‘merch’, that’d probably be on a t-shirt.
//! Anyway, poor marketing decisions aside, you’ll need to create a host in
//! order to do anything.
//!
//! Hosts come in both the [`Local`](host/local/struct.Local.html) and
//! [`Plain`](host/remote/struct.Plain.html) varieties. The `Local` type points
//! to your local machine, and the `Plain` type is a remote host type that
//! connects to a remote machine over the network. Whichever type you choose,
//! simply pass it in to your endpoints as required and Intecture will do the
//! rest.
//!
//!>“Why `Plain`?” I hear you ask. Well, it’s because the `Plain` host type is
//! a remote host that uses TCP to send/receive _plaintext_ data.
//!
//!## Example
//!
//! Here’s a reproduction of the
//! [basic example](https://github.com/intecture/api/blob/master/core/examples/basic.rs)
//! from the `examples/` folder:
//!
//!```rust
//!extern crate futures;
//!extern crate intecture_api;
//!extern crate tokio_core;
//!
//!use futures::{Future, Stream};
//!use intecture_api::prelude::*;
//!use tokio_core::reactor::Core;
//!
//!fn main() {
//!    // These two lines are part of `tokio-core` and can be safely
//!    // ignored. So long as they appear at the top of your code,
//!    // all is fine with the world.
//!    let mut core = Core::new().unwrap();
//!    let handle = core.handle();
//!
//!    // Here's the meat of your project. In this example we're talking
//!    // to our local machine, so we use the `Local` host type.
//!    let host = Local::new(&handle).and_then(|host| {
//!        // Ok, we're in! Now we can pass our `host` handle to other
//!        // endpoints, which informs them of the server we mean to
//!        // talk to.
//!
//!        // Let's start with something basic - a shell command.
//!        let cmd = Command::new(&host, "whoami", None);
//!        cmd.exec().and_then(|(stream, status)| {
//!            // At this point, our command is running. As the API
//!            // is asynchronous, we don't have to wait for it to
//!            // finish before inspecting its output. This is called
//!            // "streaming".
//!
//!            // Our first argument, `stream`, is a stream of strings,
//!            // each of which represents a line of output. We can use
//!            // the `for_each` combinator to print these lines to
//!            // stdout.
//!            //
//!            // If printing isn't your thing, you are also
//!            // free to lick them or whatever you're into. I'm not
//!            // here to judge.
//!            stream.for_each(|line| { println!("{}", line); Ok(()) })
//!
//!            // The second argument is a `Future` that represents the
//!            // command's exit status. Let's print that too*.
//!            //
//!            // * Same caveat as above RE: printing. This is a safe
//!            //   place.
//!                .join(status.map(|s| println!("This command {} {}",
//!                    if s.success { "succeeded" } else { "failed" },
//!                    if let Some(e) = s.code { format!("with code {}", e) } else { String::new() })))
//!        })
//!    });
//!
//!    // This line is part of `tokio-core` and is used to execute the
//!    // chain of futures you've created above. You'll need to call
//!    // `core.run()` for each host you interact with, otherwise your
//!    // project will not run at all!
//!    core.run(host).unwrap();
//!}
//!```

#![recursion_limit = "1024"]

extern crate bytes;
extern crate erased_serde;
#[macro_use] extern crate error_chain;
extern crate futures;
extern crate hostname;
extern crate ipnetwork;
#[macro_use] extern crate log;
extern crate pnet;
extern crate regex;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_process;
extern crate tokio_proto;
extern crate tokio_service;

pub mod command;
pub mod errors;
pub mod host;
pub mod prelude {
    //! The API prelude.
    pub use command::{self, Command};
    pub use host::Host;
    pub use host::remote::{self, Plain};
    pub use host::local::{self, Local};
    pub use telemetry::{self, Cpu, FsMount, Os, OsFamily, OsPlatform, Telemetry};
}
// pub mod package;
mod provider;
#[doc(hidden)] pub mod remote;
mod target;
pub mod telemetry;
