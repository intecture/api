// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

// Hopefully in the near future this will be auto-generated.

use command;
use errors::*;
use futures::{future, Future};
use host::Host;
use package;
use std::io;
use telemetry;
use tokio_proto::streaming::{Body, Message};

pub type ExecutableResult = Box<Future<Item = Message<ResponseResult, Body<Vec<u8>, io::Error>>, Error = Error>>;

#[derive(Serialize, Deserialize)]
pub enum Request {
    CommandExec(Option<command::Provider>, Vec<String>),
    PackageInstalled(Option<package::Provider>, String),
    PackageInstall(Option<package::Provider>, String),
    PackageUninstall(Option<package::Provider>, String),
    TelemetryLoad,
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Bool(bool),
    Null,
    TelemetryLoad(telemetry::serializable::Telemetry),
}

#[derive(Serialize, Deserialize)]
pub enum ResponseResult {
    Ok(Response),
    Err(String),
}

pub trait Executable {
    fn exec<H: Host>(self, &H) -> ExecutableResult;
}

impl Executable for Request {
    fn exec<H: Host>(self, host: &H) -> ExecutableResult {
        match self {
            Request::CommandExec(provider, cmd) => {
                let provider = match provider {
                    Some(command::Provider::Generic) => Box::new(command::Generic),
                    None => match command::factory() {
                        Ok(p) => p,
                        Err(e) => return Box::new(future::err(e)),
                    },
                };
                let args: Vec<&str> = cmd.iter().map(|a| &**a).collect();
                provider.exec(host.handle(), &args)
            }

            Request::PackageInstalled(provider, name) => {
                let provider = match get_package_provider(provider) {
                    Ok(p) => p,
                    Err(e) => return Box::new(future::err(e)),
                };
                provider.installed(host.handle(), &name, &host.telemetry().os)
            }

            Request::PackageInstall(provider, name) => {
                let provider = match get_package_provider(provider) {
                    Ok(p) => p,
                    Err(e) => return Box::new(future::err(e)),
                };
                provider.install(host.handle(), &name)
            }

            Request::PackageUninstall(provider, name) => {
                let provider = match get_package_provider(provider) {
                    Ok(p) => p,
                    Err(e) => return Box::new(future::err(e)),
                };
                provider.uninstall(host.handle(), &name)
            }

            Request::TelemetryLoad => {
                let provider = match telemetry::factory() {
                    Ok(p) => p,
                    Err(e) => return Box::new(future::err(e)),
                };
                provider.load()
            }
        }
    }
}

fn get_package_provider(name: Option<package::Provider>) -> Result<Box<package::PackageProvider>> {
    match name {
        Some(package::Provider::Apt) => Ok(Box::new(package::Apt)),
        Some(package::Provider::Dnf) => Ok(Box::new(package::Dnf)),
        Some(package::Provider::Homebrew) => Ok(Box::new(package::Homebrew)),
        Some(package::Provider::Nix) => Ok(Box::new(package::Nix)),
        Some(package::Provider::Pkg) => Ok(Box::new(package::Pkg)),
        Some(package::Provider::Yum) => Ok(Box::new(package::Yum)),
        None => package::factory(),
    }
}
