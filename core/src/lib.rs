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
    pub use command::{self, Command};
    pub use command::providers::CommandProvider;
    pub use host::{Host, HostType};
    pub use host::remote::{self, Plain};
    pub use host::local::{self, Local};
    pub use provider::Provider;
    pub use telemetry::{self, Cpu, FsMount, Os, OsFamily, OsPlatform, Telemetry};
}
mod provider;
#[doc(hidden)] pub mod remote;
mod target;
pub mod telemetry;
