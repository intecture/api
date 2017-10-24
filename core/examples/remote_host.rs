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
    // These two lines are part of `tokio-core` and can be safely
    // ignored. So long as they appear at the top of your code,
    // all is fine with the world.
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Here's the meat of your project. In this example we're talking
    // to a remote machine. You'll note that this is the `Plain` host
    // type, where you might have been expecting `Remote` or some such.
    // This is to signify that this host type sends data in the clear,
    // rather than encrypting it. Thus the usual disclaimer about
    // secure networks and trust applies.
    let host = Plain::connect("127.0.0.1:7101", &handle).map(|host| {
        // Ok, we're in! Now we can pass our `host` handle to other
        // endpoints, which informs them of the server we mean to
        // talk to. See basic.rs for more usage.
        println!("Connected to {}", host.telemetry().hostname);
    });

    // This line is part of `tokio-core` and is used to execute the
    // chain of futures you've created above. You'll need to call
    // `core.run()` for each host you interact with, otherwise your
    // project will not run at all!
    core.run(host).unwrap();
}
