// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! OS abstractions for `Command`.

mod generic;

use command::ExitStatus;
use errors::*;
use provider::Provider;
use remote::ExecutableResult;
pub use self::generic::Generic;
use tokio_core::reactor::Handle;

/// Trait for specific `Command` implementations.
pub trait CommandProvider: Provider {
    #[doc(hidden)]
    fn exec(&self, &Handle, &str, &[String]) -> ExecutableResult;
}

#[doc(hidden)]
pub fn factory() -> Result<Box<CommandProvider>> {
    if Generic::available() {
        Ok(Box::new(Generic))
    } else {
        Err(ErrorKind::ProviderUnavailable("Command").into())
    }
}
