// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Package

use {Host, Providers};
use command::ffi::Ffi__CommandResult;
use host::ffi::Ffi__Host;
use libc::{c_char, uint8_t};
use std::{convert, ptr};
use std::ffi::CString;
use super::*;
use super::providers::ProviderFactory;

#[repr(C)]
pub struct Ffi__Package {
    name: *const c_char,
    provider: Ffi__Providers,
    installed: uint8_t,
}

impl convert::From<Package> for Ffi__Package {
    fn from(pkg: Package) -> Ffi__Package {
        Ffi__Package {
            name: CString::new(pkg.name).unwrap().into_raw(),
            provider: Ffi__Providers::from(pkg.provider.get_providers()),
            installed: if pkg.installed { 1 } else { 0 },
        }
    }
}

impl convert::From<Ffi__Package> for Package {
    fn from(ffi_pkg: Ffi__Package) -> Package {
        Package {
            name: ptrtostr!(ffi_pkg.name, "name string").unwrap().into(),
            provider: ProviderFactory::resolve(Option::<Providers>::from(ffi_pkg.provider).unwrap()),
            installed: ffi_pkg.installed == 1,
        }
    }
}

#[repr(C)]
pub enum Ffi__PackageResult {
    Result,
    NoAction,
}

#[repr(C)]
pub enum Ffi__Providers {
    Default,
    Apt,
    Dnf,
    Homebrew,
    Macports,
    Pkg,
    Ports,
    Yum,
}

impl convert::From<Providers> for Ffi__Providers {
    fn from(providers: Providers) -> Ffi__Providers {
        match providers {
            Providers::Apt => Ffi__Providers::Apt,
            Providers::Dnf => Ffi__Providers::Dnf,
            Providers::Homebrew => Ffi__Providers::Homebrew,
            Providers::Macports => Ffi__Providers::Macports,
            Providers::Pkg => Ffi__Providers::Pkg,
            Providers::Ports => Ffi__Providers::Ports,
            Providers::Yum => Ffi__Providers::Yum,
        }
    }
}

impl convert::From<Ffi__Providers> for Option<Providers> {
    fn from(ffi_providers: Ffi__Providers) -> Option<Providers> {
        match ffi_providers {
            Ffi__Providers::Default => None,
            Ffi__Providers::Apt => Some(Providers::Apt),
            Ffi__Providers::Dnf => Some(Providers::Dnf),
            Ffi__Providers::Homebrew => Some(Providers::Homebrew),
            Ffi__Providers::Macports => Some(Providers::Macports),
            Ffi__Providers::Pkg => Some(Providers::Pkg),
            Ffi__Providers::Ports => Some(Providers::Ports),
            Ffi__Providers::Yum => Some(Providers::Yum),
        }
    }
}

