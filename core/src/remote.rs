// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::providers::CommandRunnable;
use erased_serde::Serialize;
use errors::*;
use futures::Future;
use host::local::Local;
use telemetry::providers::TelemetryRunnable;

pub trait Executable {
    fn exec(self, &Local) -> Box<Future<Item = Box<Serialize>, Error = Error>>;
}

#[derive(Serialize, Deserialize)]
pub enum Runnable {
    Command(CommandRunnable),
    Telemetry(TelemetryRunnable),
}

impl Executable for Runnable {
    fn exec(self, host: &Local) -> Box<Future<Item = Box<Serialize>, Error = Error>> {
        match self {
            Runnable::Command(p) => p.exec(host),
            Runnable::Telemetry(p) => p.exec(host),
        }
    }
}
