// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The primitive for installing and manging software packages on a
//! managed host.
//!
//! # Examples
//!
//! Initialise a new Host using your managed host's IP address and
//! port number:
//!
//! ```no_run
//! # use inapi::Host;
//! let mut host = Host::new();
#![cfg_attr(feature = "remote-run", doc = "host.connect(\"127.0.0.1\", 7101).unwrap();")]
//! ```
//!
//! Now install the package `nginx` using the default provider:
//!
//! ```no_run
//! # use inapi::{Host, Package};
//! # let mut host = Host::new();
//! let package = Package::new(&mut host, "nginx", None).unwrap();
//! package.install(&mut host);
//! ```
//!
//! You can also specify a package provider manually, instead of
//! relying on Intecture to choose one for you. This is useful if you
//! have multiple providers on your managed host.
//!
//! ```no_run
//! # use inapi::{Host, Package, Providers};
//! # let mut host = Host::new();
//! let package = Package::new(&mut host, "nginx", Some(Providers::Homebrew)).unwrap();
//! package.install(&mut host);
//! ```

pub mod ffi;
pub mod providers;

use {CommandResult, Host, Result};
use self::providers::*;

/// Container for operating on a package.
pub struct Package {
    /// The name of the package, e.g. `nginx`
    name: String,
    /// The package source
    provider: Box<Provider + 'static>,
    /// Package installed bool
    installed: bool,
}

impl Package {
    /// Create a new Package.
    ///
    /// If you have multiple package providers, you can specify one
    /// or allow Intecture to select a default based on the OS.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::{Host, Package, Providers};
    /// # let mut host = Host::new();
    /// let pkg = Package::new(&mut host, "nginx", Some(Providers::Yum));
    /// ```
    pub fn new(host: &mut Host, name: &str, providers: Option<Providers>) -> Result<Package> {
        let provider = try!(ProviderFactory::create(host, providers));
        let installed = try!(provider.is_installed(host, name));

        let pkg = Package {
            name: name.to_string(),
            provider: provider,
            installed: installed,
        };

        Ok(pkg)
    }

    /// Check if the package is installed.
    pub fn is_installed(&self) -> bool {
        self.installed
    }

    /// Install the package.
    pub fn install(&self, host: &mut Host) -> Result<CommandResult> {
        self.provider.install(host, &self.name)
    }

    /// Uninstall the package.
    pub fn uninstall(&self, host: &mut Host) -> Result<CommandResult> {
        self.provider.uninstall(host, &self.name)
    }
}

pub trait PackageTarget {
    fn default_provider(host: &mut Host) -> Result<Box<Provider + 'static>>;
}

#[cfg(test)]
mod tests {
    use Host;
    use super::*;
    use super::providers::Providers;
    #[cfg(feature = "remote-run")]
    use std::thread;
    #[cfg(feature = "remote-run")]
    use zmq;

    #[test]
    fn test_new_homebrew() {
        let mut host = Host::new();
        let pkg = Package::new(&mut host, "nginx", Some(Providers::Homebrew));
        assert!(pkg.is_ok());
    }

    #[cfg(feature = "local-run")]
    #[test]
    fn test_new_default() {
        let mut host = Host::new();
        let pkg = Package::new(&mut host, "nginx", None);
        assert!(pkg.is_ok());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_default() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_new_default").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("package::default_provider", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);
            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Homebrew", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_new_default").unwrap();

        let mut host = Host::test_new(sock);

        let pkg = Package::new(&mut host, "nginx", None);
        assert!(pkg.is_ok());

        agent_mock.join().unwrap();
    }
}
