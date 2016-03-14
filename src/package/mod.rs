// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The primitive for installing and managing software packages on a
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
#![cfg_attr(feature = "remote-run", doc = "host.connect(\"127.0.0.1\", 7101, 7102, 7103).unwrap();")]
//! ```
//!
//! Now install the package `nginx` using the default provider:
//!
//! ```no_run
//! # use inapi::{Host, Package};
//! # let mut host = Host::new();
//! let mut package = Package::new(&mut host, "nginx", None).unwrap();
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
//! let mut package = Package::new(&mut host, "nginx", Some(Providers::Homebrew)).unwrap();
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

        Ok(Package {
            name: name.to_string(),
            provider: provider,
            installed: installed,
        })
    }

    /// Check if the package is installed.
    pub fn is_installed(&self) -> bool {
        self.installed
    }

    /// Install the package.
    pub fn install(&mut self, host: &mut Host) -> Result<PackageResult> {
        if self.installed {
            Ok(PackageResult::NoAction)
        } else {
            let result = try!(self.provider.install(host, &self.name));

            if result.exit_code == 0 {
                self.installed = true;
            }

            Ok(PackageResult::Result(result))
        }
    }

    /// Uninstall the package.
    pub fn uninstall(&mut self, host: &mut Host) -> Result<PackageResult> {
        if self.installed {
            let result = try!(self.provider.uninstall(host, &self.name));

            if result.exit_code == 0 {
                self.installed = false;
            }

            Ok(PackageResult::Result(result))
        } else {
            Ok(PackageResult::NoAction)
        }
    }
}

/// Result of package operation.
#[derive(Debug)]
pub enum PackageResult {
    /// The command result from a package operation
    /// (e.g. installing/uninstalling)
    Result(CommandResult),
    /// No action was necessary to achieve the desired state
    /// (e.g. calling install() on a currently installed package)
    NoAction,
}

pub trait PackageTarget {
    fn default_provider(host: &mut Host) -> Result<Providers>;
}

#[cfg(test)]
mod tests {
    use Host;
    use super::*;
    #[cfg(feature = "remote-run")]
    use super::providers::Providers;
    #[cfg(feature = "remote-run")]
    use std::thread;
    #[cfg(feature = "remote-run")]
    use zmq;

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_homebrew() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_new_homebrew").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("which brew", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("/usr/local/bin/brew", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();

            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("brew list | grep nginx", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test_new_homebrew").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let pkg = Package::new(&mut host, "nginx", Some(Providers::Homebrew)).unwrap();

        assert_eq!(pkg.name, "nginx");
        assert!(!pkg.is_installed());

        agent_mock.join().unwrap();
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

            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("which brew", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("/usr/local/bin/brew", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();

            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("brew list | grep nginx", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_new_default").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let pkg = Package::new(&mut host, "nginx", None);
        assert!(pkg.is_ok());

        agent_mock.join().unwrap();
    }
}
