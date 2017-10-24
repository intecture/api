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
    // These two lines are part of `tokio-core` and can be safely
    // ignored. So long as they appear at the top of your code,
    // all is fine with the world.
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Here's the meat of your project. In this example we're talking
    // to our local machine, so we use the `Local` host type.
    let host = Local::new().and_then(|host| {
        // Ok, we're in! Now we can pass our `host` handle to other
        // endpoints, which informs them of the server we mean to
        // talk to.

        // Let's start with something basic - a shell command.
        Command::new(&host, "whoami", None).and_then(|cmd| {
            // Now that we have our `Command` instance, let's run it.
            cmd.exec(&handle).and_then(|(stream, status)| {
                // At this point, our command is running. As the API
                // is asynchronous, we don't have to wait for it to
                // finish before inspecting its output. This is called
                // "streaming".

                // Our first argument, `stream`, is a stream of strings,
                // each of which represents a line of output. We can use
                // the `for_each` combinator to print these lines to
                // stdout.
                //
                // If printing isn't your thing, you are also
                // free to lick them or whatever you're into. I'm not
                // here to judge.
                stream.for_each(|line| { println!("{}", line); Ok(()) })

                // The second argument is a `Future` that represents the
                // command's exit status. Let's print that too*.
                //
                // * Same caveat as above RE: printing. This is a safe
                //   place.
                    .join(status.map(|s| println!("This command {} {}",
                        if s.success { "succeeded" } else { "failed" },
                        if let Some(e) = s.code { format!("with code {}", e) } else { String::new() })))
            })
        })
    });

    // This line is part of `tokio-core` and is used to execute the
    // chain of futures you've created above. You'll need to call
    // `core.run()` for each host you interact with, otherwise your
    // project will not run at all!
    core.run(host).unwrap();
}
