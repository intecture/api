// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

mod generic;

use command::ExitStatus;
use errors::*;
use futures::{future, Future};
use futures::stream::Stream;
use host::Host;
use provider::Provider;
pub use self::generic::Generic;
use tokio_core::reactor::Handle;

pub trait CommandProvider<H: Host>: Provider<H> {
    fn exec(&self, &H, &Handle, &str, &[String]) ->
        Box<Future<Item = (
            Box<Stream<Item = String, Error = Error>>,
            Box<Future<Item = ExitStatus, Error = Error>>
        ), Error = Error>>;
}

pub fn factory<H: Host + 'static>(host: &H) -> Box<Future<Item = Box<CommandProvider<H>>, Error = Error>> {
    Box::new(Generic::try_new(host)
        .and_then(|opt| match opt {
            Some(provider) => future::ok(Box::new(provider) as Box<CommandProvider<H>>),
            None => future::err(ErrorKind::ProviderUnavailable("Command").into())
        }))
}
