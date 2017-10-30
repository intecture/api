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
use regex::Regex;
use remote::{ExecutableResult, ProviderName, Response, ResponseResult};
use std::process;
use super::PackageProvider;
use telemetry::Os;
use tokio_core::reactor::Handle;
use tokio_process::CommandExt;
use tokio_proto::streaming::Message;

/// The Yum `Package` provider.
pub struct Yum;

impl Provider for Yum {
    fn available() -> Result<bool> {
        Ok(process::Command::new("/usr/bin/type")
            .arg("yum")
            .status()
            .chain_err(|| "Could not determine provider availability")?
            .success())
    }

    fn name(&self) -> ProviderName {
        ProviderName::PackageYum
    }
}

impl PackageProvider for Yum {
    #[doc(hidden)]
    fn installed(&self, handle: &Handle, name: &str, os: &Os) -> ExecutableResult {
        let handle = handle.clone();
        let name = name.to_owned();
        let arch = os.arch.clone();

        Box::new(process::Command::new("yum")
            .args(&["list", "installed"])
            .output_async(&handle)
            .chain_err(|| "Could not get installed packages")
            .and_then(move |output| {
                if output.status.success() {
                    let re = match Regex::new(&format!("(?m)^{}\\.({}|noarch)\\s+", name, arch)) {
                        Ok(r) => r,
                        Err(e) => return future::err(ErrorKind::Regex(e).into()),
                    };
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    future::ok(
                        Message::WithoutBody(
                            ResponseResult::Ok(
                                Response::Bool(
                                    re.is_match(&stdout)))))
                } else {
                    future::ok(
                        Message::WithoutBody(
                            ResponseResult::Err(
                                format!("Error running `yum list installed`: {}", String::from_utf8_lossy(&output.stderr))
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
        cmd.exec(handle, name, &["yum".into(), "-y".into(), "install".into()])
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
        cmd.exec(handle, name, &["yum".into(), "-y".into(), "remove".into()])
    }
}
