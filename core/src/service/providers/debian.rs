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
use std::fs::read_dir;
use std::process;
use super::ServiceProvider;
use telemetry::{LinuxDistro, OsFamily, Telemetry};
use tokio_core::reactor::Handle;
use tokio_process::CommandExt;
use tokio_proto::streaming::Message;

pub struct Debian;

impl ServiceProvider for Debian {
    fn available(telemetry: &Telemetry) -> Result<bool> {
        Ok(telemetry.os.family == OsFamily::Linux(LinuxDistro::Debian))
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

        Box::new(process::Command::new("/sbin/runlevel")
            .output_async(&handle)
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("/sbin/runlevel")))
            .and_then(move |output| {
                if output.status.success() {
                    let mut stdout = (*String::from_utf8_lossy(&output.stdout)).to_owned();
                    let runlevel = match stdout.pop() {
                        Some(c) => c,
                        None => return future::ok(Message::WithoutBody(ResponseResult::Err("Could not determine current runlevel".into()))),
                    };

                    let dir = match read_dir(&format!("/etc/rc{}.d", runlevel)) {
                        Ok(dir) => dir,
                        Err(e) => return future::err(Error::with_chain(e, ErrorKind::Msg("Could not read rc dir".into()))),
                    };

                    let regex = match Regex::new(&format!("/S[0-9]+{}$", name)) {
                        Ok(r) => r,
                        Err(e) => return future::err(Error::with_chain(e, ErrorKind::Msg("Could not create Debian::enabled regex".into()))),
                    };
                    let mut enabled = false;
                    for file in dir {
                        if let Ok(file) = file {
                            if regex.is_match(&file.file_name().to_string_lossy()) {
                                enabled = true;
                                break;
                            }
                        }
                    }

                    future::ok(Message::WithoutBody(ResponseResult::Ok(Response::Bool(enabled))))
                } else {
                    future::err(ErrorKind::SystemCommand("/usr/bin/runlevel").into())
                }
            }))
    }

    fn enable(&self, handle: &Handle, name: &str) -> ExecutableResult {
        Box::new(process::Command::new("/usr/sbin/update-rc.d")
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
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("update-rc.d enable <service>"))))
    }

    fn disable(&self, handle: &Handle, name: &str) -> ExecutableResult {
        Box::new(process::Command::new("/usr/sbin/update-rc.d")
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
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("update-rc.d disable <service>"))))
    }
}
