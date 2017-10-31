// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::providers::factory;
use error_chain::ChainedError;
use errors::*;
use futures::{future, Future};
use provider::Provider;
use remote::{ExecutableResult, ProviderName, Response, ResponseResult};
use std::process;
use super::PackageProvider;
use telemetry::Os;
use tokio_core::reactor::Handle;
use tokio_process::CommandExt;
use tokio_proto::streaming::Message;

/// The Nix `Package` provider.
pub struct Nix;

impl Provider for Nix {
    fn available() -> Result<bool> {
        Ok(process::Command::new("/usr/bin/type")
            .arg("nix-env")
            .status()
            .chain_err(|| "Could not determine provider availability")?
            .success())
    }

    fn name(&self) -> ProviderName {
        ProviderName::PackageNix
    }
}

impl PackageProvider for Nix {
    #[doc(hidden)]
    fn installed(&self, handle: &Handle, name: &str, _: &Os) -> ExecutableResult {
        let handle = handle.clone();
        let name = name.to_owned();

        Box::new(process::Command::new("nix-env")
            .args(&["--install", "--dry-run", &name])
            .output_async(&handle)
            .chain_err(|| "Could not check if package is installed")
            .and_then(move |output| {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    future::ok(
                        Message::WithoutBody(
                            ResponseResult::Ok(
                                Response::Bool(
                                    !stdout.contains("these paths will be fetched")))))
                } else {
                    future::ok(
                        Message::WithoutBody(
                            ResponseResult::Err(
                                format!("Error running `nix-env --install --dry-run {}`: {}", name, String::from_utf8_lossy(&output.stderr))
                            )
                        )
                    )
                }
            }))
    }

    #[doc(hidden)]
    fn install(&self, handle: &Handle, name: &str) -> ExecutableResult {
        let cmd = match factory() {
            Ok(c) => c,
            Err(e) => return Box::new(future::ok(
                Message::WithoutBody(
                    ResponseResult::Err(
                        format!("{}", e.display_chain()))))),
        };
        cmd.exec(handle, &["nix-env", "--install", name])
    }

    #[doc(hidden)]
    fn uninstall(&self, handle: &Handle, name: &str) -> ExecutableResult {
        let cmd = match factory() {
            Ok(c) => c,
            Err(e) => return Box::new(future::ok(
                Message::WithoutBody(
                    ResponseResult::Err(
                        format!("{}", e.display_chain()))))),
        };
        cmd.exec(handle, &["nix-env", "--uninstall", name])
    }
}
