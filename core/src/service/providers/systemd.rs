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
use super::ServiceProvider;
use telemetry::Telemetry;
use tokio_core::reactor::Handle;
use tokio_process::CommandExt;
use tokio_proto::streaming::Message;

pub struct Systemd;

impl ServiceProvider for Systemd {
    fn available(_: &Telemetry) -> Result<bool> {
        let output = process::Command::new("/usr/bin/stat")
            .args(&["--format=%N", "/proc/1/exe"])
            .output()
            .chain_err(|| "Could not determine provider availability")?;

        if output.status.success() {
            let out = String::from_utf8_lossy(&output.stdout);
            Ok(out.contains("systemd"))
        } else {
            Err(ErrorKind::SystemCommand("/usr/bin/stat").into())
        }
    }

    fn running(&self, handle: &Handle, name: &str) -> ExecutableResult {
        let status = match process::Command::new("systemctl")
            .args(&["is-active", name])
            .status_async2(handle)
            .chain_err(|| "Error checking if service is running")
        {
            Ok(s) => s,
            Err(e) => return Box::new(future::err(e)),
        };
        Box::new(status.map(|s| Message::WithoutBody(
                ResponseResult::Ok(
                    Response::Bool(s.success()))))
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("systemctl is-active"))))
    }

    fn action(&self, handle: &Handle, name: &str, action: &str) -> ExecutableResult {
        let cmd = match factory() {
            Ok(c) => c,
            Err(e) => return Box::new(future::ok(
                Message::WithoutBody(
                    ResponseResult::Err(
                        format!("{}", e.display_chain()))))),
        };
        cmd.exec(handle, &["systemctl", action, name])
    }

    fn enabled(&self, handle: &Handle, name: &str) -> ExecutableResult {
        let status = match process::Command::new("systemctl")
            .args(&["is-enabled", name])
            .status_async2(handle)
            .chain_err(|| "Error checking if service is enabled")
        {
            Ok(s) => s,
            Err(e) => return Box::new(future::err(e)),
        };
        Box::new(status.map(|s| Message::WithoutBody(
                ResponseResult::Ok(
                    Response::Bool(s.success()))))
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("systemctl is-enabled"))))
    }

    fn enable(&self, handle: &Handle, name: &str) -> ExecutableResult {
        Box::new(process::Command::new("systemctl")
            .args(&["enable", name])
            .output_async(handle)
            .map(|out| {
                if out.status.success() {
                    Message::WithoutBody(ResponseResult::Ok(Response::Null))
                } else {
                    Message::WithoutBody(ResponseResult::Err(
                        format!("Could not enable service: {}", String::from_utf8_lossy(&out.stderr))))
                }
            })
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("systemctl enable <service>"))))
    }

    fn disable(&self, handle: &Handle, name: &str) -> ExecutableResult {
        Box::new(process::Command::new("systemctl")
            .args(&["disable", name])
            .output_async(handle)
            .map(|out| {
                if out.status.success() {
                    Message::WithoutBody(ResponseResult::Ok(Response::Null))
                } else {
                    Message::WithoutBody(ResponseResult::Err(
                        format!("Could not disable service: {}", String::from_utf8_lossy(&out.stderr))))
                }
            })
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("systemctl disable <service>"))))
    }
}
