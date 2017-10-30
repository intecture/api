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

/// The Apt `Package` provider.
pub struct Apt;

impl Provider for Apt {
    fn available() -> bool {
        process::Command::new("/usr/bin/type")
            .arg("apt-get")
            .status()
            .unwrap()
            .success()
    }

    fn name(&self) -> ProviderName {
        ProviderName::PackageApt
    }
}

impl PackageProvider for Apt {
    fn installed(&self, handle: &Handle, name: &str, _: &Os) -> ExecutableResult {
        let handle = handle.clone();
        let name = name.to_owned();

        Box::new(process::Command::new("dpkg")
            .args(&["--get-selections"])
            .output_async(&handle)
            .chain_err(|| "Could not get installed packages")
            .and_then(move |output| {
                if output.status.success() {
                    let re = match Regex::new(&format!("(?m){}\\s+install$", name)) {
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
                                format!("Error running `dpkg --get-selections`: {}", String::from_utf8_lossy(&output.stderr))
                            )
                        )
                    )
                }
            }))
    }

    fn install(&self, handle: &Handle, name: &str) -> ExecutableResult {
        let cmd = match factory() {
            Ok(c) => c,
            Err(e) => return Box::new(future::ok(
                Message::WithoutBody(
                    ResponseResult::Err(
                        format!("{}", e.display_chain()))))),
        };
        cmd.exec(handle, name, &["apt-get".into(), "-y".into(), "install".into()])
    }

    fn uninstall(&self, handle: &Handle, name: &str) -> ExecutableResult {
        let cmd = match factory() {
            Ok(c) => c,
            Err(e) => return Box::new(future::ok(
                Message::WithoutBody(
                    ResponseResult::Err(
                        format!("{}", e.display_chain()))))),
        };
        cmd.exec(handle, name, &["apt-get".into(), "-y".into(), "remove".into()])
    }
}
