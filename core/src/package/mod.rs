// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Endpoint for managing packages.
//!
//! A package is represented by the `Package` struct, which is idempotent. This
//! means you can execute it repeatedly and it'll only run as needed.

pub mod providers;

use command::CommandStatus;
use errors::*;
use futures::{future, Future};
use host::Host;
use remote::{Request, Response};
use self::providers::PackageProvider;

/// Represents a system package to be managed for a host.
///
///# Example
///
/// Install a package and print the result.
///
///```no_run
///extern crate futures;
///extern crate intecture_api;
///extern crate tokio_core;
///
///use futures::{future, Future};
///use intecture_api::errors::*;
///use intecture_api::prelude::*;
///use tokio_core::reactor::Core;
///
///# fn main() {
///let mut core = Core::new().unwrap();
///let handle = core.handle();
///
///let host = Local::new(&handle).wait().unwrap();
///
///let nginx = Package::new(&host, "nginx");
///let result = nginx.install().and_then(|status| {
///    match status {
///        // We're performing the install
///        Some(status) => Box::new(status.result().unwrap()
///            .map(|_| println!("Installed"))
///            .map_err(|e| {
///                match *e.kind() {
///                    ErrorKind::Command(ref output) => println!("Failed with output: {}", output),
///                    _ => unreachable!(),
///                }
///                e
///            })),
///
///        // This package is already installed
///        None => {
///            println!("Already installed");
///            Box::new(future::ok(())) as Box<Future<Item = _, Error = Error>>
///        },
///    }
///});
///
///core.run(result).unwrap();
///# }
///```
pub struct Package<H: Host> {
    host: H,
    provider: Option<Box<PackageProvider>>,
    name: String,
}

impl<H: Host + 'static> Package<H> {
    /// Create a new `Package` with the default `PackageProvider`.
    pub fn new(host: &H, name: &str) -> Package<H> {
        Package {
            host: host.clone(),
            provider: None,
            name: name.into(),
        }
    }

    /// Create a new `Package` with the specified `PackageProvider`.
    ///
    ///## Example
    ///```
    ///extern crate futures;
    ///extern crate intecture_api;
    ///extern crate tokio_core;
    ///
    ///use futures::Future;
    ///use intecture_api::package::providers::Yum;
    ///use intecture_api::prelude::*;
    ///use tokio_core::reactor::Core;
    ///
    ///# fn main() {
    ///let mut core = Core::new().unwrap();
    ///let handle = core.handle();
    ///
    ///let host = Local::new(&handle).wait().unwrap();
    ///
    ///Package::with_provider(&host, Yum, "nginx");
    ///# }
    pub fn with_provider<P>(host: &H, provider: P, name: &str) -> Package<H>
        where P: PackageProvider + 'static
    {
        Package {
            host: host.clone(),
            provider: Some(Box::new(provider)),
            name: name.into(),
        }
    }

    /// Check if the package is installed.
    pub fn installed(&self) -> Box<Future<Item = bool, Error = Error>> {
        let request = Request::PackageInstalled(self.provider.as_ref().map(|p| p.name()), self.name.clone());
        Box::new(self.host.request(request)
            .chain_err(|| ErrorKind::Request { endpoint: "Package", func: "installed" })
            .map(|msg| {
                match msg.into_inner() {
                    Response::Bool(b) => b,
                    _ => unreachable!(),
                }
            }))
    }

    /// Install the package.
    ///
    /// This function is idempotent, which is represented by the type
    /// `Future<Item = Option<..>, ...>`. Thus if it returns `Option::None`
    /// then the package is already installed, and if it returns `Option::Some`
    /// then Intecture is attempting to install the package.
    ///
    /// If this fn returns `Option::Some<..>`, the nested tuple will hold
    /// handles to the live output and the result of the installation. Under
    /// the hood this reuses the `Command` endpoint, so see
    /// [`Command` docs](../command/struct.Command.html) for detailed
    /// usage.
    pub fn install(&self) -> Box<Future<Item = Option<CommandStatus>, Error = Error>>
    {
        let host = self.host.clone();
        let provider = self.provider.as_ref().map(|p| p.name());
        let name = self.name.clone();

        Box::new(self.installed()
            .and_then(move |installed| {
                if installed {
                    Box::new(future::ok(None)) as Box<Future<Item = _, Error = Error>>
                } else {
                    Box::new(host.request(Request::PackageInstall(provider, name))
                        .chain_err(|| ErrorKind::Request { endpoint: "Package", func: "install" })
                        .map(|msg| {
                            Some(CommandStatus::new(msg))
                        }))
                }
            }))
    }

    /// Uninstall the package.
    ///
    /// This function is idempotent, which is represented by the type
    /// `Future<Item = Option<..>, ...>`. Thus if it returns `Option::None`
    /// then the package is already uninstalled, and if it returns
    /// `Option::Some` then Intecture is attempting to uninstall the package.
    ///
    /// If this fn returns `Option::Some<..>`, the nested tuple will hold
    /// handles to the live output and the result of the deinstallation. Under
    /// the hood this reuses the `Command` endpoint, so see
    /// [`Command` docs](../command/struct.Command.html) for detailed
    /// usage.
    pub fn uninstall(&self) -> Box<Future<Item = Option<CommandStatus>, Error = Error>>
    {
        let host = self.host.clone();
        let provider = self.provider.as_ref().map(|p| p.name());
        let name = self.name.clone();

        Box::new(self.installed()
            .and_then(move |installed| {
                if installed {
                    Box::new(host.request(Request::PackageUninstall(provider, name))
                        .chain_err(|| ErrorKind::Request { endpoint: "Package", func: "uninstall" })
                        .map(|msg| {
                            Some(CommandStatus::new(msg))
                        }))
                } else {
                    Box::new(future::ok(None)) as Box<Future<Item = _, Error = Error>>
                }
            }))
    }
}
