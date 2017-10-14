// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! A library for configuring your servers.
//!
//! The library is organised into a set of endpoints, which are
//! the building blocks for creating complex configurations.

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
extern crate tokio_proto;
extern crate tokio_service;

pub mod command;
pub mod errors;
pub mod host;
pub mod prelude {
    pub use command;
    pub use host::Host;
    pub use host::remote::{self, Plain, RemoteHost};
    pub use host::local::{self, Local};
    pub use telemetry::{self, Cpu, FsMount, Os, OsFamily, OsPlatform, Telemetry};
}
mod target;
pub mod telemetry;

use errors::*;
use erased_serde::Serialize;
use futures::Future;

#[doc(hidden)]
pub trait Executable {
    fn exec(self) -> Box<Future<Item = Box<Serialize>, Error = Error>>;
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum Runnable {
    Command(command::CommandRunnable),
    Telemetry(telemetry::TelemetryRunnable),
}

impl Executable for Runnable {
    fn exec(self) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            Runnable::Command(p) => p.exec(),
            Runnable::Telemetry(p) => p.exec(),
        }
    }
}
