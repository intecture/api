// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::factory;
use error_chain::ChainedError;
use errors::*;
use futures::{future, Future};
use regex::Regex;
use remote::{ExecutableResult, Response, ResponseResult};
use std::process;
use super::ServiceProvider;
use telemetry::{OsFamily, Telemetry};
use tokio_core::reactor::Handle;
use tokio_process::CommandExt;
use tokio_proto::streaming::Message;

pub struct Rc;

impl ServiceProvider for Rc {
    fn available(telemetry: &Telemetry) -> Result<bool> {
        Ok(telemetry.os.family == OsFamily::Bsd)
    }

    fn running(&self, handle: &Handle, name: &str) -> ExecutableResult {
        let status = match process::Command::new("service")
            .args(&[name, "status"])
            .status_async2(handle)
            .chain_err(|| "Error checking if service is running")
        {
            Ok(s) => s,
            Err(e) => return Box::new(future::err(e)),
        };
        Box::new(status.map(|s| Message::WithoutBody(
                ResponseResult::Ok(
                    Response::Bool(s.success()))))
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("service <service> status"))))
    }

    fn action(&self, handle: &Handle, name: &str, action: &str) -> ExecutableResult {
        let cmd = match factory() {
            Ok(c) => c,
            Err(e) => return Box::new(future::ok(
                Message::WithoutBody(
                    ResponseResult::Err(
                        format!("{}", e.display_chain()))))),
        };
        cmd.exec(handle, &["service", action, name])
    }

    fn enabled(&self, handle: &Handle, name: &str) -> ExecutableResult {
        let name = name.to_owned();

        Box::new(process::Command::new("/usr/sbin/sysrc")
            .arg(&format!("{}_enable", name)) // XXX Assuming "_enable" is the correct suffix
            .output_async(&handle)
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("/usr/sbin/sysrc <service>_enable")))
            .and_then(move |output| {
                if output.status.success() {
                    let re = match Regex::new(&format!("^{}_enable: (?i:no)", name)) {
                        Ok(r) => r,
                        Err(e) => return future::err(Error::with_chain(e, ErrorKind::Msg("Could not create Rc::enabled Regex".into())))
                    };
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // XXX Assuming anything other than "no" is enabled
                    let is_match = !re.is_match(&stdout);

                    future::ok(Message::WithoutBody(ResponseResult::Ok(Response::Bool(is_match))))
                } else {
                    future::ok(Message::WithoutBody(ResponseResult::Ok(Response::Bool(false))))
                }
            }))
    }

    fn enable(&self, handle: &Handle, name: &str) -> ExecutableResult {
        Box::new(process::Command::new("/usr/sbin/sysrc")
            .arg(&format!("{}_enable=\"YES\"", name))
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
        Box::new(process::Command::new("/usr/sbin/sysrc")
            .arg(&format!("{}_enable=\"NO\"", name))
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
