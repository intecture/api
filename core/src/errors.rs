// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use futures::Future;
use std::{error, io};

error_chain! {
    foreign_links {
        Io(io::Error);
    }

    errors {
        InvalidTelemetryKey {
            cmd: &'static str,
            key: String,
        } {
            description("Provided key not found in output"),
            display("Provided key '{}' not found in {} output", key, cmd),
        }

        ProviderUnavailable(p: &'static str) {
            description("No providers available"),
            display("No providers available for {}", p),
        }

        Request {
            endpoint: &'static str,
            func: &'static str,
        } {
            description("Could not run provider function on host"),
            display("Could not run {}::{}() on host", endpoint, func),
        }

        SystemCommand(c: &'static str) {
            description("Error running system command"),
            display("Error running system command '{}'", c),
        }

        SystemCommandOutput(c: &'static str) {
            description("Could not understand output of system command"),
            display("Could not understand output of system command '{}'", c),
        }

        SystemFile(c: &'static str) {
            description("Could not open system file"),
            display("Could not open system file '{}'", c),
        }

        SystemFileOutput(c: &'static str) {
            description("Could not understand output of system file"),
            display("Could not understand output of system file '{}'", c),
        }
    }
}

// @todo This should disappear once Futures are officially supported
// by error_chain.
// See: https://github.com/rust-lang-nursery/error-chain/issues/90
pub type SFuture<T> = Box<Future<Item = T, Error = Error>>;

pub trait FutureChainErr<T> {
    fn chain_err<F, E>(self, callback: F) -> SFuture<T>
        where F: FnOnce() -> E + 'static,
              E: Into<ErrorKind>;
}

impl<F> FutureChainErr<F::Item> for F
    where F: Future + 'static,
          F::Error: error::Error + Send + 'static,
{
    fn chain_err<C, E>(self, callback: C) -> SFuture<F::Item>
        where C: FnOnce() -> E + 'static,
              E: Into<ErrorKind>,
    {
        Box::new(self.then(|r| r.chain_err(callback)))
    }
}
