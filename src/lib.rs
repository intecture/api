// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
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

extern crate inprimitives;
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate rustc_serialize;
extern crate zmq;

pub mod command;
mod error;
pub mod host;
pub mod telemetry;

pub use command::{Command, CommandResult};
pub use error::{Error, Result};
pub use host::Host;
pub use telemetry::{Cpu, FsMount, Netif, NetifStatus, NetifIPv4, NetifIPv6, Os, Telemetry, TelemetryInit};
