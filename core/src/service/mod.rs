// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Endpoint for managing system services.
//!
//! A service is represented by the `Service` struct, which is idempotent. This
//! means you can execute it repeatedly and it'll only run as needed.

mod providers;

use command::CommandStatus;
use errors::*;
use futures::{future, Future};
use host::Host;
use remote::{Request, Response};
#[doc(hidden)]
pub use self::providers::{
    factory, ServiceProvider, Debian, Homebrew, Launchctl,
    Rc, Redhat, Systemd
};
pub use self::providers::Provider;

/// Represents a system service to be managed for a host.
///
///## Example
///
/// Enable and start a service.
///
///```no_run
///extern crate futures;
///extern crate intecture_api;
///extern crate tokio_core;
///
///use futures::{future, Future};
///use intecture_api::errors::Error;
///use intecture_api::prelude::*;
///use tokio_core::reactor::Core;
///
///# fn main() {
///let mut core = Core::new().unwrap();
///let handle = core.handle();
///
///let host = Local::new(&handle).wait().unwrap();
///
///let nginx = Service::new(&host, "nginx");
///let result = nginx.enable()
///    .and_then(|_| {
///        nginx.action("start")
///            .and_then(|maybe_status| {
///                match maybe_status {
///                    Some(status) => Box::new(status.result().unwrap().map(|_| ())) as Box<Future<Item = (), Error = Error>>,
///                    None => Box::new(future::ok(())),
///                }
///            })
///    });
///
///core.run(result).unwrap();
///# }
///```
pub struct Service<H: Host> {
    host: H,
    provider: Option<Provider>,
    name: String,
}

