// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

mod generic;

use errors::*;
use erased_serde::Serialize;
use futures::{future, Future};
use host::Host;
use host::local::Local;
use provider::Provider;
use remote::Executable;
pub use self::generic::{Generic, GenericRunnable};
use super::CommandResult;

pub trait CommandProvider<H: Host>: Provider<H> {
    fn exec(&self, &H, &str, &[String]) -> Box<Future<Item = CommandResult, Error = Error>>;
}

#[doc(hidden)]
#[derive(Serialize, Deserialize)]
pub enum CommandRunnable {
    Generic(GenericRunnable)
}

impl Executable for CommandRunnable {
    fn exec(self, host: &Local) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            CommandRunnable::Generic(p) => p.exec(host)
        }
    }
}

pub fn factory<H: Host + 'static>(host: &H) -> Box<Future<Item = Box<CommandProvider<H>>, Error = Error>> {
    Box::new(Generic::try_new(host)
        .and_then(|opt| match opt {
            Some(provider) => future::ok(Box::new(provider) as Box<CommandProvider<H>>),
            None => future::err(ErrorKind::ProviderUnavailable("Command").into())
        }))
}