#[no_mangle]
pub extern "C" fn package_new(host_ptr: *const Ffi__Host, name_ptr: *const c_char, ffi_providers: Ffi__Providers) -> *mut Ffi__Package {
    let mut host: Host = trynull!(readptr!(host_ptr, "Host struct"));
    let name = trynull!(ptrtostr!(name_ptr, "name string"));
    let providers = Option::<Providers>::from(ffi_providers);

    let result = Ffi__Package::from(trynull!(Package::new(&mut host, name, providers)));

    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn package_is_installed(pkg_ptr: *const Ffi__Package) -> *mut uint8_t {
    let pkg: Package = trynull!(readptr!(pkg_ptr, "Package struct"));
    Box::into_raw(Box::new(if pkg.installed { 1 } else { 0 }))
}

#[no_mangle]
pub extern "C" fn package_install(pkg_ptr: *mut Ffi__Package, host_ptr: *const Ffi__Host, result_ptr: *mut Ffi__CommandResult) -> *mut Ffi__PackageResult {
    let mut pkg: Package = trynull!(readptr!(pkg_ptr, "Package struct"));
    let mut host: Host = trynull!(readptr!(host_ptr, "Host struct"));

    let result = trynull!(pkg.install(&mut host));

    // Write mutated Package state back to pointer
    unsafe { ptr::write(&mut *pkg_ptr, Ffi__Package::from(pkg)); }

    Box::into_raw(Box::new(match result {
        PackageResult::Result(r) => {
            unsafe { ptr::write(&mut *result_ptr, Ffi__CommandResult::from(r)); };
            Ffi__PackageResult::Result
        },
        PackageResult::NoAction => Ffi__PackageResult::NoAction,
    }))
}

#[no_mangle]
pub extern "C" fn package_uninstall(pkg_ptr: *mut Ffi__Package, host_ptr: *const Ffi__Host, result_ptr: *mut Ffi__CommandResult) -> *mut Ffi__PackageResult {
    let mut pkg: Package = trynull!(readptr!(pkg_ptr, "Package struct"));
    let mut host: Host = trynull!(readptr!(host_ptr, "Host struct"));

    let result = trynull!(pkg.uninstall(&mut host));

    // Write mutated Package state back to pointer
    unsafe { ptr::write(&mut *pkg_ptr, Ffi__Package::from(pkg)); }

    Box::into_raw(Box::new(match result {
        PackageResult::Result(r) => {
            unsafe { ptr::write(&mut *result_ptr, Ffi__CommandResult::from(r)); };
            Ffi__PackageResult::Result
        },
        PackageResult::NoAction => Ffi__PackageResult::NoAction,
    }))
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "remote-run")]
    use command::ffi::Ffi__CommandResult;
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    #[cfg(feature = "remote-run")]
    use Host;
    use host::ffi::Ffi__Host;
    #[cfg(feature = "remote-run")]
    use host::ffi::host_close;
    use libc::uint8_t;
    use Package;
    use package::providers::Homebrew;
    use std::ffi::CString;
    #[cfg(feature = "remote-run")]
    use std::ptr;
    use std::str;
    #[cfg(feature = "remote-run")]
    use std::thread;
    use super::*;

    #[test]
    fn test_convert_package() {
        let package = Package {
            name: "whoami".to_string(),
            provider: Box::new(Homebrew),
            installed: true,
        };
        Ffi__Package::from(package);
    }

    #[test]
    fn test_convert_ffi_package() {
        let ffi_pkg = Ffi__Package {
            name: CString::new("nginx").unwrap().into_raw(),
            provider: Ffi__Providers::Homebrew,
            installed: 1,
        };
        Package::from(ffi_pkg);
    }

    #[cfg(feature = "local-run")]
    #[test]
    fn test_package_new_default() {
        let host = Ffi__Host;
        let name = CString::new("nginx").unwrap().into_raw();

        let pkg: Package = readptr!(package_new(&host, name, Ffi__Providers::Default), "Package struct").unwrap();

        assert_eq!(pkg.name, "nginx");
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_new_default() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("Homebrew").unwrap();
            rep.send(&server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("/usr/local/bin/brew").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();
        });

        let mut ffi_host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let name = CString::new("nginx").unwrap().into_raw();
        let pkg: Package = readptr!(package_new(&ffi_host, name, Ffi__Providers::Default), "Package struct").unwrap();
        assert_eq!(pkg.name, "nginx");

        assert_eq!(host_close(&mut ffi_host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "local-run")]
    #[test]
    fn test_package_new_specific() {
        let host = Ffi__Host;
        let name = CString::new("nginx").unwrap().into_raw();
        let mut providers = Ffi__Providers::Default;

        if cfg!(in_os_platform = "centos") {
            providers = Ffi__Providers::Yum;
        }
        if cfg!(in_os_platform = "debian") {
            providers = Ffi__Providers::Apt;
        }
        if cfg!(in_os_platform = "fedora") {
            providers = Ffi__Providers::Dnf;
        }
        if cfg!(in_os_platform = "freebsd") {
            providers = Ffi__Providers::Pkg;
        }
        if cfg!(in_os_platform = "macos") {
            providers = Ffi__Providers::Homebrew;
        }
        if cfg!(in_os_platform = "redhat") {
            providers = Ffi__Providers::Yum;
        }
        if cfg!(in_os_platform = "ubuntu") {
            providers = Ffi__Providers::Apt;
        }

        let pkg: Package = readptr!(package_new(&host, name, providers), "Package struct").unwrap();

        assert_eq!(pkg.name, "nginx");
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_new_homebrew() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("/usr/local/bin/brew").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();
        });

        let mut ffi_host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let name = CString::new("nginx").unwrap().into_raw();
        let pkg: Package = readptr!(package_new(&ffi_host, name, Ffi__Providers::Homebrew), "Package struct").unwrap();
        assert_eq!(pkg.name, "nginx");
        assert!(!pkg.is_installed());

        assert_eq!(host_close(&mut ffi_host), 0);
        agent_mock.join().unwrap();
    }

    #[test]
    fn test_package_is_installed() {
        let pkg = Ffi__Package {
            name: CString::new("nginx").unwrap().into_raw(),
            provider: Ffi__Providers::Homebrew,
            installed: 1,
        };
        let result: uint8_t = readptr!(package_is_installed(&pkg as *const Ffi__Package), "bool").unwrap();
        assert_eq!(result, 1);
    }

    // XXX This requires mocking shell commands
    // #[cfg(feature = "local-run")]
    // #[test]
    // fn test_package_install() {
    //     let host = Ffi__Host;
    //     let pkg = Ffi__Package {
    //         name: CString::new("nginx").unwrap().as_ptr(),
    //         provider: Box::into_raw(Box::new(Homebrew)),
    //         installed: 0,
    //     };
    //     let result = package_install(&pkg as *const Ffi__Package, &host as *const Ffi__Host);
    //     assert_eq!(result.exit_code, 0);
    // }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_install() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();
        });

        let mut ffi_host = Ffi__Host::from(Host::test_new(None, Some(client), None));
        let mut ffi_pkg = Ffi__Package {
            name: CString::new("nginx").unwrap().into_raw(),
            provider: Ffi__Providers::Homebrew,
            installed: 0,
        };

        let mut result = Ffi__CommandResult {
            exit_code: 0,
            stdout: CString::new("").unwrap().into_raw(),
            stderr: CString::new("").unwrap().into_raw(),
        };
        let action = package_install(&mut ffi_pkg, &ffi_host, &mut result);
        assert!(!action.is_null());

        match unsafe { ptr::read(action) } {
            Ffi__PackageResult::Result => assert_eq!(result.exit_code, 0),
            _ => panic!("Package install not attempted"),
        }

        assert_eq!(host_close(&mut ffi_host), 0);
        agent_mock.join().unwrap();
    }

    // XXX This requires mocking shell commands
    // #[cfg(feature = "local-run")]
    // #[test]
    // fn test_package_uninstall() {
    //     let host = Ffi__Host;
    //     let pkg = Ffi__Package {
    //         name: CString::new("nginx").unwrap().as_ptr(),
    //         provider: Ffi__Provider {
    //             provider: Box::into_raw(Box::new(Homebrew)),
    //         },
    //     };
    //     let result = package_uninstall(&pkg as *const Ffi__Package, &host as *const Ffi__Host);
    //     assert_eq!(result.exit_code, 0);
    // }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_uninstall() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();
        });

        let mut ffi_host = Ffi__Host::from(Host::test_new(None, Some(client), None));
        let mut ffi_pkg = Ffi__Package {
            name: CString::new("nginx").unwrap().into_raw(),
            provider: Ffi__Providers::Homebrew,
            installed: 1,
        };

        let mut result = Ffi__CommandResult {
            exit_code: 0,
            stdout: CString::new("").unwrap().into_raw(),
            stderr: CString::new("").unwrap().into_raw(),
        };
        let action = package_uninstall(&mut ffi_pkg, &ffi_host, &mut result);
        assert!(!action.is_null());

        match unsafe { ptr::read(action) } {
            Ffi__PackageResult::Result => assert_eq!(result.exit_code, 0),
            _ => panic!("Package uninstall not attempted"),
        }

        assert_eq!(host_close(&mut ffi_host), 0);
        agent_mock.join().unwrap();
    }
}
