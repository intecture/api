// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate futures;
extern crate intecture_api;
extern crate tokio_core;

use futures::{Future, Stream};
use intecture_api::prelude::*;
use tokio_core::reactor::Core;

fn main() {
    // These two lines are part of `tokio-core` and can be safely ignored. So
    // long as they appear at the top of your code, all is fine with the world.
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Here's the meat of your project. In this example we're talking to our
    // local machine, so we use the `Local` host type.
    let host = Local::new(&handle).and_then(|host| {
        // Ok, we're in! Now we can pass our `host` handle to other endpoints,
        // which informs them of the server we mean to talk to.

        // Let's start with something basic - a shell command.
        let cmd = Command::new(&host, "whoami", None);
        cmd.exec().and_then(|mut status| {
            // At this point, our command is running. As the API is
            // asynchronous, we don't have to wait for it to finish before
            // inspecting its output. This is called "streaming".

            // First let's grab the stream from `CommandStatus`. This stream is
            // a stream of strings, each of which represents a line of command
            // output. We can use the `for_each` combinator to print these
            // lines to stdout.
            //
            // If printing isn't your thing, you are also free to lick them or
            // whatever you're into. I'm not here to judge.
            let stream = status.take_stream()
                .unwrap() // Unwrap is fine here as we haven't called it before
                .for_each(|line| { println!("{}", line); Ok(()) });

            // Next, let's check on the result of our command.
            // `CommandStatus` is a `Future` that represents the command's
            // exit status. We can use the `map` combinator to print it out.*
            //
            // * Same caveat as above RE: printing. This is a safe
            //   place.
            let status = status.map(|s| println!("This command {} {}",
                if s.success { "succeeded" } else { "failed" },
                if let Some(e) = s.code { format!("with code {}", e) } else { String::new() }));

            // Finally, we need to return these two `Future`s (stream and
            // status) so that they will be executed by the event loop. Sadly
            // we can't return them both as a tuple, so we use the join
            // combinator instead to turn them into a single `Future`. Easy!
            stream.join(status)
        })
    });

    // This line is part of `tokio-core` and is used to execute the
    // chain of futures you've created above. You'll need to call
    // `core.run()` for each host you interact with, otherwise your
    // project will not run at all!
    core.run(host).unwrap();
}
