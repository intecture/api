// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! OS abstractions for `Service`.

mod debian;
mod homebrew;
mod launchctl;
mod rc;
mod redhat;
mod systemd;

use errors::*;
use remote::ExecutableResult;
pub use self::debian::Debian;
pub use self::homebrew::Homebrew;
pub use self::launchctl::Launchctl;
pub use self::rc::Rc;
pub use self::redhat::Redhat;
pub use self::systemd::Systemd;
use telemetry::Telemetry;
use tokio_core::reactor::Handle;

/// Specific implementation of `Service`
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Provider {
    Debian,
    Homebrew,
    Launchctl,
    Rc,
    Redhat,
    Systemd,
}

pub trait ServiceProvider {
    fn available(&Telemetry) -> Result<bool> where Self: Sized;
    fn running(&self, &Handle, &str) -> ExecutableResult;
    fn action(&self, &Handle, &str, &str) -> ExecutableResult;
    fn enabled(&self, &Handle, &str) -> ExecutableResult;
    fn enable(&self, &Handle, &str) -> ExecutableResult;
    fn disable(&self, &Handle, &str) -> ExecutableResult;
}

#[doc(hidden)]
pub fn factory(telemetry: &Telemetry) -> Result<Box<ServiceProvider>> {
    if Systemd::available(telemetry)? {
        Ok(Box::new(Systemd))
    } else if Debian::available(telemetry)? {
        Ok(Box::new(Debian))
    } else if Homebrew::available(telemetry)? {
        Ok(Box::new(Homebrew::new(telemetry)))
    } else if Launchctl::available(telemetry)? {
        Ok(Box::new(Launchctl::new(telemetry)))
    } else if Rc::available(telemetry)? {
        Ok(Box::new(Rc))
    } else if Redhat::available(telemetry)? {
        Ok(Box::new(Redhat))
    } else {
        Err(ErrorKind::ProviderUnavailable("Service").into())
    }
}
