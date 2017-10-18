// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Manages the connection between the API and your servers.

pub mod local;
pub mod remote;

use command::providers::{CommandProvider, factory as cmd_factory};
use errors::*;
use futures::Future;
use telemetry::Telemetry;

pub trait Host: Clone {
    //// Retrieve Telemetry
    fn telemetry(&self) -> &Telemetry;
    #[doc(hidden)]
    fn get_type<'a>(&'a self) -> HostType<'a>;
    #[doc(hidden)]
    fn command_provider(&self) -> Box<Future<Item = Box<CommandProvider<Self>>, Error = Error>> where Self: 'static {
        cmd_factory(&self)
    }
}

pub enum HostType<'a> {
    Local(&'a local::Local),
    Remote(&'a remote::Plain),
}
