// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! # Intecture API
//!
//! The Intecture API is the interface between your code and your managed
//! hosts. The library is organised into a set of 'primitives', which
//! are the building blocks used to configure your systems.
//! Developers should implement these primitives in their source code
//! in order to configure and maintain their managed hosts.
//!
//! ## Communication
//!
//! The API communicates with the Agent service on your managed hosts
//! via a ZeroMQ "REQ" (REQuest) socket. No other socket types are
//! supported by the Intecture Agent.
//!
//! **Note:** The Agent service must be running when you call any
//! Intecture API primitives, or the program will hang while it
//! attempts to connect to a non-existent socket.

#[cfg(feature = "remote-run")]
extern crate czmq;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate mustache;
extern crate regex;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
#[cfg(test)]
extern crate tempdir;
extern crate tempfile;
extern crate zdaemon;
extern crate zfilexfer;

#[macro_use]
mod ffi_helpers;
pub mod command;
mod config;
#[macro_use]
mod data;
pub mod directory;
pub mod error;
pub mod file;
pub mod host;
#[cfg(all(test, feature = "remote-run"))]
mod mock_env;
pub mod package;
pub mod service;
mod target;
pub mod telemetry;
pub mod template;

pub use command::{Command, CommandResult};
pub use data::DataParser;
pub use data::ffi::{data_open, free_value, get_value, get_value_type};
pub use directory::{Directory, DirectoryOpts};
pub use error::Error;
pub use file::{File, FileOwner};
pub use host::Host;
pub use mustache::{MapBuilder, VecBuilder};
pub use package::Package;
pub use package::providers::{Provider, ProviderFactory, Providers};
pub use serde_json::Value;
pub use service::{Service, ServiceRunnable};
pub use telemetry::{Cpu, FsMount, Netif, NetifStatus, NetifIPv4, NetifIPv6, Os, Telemetry};
pub use template::Template;
pub use zfilexfer::FileOptions;

#[cfg(feature = "remote-run")]
use zdaemon::ConfigFile;

#[cfg(all(test, feature = "remote-run"))]
lazy_static! {
    static ref _MOCK_ENV: mock_env::MockEnv = mock_env::MockEnv::new();
}

#[cfg(feature = "remote-run")]
lazy_static! {
    static ref PROJECT_CONFIG: config::Config = config::Config::load("project.json").expect("Could not load project.json");
}
