// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use error_chain::ChainedError;
use errors::*;
use futures::future;
use remote::{ExecutableResult, ResponseResult};
use std::process;
use super::{Launchctl, ServiceProvider};
use telemetry::Telemetry;
use tokio_core::reactor::Handle;
use tokio_proto::streaming::Message;

pub struct Homebrew {
    inner: Launchctl,
}

impl Homebrew {
    #[doc(hidden)]
    pub fn new(telemetry: &Telemetry) -> Homebrew {
        Homebrew {
            inner: Launchctl::new(telemetry),
        }
    }
}

impl ServiceProvider for Homebrew {
    fn available(telemetry: &Telemetry) -> Result<bool> {
        let brew = process::Command::new("/usr/bin/type")
            .arg("brew")
            .status()
            .chain_err(|| "Could not determine provider availability")?
            .success();

        Ok(brew && Launchctl::available(telemetry)?)
    }

    fn running(&self, handle: &Handle, name: &str) -> ExecutableResult {
        self.inner.running(handle, name)
    }

    fn action(&self, handle: &Handle, name: &str, action: &str) -> ExecutableResult {
        // @todo This isn't the most reliable method. Ideally a user would
        // invoke these commands themselves.
        let result = if action == "stop" {
            self.inner.uninstall_plist(name)
        } else {
            let path = format!("/usr/local/opt/{}/homebrew.mxcl.{0}.plist", name);
            self.inner.install_plist(path)
        };

        if let Err(e) = result {
            return Box::new(future::ok(
                Message::WithoutBody(
                    ResponseResult::Err(
                        format!("{}", e.display_chain())))))
        }

        self.inner.action(handle, name, action)
    }

    fn enabled(&self, handle: &Handle, name: &str) -> ExecutableResult {
        self.inner.enabled(handle, name)
    }

    fn enable(&self, handle: &Handle, name: &str) -> ExecutableResult {
        self.inner.enable(handle, name)
    }

    fn disable(&self, handle: &Handle, name: &str) -> ExecutableResult {
        self.inner.disable(handle, name)
    }
}
