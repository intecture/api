// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::factory;
use error_chain::ChainedError;
use errors::*;
use futures::{future, Future};
use remote::{ExecutableResult, Response, ResponseResult};
use std::process;
use super::PackageProvider;
use telemetry::Os;
use tokio_core::reactor::Handle;
use tokio_process::CommandExt;
use tokio_proto::streaming::Message;

pub struct Pkg;

impl PackageProvider for Pkg {
    fn available() -> Result<bool> {
        Ok(process::Command::new("/usr/bin/type")
            .arg("pkg")
            .status()
            .chain_err(|| "Could not determine provider availability")?
            .success())
    }

    fn installed(&self, handle: &Handle, name: &str, _: &Os) -> ExecutableResult {
        let handle = handle.clone();
        let name = name.to_owned();

        Box::new(process::Command::new("pkg")
            .args(&["query", "\"%n\"", &name])
            .output_async(&handle)
            .chain_err(|| "Could not get installed packages")
            .and_then(move |output| {
                future::ok(
                    Message::WithoutBody(
                        ResponseResult::Ok(
                            Response::Bool(
                                output.status.success()))))
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
        cmd.exec(handle, &["pkg", "install", "-y", name])
    }

    fn uninstall(&self, handle: &Handle, name: &str) -> ExecutableResult {
        let cmd = match factory() {
            Ok(c) => c,
            Err(e) => return Box::new(future::ok(
                Message::WithoutBody(
                    ResponseResult::Err(
                        format!("{}", e.display_chain()))))),
        };
        cmd.exec(handle, &["pkg", "delete", "-y", name])
    }
}
