// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The host wrapper for communicating with a managed host.
//!
#![cfg_attr(feature = "local-run", doc = "# Local run")]
#![cfg_attr(feature = "local-run", doc = "When the `local-run` feature is enabled, this module exists to")]
#![cfg_attr(feature = "local-run", doc = "provide consistent function signatures with the `remote-run`")]
#![cfg_attr(feature = "local-run", doc = "feature. This means that projects implementing the API aren't")]
#![cfg_attr(feature = "local-run", doc = "required to modify their code depending on the feature that is")]
#![cfg_attr(feature = "local-run", doc = "enabled. The only exception to this is the Host::connect() method")]
#![cfg_attr(feature = "local-run", doc = "which is only present for the `remote-run` feature.")]
//!
//! # Examples
//!
//! ```no_run
//! # use inapi::{Command, Host};
//! let mut host = Host::new();
#![cfg_attr(feature = "local-run", doc = "// host.connect(...) <-- we don't need this")]
#![cfg_attr(feature = "remote-run", doc = "host.connect(\"myhost.example.com\", 7101, 7102).unwrap();")]
//!
//! let cmd = Command::new("whoami");
//! let result = cmd.exec(&mut host).unwrap();
//! ```

pub mod ffi;
#[cfg(feature = "local-run")]
pub mod local;
#[cfg(feature = "remote-run")]
pub mod remote;

#[cfg(feature = "remote-run")]
use czmq::ZSock;
#[cfg(feature = "local-run")]
pub use self::local::*;
#[cfg(feature = "remote-run")]
pub use self::remote::*;

/// Representation of a managed host.
#[cfg(feature = "local-run")]
pub struct Host;
#[cfg(feature = "remote-run")]
pub struct Host {
    /// Hostname or IP of managed host
    hostname: Option<String>,
    /// API socket
    api_sock: Option<ZSock>,
    /// File transfer socket
    file_sock: Option<ZSock>,
}
