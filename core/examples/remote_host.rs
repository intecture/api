// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate futures;
extern crate intecture_api;
extern crate tokio_core;

use futures::Future;
use intecture_api::prelude::*;
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let host = Plain::connect("127.0.0.1:7101", &handle).map(|host| {
        println!("Connected to {}", host.telemetry().hostname);
    });

    core.run(host).unwrap();
}
