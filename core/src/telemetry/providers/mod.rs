// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! OS abstractions for `Telemetry`.

mod centos;
mod debian;
mod fedora;
mod freebsd;
mod macos;
mod nixos;
mod ubuntu;

pub use self::centos::Centos;
pub use self::debian::Debian;
pub use self::fedora::Fedora;
pub use self::freebsd::Freebsd;
pub use self::macos::Macos;
pub use self::nixos::Nixos;
pub use self::ubuntu::Ubuntu;

use errors::*;
use provider::Provider;
use remote::ExecutableResult;

/// Trait for specific `Telemetry` implementations.
pub trait TelemetryProvider: Provider {
    #[doc(hidden)]
    fn load(&self) -> ExecutableResult;
}

#[doc(hidden)]
pub fn factory() -> Result<Box<TelemetryProvider>> {
    if Centos::available()? {
        Ok(Box::new(Centos))
    }
    else if Debian::available()? {
        Ok(Box::new(Debian))
    }
    else if Fedora::available()? {
        Ok(Box::new(Fedora))
    }
    else if Freebsd::available()? {
        Ok(Box::new(Freebsd))
    }
    else if Macos::available()? {
        Ok(Box::new(Macos))
    }
    else if Nixos::available()? {
        Ok(Box::new(Nixos))
    }
    else if Ubuntu::available()? {
        Ok(Box::new(Ubuntu))
    } else {
        Err(ErrorKind::ProviderUnavailable("Telemetry").into())
    }
}