impl<H: Host + 'static> Service<H> {
    /// Create a new `Service` with the default [`Provider`](enum.Provider.html).
    pub fn new(host: &H, name: &str) -> Service<H> {
        Service {
            host: host.clone(),
            provider: None,
            name: name.into(),
        }
    }

    /// Create a new `Service` with the specified [`Provider`](enum.Provider.html).
    ///
    ///## Example
    ///```
    ///extern crate futures;
    ///extern crate intecture_api;
    ///extern crate tokio_core;
    ///
    ///use futures::Future;
    ///use intecture_api::service::Provider;
    ///use intecture_api::prelude::*;
    ///use tokio_core::reactor::Core;
    ///
    ///# fn main() {
    ///let mut core = Core::new().unwrap();
    ///let handle = core.handle();
    ///
    ///let host = Local::new(&handle).wait().unwrap();
    ///
    ///Service::with_provider(&host, Provider::Systemd, "nginx");
    ///# }
    pub fn with_provider(host: &H, provider: Provider, name: &str) -> Service<H> {
        Service {
            host: host.clone(),
            provider: Some(provider),
            name: name.into(),
        }
    }

    /// Check if the service is currently running.
    pub fn running(&self) -> Box<Future<Item = bool, Error = Error>> {
        let request = Request::ServiceRunning(self.provider, self.name.clone());
        Box::new(self.host.request(request)
            .chain_err(|| ErrorKind::Request { endpoint: "Service", func: "running" })
            .map(|msg| {
                match msg.into_inner() {
                    Response::Bool(b) => b,
                    _ => unreachable!(),
                }
            }))
    }

    /// Perform an action for the service, e.g. "start".
    ///
    ///## Cross-platform services
    ///
    /// By design, actions are specific to a particular service and are not
    /// cross-platform. Actions are defined by the package maintainer that
    /// wrote the service configuration, thus users should take care that they
    /// adhere to the configuration for each platform they target.
    ///
    ///## Idempotence
    ///
    /// This function is idempotent when running either the "start" or "stop"
    /// actions, as it will check first whether the service is already running.
    /// Idempotence is represented by the type `Future<Item = Option<..>, ...>`.
    /// Thus if it returns `Option::None` then the service is already in the
    /// required state, and if it returns `Option::Some` then Intecture is
    /// attempting to transition the service to the required state.
    ///
    /// If this fn returns `Option::Some<..>`, the nested tuple will hold
    /// handles to the live output and the result of the action. Under the hood
    /// this reuses the `Command` endpoint, so see
    /// [`Command` docs](../command/struct.Command.html) for detailed
    /// usage.
    pub fn action(&self, action: &str) -> Box<Future<Item = Option<CommandStatus>, Error = Error>>
    {
        if action == "start" || action == "stop" {
            let host = self.host.clone();
            let name = self.name.clone();
            let provider = self.provider;
            let action = action.to_owned();

            Box::new(self.running()
                .and_then(move |running| {
                    if (running && action == "start") || (!running && action == "stop") {
                        Box::new(future::ok(None)) as Box<Future<Item = _, Error = Error>>
                    } else {
                        Self::do_action(&host, provider, &name, &action)
                    }
                }))
        } else {
            Self::do_action(&self.host, self.provider, &self.name, action)
        }
    }

    fn do_action(host: &H, provider: Option<Provider>, name: &str, action: &str)
        -> Box<Future<Item = Option<CommandStatus>, Error = Error>>
    {
        let request = Request::ServiceAction(provider, name.into(), action.into());
        Box::new(host.request(request)
            .chain_err(|| ErrorKind::Request { endpoint: "Service", func: "action" })
            .map(|msg| Some(CommandStatus::new(msg))))
    }

    /// Check if the service will start at boot.
    pub fn enabled(&self) -> Box<Future<Item = bool, Error = Error>> {
        let request = Request::ServiceEnabled(self.provider, self.name.clone());
        Box::new(self.host.request(request)
            .chain_err(|| ErrorKind::Request { endpoint: "Service", func: "enabled" })
            .map(|msg| {
                match msg.into_inner() {
                    Response::Bool(b) => b,
                    _ => unreachable!(),
                }
            }))
    }

    /// Instruct the service to start at boot.
    ///
    ///## Idempotence
    ///
    /// This function is idempotent, which is represented by the type
    /// `Future<Item = Option<..>, ...>`. Thus if it returns `Option::None`
    /// then the service is already enabled, and if it returns `Option::Some`
    /// then Intecture is attempting to enable the service.
    ///
    /// If this fn returns `Option::Some<..>`, the nested tuple will hold
    /// handles to the live output and the 'enable' command result. Under
    /// the hood this reuses the `Command` endpoint, so see
    /// [`Command` docs](../command/struct.Command.html) for detailed
    /// usage.
    pub fn enable(&self) -> Box<Future<Item = Option<()>, Error = Error>>
    {
        let host = self.host.clone();
        let provider = self.provider;
        let name = self.name.clone();

        Box::new(self.enabled()
            .and_then(move |enabled| {
                if enabled {
                    Box::new(future::ok(None)) as Box<Future<Item = _, Error = Error>>
                } else {
                    let request = Request::ServiceEnable(provider, name);
                    Box::new(host.request(request)
                        .chain_err(|| ErrorKind::Request { endpoint: "Service", func: "enable" })
                        .map(|msg| match msg.into_inner() {
                            Response::Null => Some(()),
                            _ => unreachable!(),
                        }))
                }
            }))
    }

    /// Prevent the service from starting at boot.
    ///
    ///## Idempotence
    ///
    /// This function is idempotent, which is represented by the type
    /// `Future<Item = Option<..>, ...>`. Thus if it returns `Option::None`
    /// then the service is already disabled, and if it returns `Option::Some`
    /// then Intecture is attempting to disable the service.
    ///
    /// If this fn returns `Option::Some<..>`, the nested tuple will hold
    /// handles to the live output and the 'disable' command result. Under
    /// the hood this reuses the `Command` endpoint, so see
    /// [`Command` docs](../command/struct.Command.html) for detailed
    /// usage.
    pub fn disable(&self) -> Box<Future<Item = Option<()>, Error = Error>>
    {
        let host = self.host.clone();
        let provider = self.provider;
        let name = self.name.clone();

        Box::new(self.enabled()
            .and_then(move |enabled| {
                if enabled {
                    let request = Request::ServiceDisable(provider, name);
                    Box::new(host.request(request)
                        .chain_err(|| ErrorKind::Request { endpoint: "Service", func: "disable" })
                        .map(|msg| match msg.into_inner() {
                            Response::Null => Some(()),
                            _ => unreachable!(),
                        }))
                } else {
                    Box::new(future::ok(None)) as Box<Future<Item = _, Error = Error>>
                }
            }))
    }
}
