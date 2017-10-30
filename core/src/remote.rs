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
    CommandExec(Option<ProviderName>, String, Vec<String>),
    PackageInstalled(Option<ProviderName>, String),
    PackageInstall(Option<ProviderName>, String),
    PackageUninstall(Option<ProviderName>, String),
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

#[derive(Serialize, Deserialize)]
pub enum ProviderName {
    CommandGeneric,
    PackageApt,
    PackageDnf,
    PackageHomebrew,
    PackageNix,
    PackagePkg,
    PackageYum,
    TelemetryCentos,
    TelemetryDebian,
    TelemetryFedora,
    TelemetryFreebsd,
    TelemetryMacos,
    TelemetryNixos,
    TelemetryUbuntu,
}

pub trait Executable {
    fn exec<H: Host>(self, &H) -> ExecutableResult;
}

impl Executable for Request {
    fn exec<H: Host>(self, host: &H) -> ExecutableResult {
        match self {
            Request::CommandExec(provider, cmd, shell) => {
                let provider = match provider {
                    Some(ProviderName::CommandGeneric) => Box::new(command::providers::Generic),
                    None => match command::providers::factory() {
                        Ok(p) => p,
                        Err(e) => return Box::new(future::err(e)),
                    },
                    _ => unreachable!(),
                };
                provider.exec(host.handle(), &cmd, &shell)
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
                let provider = match telemetry::providers::factory() {
                    Ok(p) => p,
                    Err(e) => return Box::new(future::err(e)),
                };
                provider.load()
            }
        }
    }
}

fn get_package_provider(name: Option<ProviderName>) -> Result<Box<package::providers::PackageProvider>> {
    match name {
        Some(ProviderName::PackageApt) => Ok(Box::new(package::providers::Apt)),
        Some(ProviderName::PackageDnf) => Ok(Box::new(package::providers::Dnf)),
        Some(ProviderName::PackageHomebrew) => Ok(Box::new(package::providers::Homebrew)),
        Some(ProviderName::PackageNix) => Ok(Box::new(package::providers::Nix)),
        Some(ProviderName::PackagePkg) => Ok(Box::new(package::providers::Pkg)),
        Some(ProviderName::PackageYum) => Ok(Box::new(package::providers::Yum)),
        None => package::providers::factory(),
        _ => unreachable!(),
    }
}
