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
#![cfg_attr(feature = "remote-run", doc = "host.connect(\"myhost.example.com\", 7101, 7102).unwrap();")]
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

use command::CommandResult;
use error::Result;
use host::Host;
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
    pub fn install(&mut self, host: &mut Host) -> Result<Option<CommandResult>> {
        if self.installed {
            Ok(None)
        } else {
            let result = try!(self.provider.install(host, &self.name));

            if result.exit_code == 0 {
                self.installed = true;
            }

            Ok(Some(result))
        }
    }

    /// Uninstall the package.
    pub fn uninstall(&mut self, host: &mut Host) -> Result<Option<CommandResult>> {
        if self.installed {
            let result = try!(self.provider.uninstall(host, &self.name));

            if result.exit_code == 0 {
                self.installed = false;
            }

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

pub trait PackageTarget {
    fn default_provider(host: &mut Host) -> Result<Providers>;
}

#[cfg(test)]
mod tests {
    use Host;
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    use super::*;
    #[cfg(feature = "remote-run")]
    use super::providers::Providers;
    #[cfg(feature = "remote-run")]
    use std::thread;

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_homebrew() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();
        client.set_rcvtimeo(Some(500));
        server.set_rcvtimeo(Some(500));

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&server).unwrap();
            assert_eq!("command::exec", req.popstr().unwrap().unwrap());
            assert_eq!("which brew", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("/usr/local/bin/brew").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();

            let req = ZMsg::recv(&server).unwrap();
            assert_eq!("command::exec", req.popstr().unwrap().unwrap());
            assert_eq!("brew list", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("nginx-filesystem").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None);
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
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();
        client.set_rcvtimeo(Some(500));
        server.set_rcvtimeo(Some(500));

        let agent_mock = thread::spawn(move || {
            assert_eq!("package::default_provider", server.recv_str().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("Homebrew").unwrap();
            rep.send(&server).unwrap();

            let req = ZMsg::recv(&server).unwrap();
            assert_eq!("command::exec", req.popstr().unwrap().unwrap());
            assert_eq!("which brew", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("/usr/local/bin/brew").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();

            let req = ZMsg::recv(&server).unwrap();
            assert_eq!("command::exec", req.popstr().unwrap().unwrap());
            assert_eq!("brew list", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("abc def nginx pkg").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None);
        let pkg = Package::new(&mut host, "nginx", None);
        assert!(pkg.is_ok());

        agent_mock.join().unwrap();
    }
}
