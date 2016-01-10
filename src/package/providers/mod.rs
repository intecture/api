// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

pub mod apt;
pub mod dnf;
pub mod homebrew;
pub mod macports;
pub mod pkg;
pub mod ports;
pub mod yum;

use {CommandResult, Error, Host, Result};
pub use self::homebrew::Homebrew;
use std::convert;
use std::string::ToString;
use super::PackageTarget;
use target::Target;

pub enum Providers {
    Apt,
    Dnf,
    Homebrew,
    Macports,
    Pkg,
    Ports,
    Yum,
}

impl ToString for Providers {
    fn to_string(&self) -> String {
        match self {
            &Providers::Apt => "Apt".to_string(),
            &Providers::Dnf => "Dnf".to_string(),
            &Providers::Homebrew => "Homebrew".to_string(),
            &Providers::Macports => "Macports".to_string(),
            &Providers::Pkg => "Pkg".to_string(),
            &Providers::Ports => "Ports".to_string(),
            &Providers::Yum => "Yum".to_string(),
        }
    }
}

impl convert::From<String> for Providers {
    fn from(provider: String) -> Providers {
        match provider.as_ref() {
            "Apt" => Providers::Apt,
            "Dnf" => Providers::Dnf,
            "Homebrew" => Providers::Homebrew,
            "Macports" => Providers::Macports,
            "Pkg" => Providers::Pkg,
            "Ports" => Providers::Ports,
            "Yum" => Providers::Yum,
            _ => panic!("Invalid provider"),
        }
    }
}

pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(host: &mut Host, providers: Option<Providers>) -> Result<Box<Provider + 'static>> {
        let provider: Box<Provider + 'static>;

        if let Some(p) = providers {
            provider = match p {
                Providers::Apt => Box::new(apt::Apt),
                Providers::Dnf => Box::new(dnf::Dnf),
                Providers::Homebrew => Box::new(homebrew::Homebrew),
                Providers::Macports => Box::new(macports::Macports),
                Providers::Pkg => Box::new(pkg::Pkg),
                Providers::Ports => Box::new(ports::Ports),
                Providers::Yum => Box::new(yum::Yum),
            };
        } else {
            provider = try!(Target::default_provider(host));
        }

        if try!(provider.is_active(host)) {
            Ok(provider)
        } else {
            Err(Error::Generic("Provider is not active".to_string()))
        }
    }
}

pub trait Provider {
    fn get_providers(&self) -> Providers;
    fn is_active(&self, host: &mut Host) -> Result<bool>;
    fn is_installed(&self, host: &mut Host, name: &str) -> Result<bool>;
    fn install(&self, host: &mut Host, name: &str) -> Result<CommandResult>;
    fn uninstall(&self, host: &mut Host, name: &str) -> Result<CommandResult>;
}
