// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Manages the connection between the API and your servers.

pub mod local;
pub mod remote;

use errors::*;
use Runnable;
use futures::Future;
use serde::Deserialize;
use telemetry::Telemetry;

pub trait Host {
    /// Retrieve Telemetry
    fn telemetry(&self) -> &Telemetry;

    #[doc(hidden)]
    fn run<D: 'static>(&self, Runnable) -> Box<Future<Item = D, Error = Error>>
        where for<'de> D: Deserialize<'de>;
}
