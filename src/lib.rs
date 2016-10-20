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
mod project;
pub mod directory;
pub mod error;
pub mod file;
#[macro_use]
mod host;
#[cfg(all(test, feature = "remote-run"))]
mod mock_env;
pub mod package;
#[cfg(feature = "remote-run")]
mod payload;
pub mod service;
mod target;
pub mod template;

pub use command::{Command, CommandResult};
pub use directory::{Directory, DirectoryOpts};
pub use error::Error;
pub use file::{File, FileOwner};
pub use host::Host;
pub use host::data::open as data_open;
pub use host::ffi::{get_value, get_value_keys, get_value_type};
#[cfg(feature = "local-run")]
pub use host::ffi::host_local;
#[cfg(feature = "remote-run")]
pub use host::ffi::{host_connect, host_connect_endpoint, host_connect_payload, host_close};
pub use mustache::{MapBuilder, VecBuilder};
pub use package::Package;
pub use package::providers::{Provider, ProviderFactory, Providers};
#[cfg(feature = "remote-run")]
pub use payload::Payload;
#[cfg(feature = "remote-run")]
pub use payload::config::Config as PayloadConfig;
#[cfg(feature = "remote-run")]
pub use payload::ffi::{payload_new, payload_build, payload_run};
pub use project::{Language, ProjectConfig};
pub use serde_json::Value;
pub use service::{Service, ServiceRunnable};
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
    static ref PROJECT_CONFIG: project::ProjectConfig = project::ProjectConfig::load("project.json")
                                                                               .expect("Could not load project.json");
}
