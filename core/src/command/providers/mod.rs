// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! OS abstractions for `Command`.

mod generic;

use command::ExitStatus;
use errors::*;
use futures::{future, Future};
use futures::stream::Stream;
use host::Host;
use provider::Provider;
pub use self::generic::Generic;
use tokio_core::reactor::Handle;

/// Trait for `Command` providers.
pub trait CommandProvider<H: Host>: Provider<H> {
    /// Execute the command for the given `Host`.
    fn exec(&self, &H, &Handle, &str, &[String]) ->
        Box<Future<Item = (
            Box<Stream<Item = String, Error = Error>>,
            Box<Future<Item = ExitStatus, Error = Error>>
        ), Error = Error>>;
}

/// Instantiate a new `CommandProvider` appropriate for the `Host`.
pub fn factory<H: Host + 'static>(host: &H) -> Box<Future<Item = Box<CommandProvider<H>>, Error = Error>> {
    Box::new(Generic::try_new(host)
        .and_then(|opt| match opt {
            Some(provider) => future::ok(Box::new(provider) as Box<CommandProvider<H>>),
            None => future::err(ErrorKind::ProviderUnavailable("Command").into())
        }))
}
