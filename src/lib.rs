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
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
extern crate libc;
extern crate regex;
extern crate rustc_serialize;
#[cfg(test)]
extern crate tempdir;
#[cfg(feature = "remote-run")]
extern crate zfilexfer;

pub mod command;
pub mod directory;
pub mod error;
mod ffi_helpers;
pub mod file;
pub mod host;
pub mod package;
pub mod service;
mod target;
pub mod telemetry;

pub use command::{Command, CommandResult};
pub use directory::{Directory, DirectoryOpts};
pub use error::Error;
pub use file::{File, FileOwner};
pub use host::Host;
pub use package::{Package, PackageResult};
pub use package::providers::{Provider, ProviderFactory, Providers};
pub use service::{Service, ServiceRunnable};
pub use telemetry::{Cpu, FsMount, Netif, NetifStatus, NetifIPv4, NetifIPv6, Os, Telemetry};
pub use zfilexfer::FileOptions;

use std::result;
pub type Result<T> = result::Result<T, Error>;

#[cfg(all(test, feature = "remote-run"))]
use czmq::ZCert;
#[cfg(all(test, feature = "remote-run"))]
use std::env::set_current_dir;
#[cfg(all(test, feature = "remote-run"))]
use std::fs::create_dir;
#[cfg(all(test, feature = "remote-run"))]
use tempdir::TempDir;
#[cfg(all(test, feature = "remote-run"))]
use std::sync::{Once, ONCE_INIT};

#[cfg(all(test, feature = "remote-run"))]
static INIT_FS: Once = ONCE_INIT;

#[cfg(all(test, feature = "remote-run"))]
lazy_static! {
    static ref INIT_FS_DIR: TempDir = TempDir::new("remote_host_connect").unwrap();
}

#[cfg(all(test, feature = "remote-run"))]
fn create_project_fs() {
    INIT_FS.call_once(|| {
        set_current_dir(INIT_FS_DIR.path()).unwrap();

        let cert = ZCert::new().unwrap();
        cert.save_secret("user.crt").unwrap();

        let cert = ZCert::new().unwrap();
        cert.save_public("auth.crt").unwrap();
        cert.save_secret(".auth_secret.crt").unwrap();

        create_dir(".hosts").unwrap();

        let cert = ZCert::new().unwrap();
        cert.save_secret(".hosts/localhost.crt").unwrap();
    });
}

#[cfg(all(test, feature = "remote-run"))]
fn mock_auth_server() -> (::std::thread::JoinHandle<()>, String) {
    let sock = ::czmq::ZSock::new(::czmq::ZSockType::REP);
    let cert = ZCert::load(".auth_secret.crt").unwrap();
    cert.apply(&sock);
    sock.set_curve_server(true);
    sock.set_zap_domain("mock_auth_server");
    let port = sock.bind("tcp://127.0.0.1:*[60000-]").unwrap();

    let handle = ::std::thread::spawn(move|| {
        sock.recv_str().unwrap().unwrap();

        let reply = ::czmq::ZMsg::new();
        reply.addstr("Ok").unwrap();
        reply.addstr("0000000000000000000000000000000000000000").unwrap();
        reply.send(&sock).unwrap();
    });

    (handle, format!("127.0.0.1:{}", port))
}
