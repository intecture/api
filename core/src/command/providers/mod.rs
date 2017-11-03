// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! OS abstractions for `Command`.

mod generic;

use command::ExitStatus;
use errors::*;
use remote::ExecutableResult;
pub use self::generic::Generic;
use tokio_core::reactor::Handle;

/// Specific implementation of `Command`
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Provider {
    Generic,
}

pub trait CommandProvider {
    fn available() -> bool where Self: Sized;
    fn exec(&self, &Handle, &[&str]) -> ExecutableResult;
}

#[doc(hidden)]
pub fn factory() -> Result<Box<CommandProvider>> {
    if Generic::available() {
        Ok(Box::new(Generic))
    } else {
        Err(ErrorKind::ProviderUnavailable("Command").into())
    }
}
