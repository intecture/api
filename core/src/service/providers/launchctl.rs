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
use std::{fs, process};
use std::path::{Path, PathBuf};
use super::ServiceProvider;
use telemetry::{OsFamily, Telemetry};
use tokio_core::reactor::Handle;
use tokio_process::CommandExt;
use tokio_proto::streaming::Message;

pub struct Launchctl {
    domain_target: String,
    service_path: PathBuf,
}

impl Launchctl {
    #[doc(hidden)]
    pub fn new(telemetry: &Telemetry) -> Launchctl {
        let (domain_target, service_path) = if telemetry.user.is_root() {
            ("system".into(), "/Library/LaunchDaemons".into())
        } else {
            let mut path = telemetry.user.home_dir.clone();
            path.push("Library/LaunchAgents");
            (format!("gui/{}", telemetry.user.uid), path)
        };

        Launchctl { domain_target, service_path }
    }

    #[doc(hidden)]
    pub fn install_plist<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if let Some(name) = path.as_ref().file_name() {
            let mut install_path = self.service_path.clone();

            // Create `Launch..` dir if it doesn't already exist.
            if !install_path.exists() {
                fs::create_dir(&install_path)?;
            }

            install_path.push(name);

            if !install_path.exists() {
                fs::copy(&path, &self.service_path)
                    .chain_err(|| "Could not install plist")?;
            }

            Ok(())
        } else {
            Err("Plist path does not contain filename".into())
        }
    }

    #[doc(hidden)]
    pub fn uninstall_plist(&self, name: &str) -> Result<()> {
        let mut path = self.service_path.clone();
        path.push(name);
        path.set_extension("plist");
        if path.exists() {
            fs::remove_file(&path)
                .chain_err(|| "Could not uninstall plist")?;
        }

        Ok(())
    }
}

impl ServiceProvider for Launchctl {
    fn available(telemetry: &Telemetry) -> Result<bool> {
        Ok(telemetry.os.family == OsFamily::Darwin && telemetry.os.version_min >= 11)
    }

    fn running(&self, handle: &Handle, name: &str) -> ExecutableResult {
        let status = match process::Command::new("/bin/launchctl")
            .args(&["blame", &format!("{}/{}", self.domain_target, name)])
            .status_async2(handle)
            .chain_err(|| "Error checking if service is running")
        {
            Ok(s) => s,
            Err(e) => return Box::new(future::err(e)),
        };
        Box::new(status.map(|s| Message::WithoutBody(
                ResponseResult::Ok(
                    Response::Bool(s.success()))))
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("launchctl blame"))))
    }

    fn action(&self, handle: &Handle, name: &str, action: &str) -> ExecutableResult {
        let action = match action {
            "start" => "bootstrap",
            "stop" => "bootout",
            "restart" => "kickstart -k",
            _ => action,
        };

        let cmd = match factory() {
            Ok(c) => c,
            Err(e) => return Box::new(future::ok(
                Message::WithoutBody(
                    ResponseResult::Err(
                        format!("{}", e.display_chain()))))),
        };

        // Run through shell as `action` may contain multiple args with spaces.
        // If we passed `action` as a single argument, it would automatically
        // be quoted and multiple args would appear as a single quoted arg.
        cmd.exec(handle, &[
            "/bin/sh",
            "-c",
            &format!("/bin/launchctl {} {} {}/{}.plist", action, self.domain_target, self.service_path.display(), name)
        ])
    }

    fn enabled(&self, handle: &Handle, name: &str) -> ExecutableResult {
        let name = name.to_owned();

        Box::new(process::Command::new("/bin/launchctl")
            .args(&["print-disabled", &self.domain_target])
            .output_async(handle)
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("launchctl print-disabled <domain_target>")))
            .and_then(move |out| {
                if out.status.success() {
                    let re = match Regex::new(&format!("^\\s+\"{}\" => false", name)) {
                        Ok(r) => r,
                        Err(e) => return future::err(Error::with_chain(e, ErrorKind::Msg("Could not create Launchctl::enabled Regex".into())))
                    };
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let is_match = !re.is_match(&stdout);

                    future::ok(Message::WithoutBody(ResponseResult::Ok(Response::Bool(is_match))))
                } else {
                    future::err(ErrorKind::SystemCommand("/bin/launchctl").into())
                }
            }))
    }

    fn enable(&self, handle: &Handle, name: &str) -> ExecutableResult {
        Box::new(process::Command::new("/bin/launchctl")
            .args(&["enable", &format!("{}/{}", self.domain_target, name)])
            .output_async(handle)
            .map(|out| {
                if out.status.success() {
                    Message::WithoutBody(ResponseResult::Ok(Response::Null))
                } else {
                    Message::WithoutBody(ResponseResult::Err(
                        format!("Could not enable service: {}", String::from_utf8_lossy(&out.stderr))))
                }
            })
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("launchctl enable <service>"))))
    }

    fn disable(&self, handle: &Handle, name: &str) -> ExecutableResult {
        Box::new(process::Command::new("/bin/launchctl")
            .args(&["disable", &format!("{}/{}", self.domain_target, name)])
            .output_async(handle)
            .map(|out| {
                if out.status.success() {
                    Message::WithoutBody(ResponseResult::Ok(Response::Null))
                } else {
                    Message::WithoutBody(ResponseResult::Err(
                        format!("Could not disable service: {}", String::from_utf8_lossy(&out.stderr))))
                }
            })
            .map_err(|e| Error::with_chain(e, ErrorKind::SystemCommand("launchctl disable <service>"))))
    }
}
