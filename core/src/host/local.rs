// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use futures::{future, Future};
use {Executable, Runnable};
use serde::Deserialize;
use serde_json;
use std::sync::Arc;
use super::Host;
use telemetry::{self, Telemetry};

pub struct Local {
    telemetry: Option<Telemetry>,
}

impl Local {
    /// Create a new Host targeting the local machine.
    pub fn new() -> Box<Future<Item = Arc<Local>, Error = Error>> {
        let mut host = Arc::new(Local {
            telemetry: None,
        });

        Box::new(telemetry::load(&host).map(|t| {
            Arc::get_mut(&mut host).unwrap().telemetry = Some(t);
            host
        }))
    }
}

impl Host for Local {
    fn telemetry(&self) -> &Telemetry {
        self.telemetry.as_ref().unwrap()
    }

    fn run<D: 'static>(&self, provider: Runnable) -> Box<Future<Item = D, Error = Error>>
        where for<'de> D: Deserialize<'de>
    {
        Box::new(provider.exec()
            .chain_err(|| "Could not run provider")
                .and_then(|s| {
                    match serde_json::to_value(s).chain_err(|| "Could not run provider") {
                        Ok(v) => match serde_json::from_value::<D>(v).chain_err(|| "Could not run provider") {
                            Ok(d) => future::ok(d),
                            Err(e) => future::err(e),
                        },
                        Err(e) => future::err(e),
                    }
                }))
    }
}
