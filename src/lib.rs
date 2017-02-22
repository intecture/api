// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
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
#[macro_use]
extern crate serde_json;
#[cfg(test)]
extern crate tempdir;
extern crate tempfile;
extern crate zdaemon;
extern crate zfilexfer;
extern crate hostname;

#[macro_use]
mod ffi_helpers;
mod command;
mod project;
mod directory;
mod error;
mod file;
#[macro_use]
mod host;
#[cfg(all(test, feature = "remote-run"))]
mod mock_env;
mod package;
#[cfg(feature = "remote-run")]
mod payload;
mod service;
mod target;
mod template;

pub use command::{Command, CommandResult, ffi as command_ffi};
pub use directory::{Directory, DirectoryOpts, ffi as directory_ffi};
pub use error::{Error, geterr};
pub use file::{File, FileOwner, ffi as file_ffi};
pub use host::{Host, ffi as host_ffi};
pub use host::data::open as data_open;
pub use mustache::{MapBuilder, VecBuilder};
pub use package::{Package, ffi as package_ffi};
pub use package::providers::Providers;
#[cfg(feature = "remote-run")]
pub use payload::{Payload, ffi as payload_ffi};
#[cfg(feature = "remote-run")]
#[doc(hidden)]
pub use payload::config::Config as PayloadConfig;
#[doc(hidden)]
pub use project::{Language, ProjectConfig};
pub use serde_json::Value;
pub use service::{Service, ServiceRunnable, ffi as service_ffi};
pub use template::{Template, ffi as template_ffi};
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
