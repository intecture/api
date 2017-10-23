// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use futures::Future;
use intecture_api;
use std::{convert, error, io};

error_chain! {
    links {
        Api(intecture_api::errors::Error, intecture_api::errors::ErrorKind);
    }
}

impl convert::From<Error> for io::Error {
    fn from(e: Error) -> io::Error {
        // @todo Return whole error chain
        io::Error::new(io::ErrorKind::Other, e.description())
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
