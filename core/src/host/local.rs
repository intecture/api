// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! A connection to the local machine.

use errors::*;
use futures::Future;
use std::sync::Arc;
use super::{Host, HostType};
use telemetry::{self, Telemetry};

/// A `Host` type that talks directly to the local machine.
#[derive(Clone)]
pub struct Local {
    inner: Arc<Inner>,
}

struct Inner {
    telemetry: Option<Telemetry>,
}

impl Local {
    /// Create a new `Host` targeting the local machine.
    pub fn new() -> Box<Future<Item = Local, Error = Error>> {
        let mut host = Local {
            inner: Arc::new(Inner { telemetry: None }),
        };

        Box::new(telemetry::providers::factory(&host)
            .chain_err(|| "Could not load telemetry for host")
            .map(|t| {
                Arc::get_mut(&mut host.inner).unwrap().telemetry = Some(t);
                host
            }))
    }
}

impl Host for Local {
    fn telemetry(&self) -> &Telemetry {
        self.inner.telemetry.as_ref().unwrap()
    }

    #[doc(hidden)]
    fn get_type<'a>(&'a self) -> HostType<'a> {
        HostType::Local(&self)
    }
}
