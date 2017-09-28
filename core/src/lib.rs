// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! A library for configuring your servers.
//!
//! The library is organised into a set of _primitives_, which are
//! the building blocks for creating complex configurations.

#![recursion_limit = "1024"]

extern crate erased_serde;
#[macro_use] extern crate error_chain;
extern crate hostname;
extern crate ipnetwork;
extern crate pnet;
extern crate regex;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

pub mod command;
pub mod errors;
pub mod host;
mod target;
pub mod telemetry;

use errors::*;
use erased_serde::Serialize;

pub trait ExecutableProvider<'de>: serde::Serialize + serde::Deserialize<'de> {
    // @todo It'd be nice to return Result<Serialize> here someday...
    // See https://github.com/rust-lang/rfcs/issues/518.
    fn exec(self, &host::Host) -> Result<Box<Serialize>>;
}

#[derive(Serialize, Deserialize)]
pub enum RemoteProvider {
    Command(command::RemoteProvider),
    Telemetry(telemetry::RemoteProvider),
}

impl <'de>ExecutableProvider<'de> for RemoteProvider {
    fn exec(self, host: &host::Host) -> Result<Box<Serialize>> {
        match self {
            RemoteProvider::Command(p) => p.exec(host),
            RemoteProvider::Telemetry(p) => p.exec(host),
        }
    }
}
