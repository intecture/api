// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use host::Host;
use futures::Future;

pub trait Provider<H: Host> {
    fn available(&H) -> Box<Future<Item = bool, Error = Error>> where Self: Sized;
    fn try_new(&H) -> Box<Future<Item = Option<Self>, Error = Error>> where Self: Sized;
}
