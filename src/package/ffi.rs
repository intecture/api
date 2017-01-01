// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Package

use command::ffi::Ffi__CommandResult;
use ffi_helpers::Leaky;
use host::Host;
use libc::{c_char, int8_t, uint8_t};
use package::providers::Providers;
use std::{convert, ptr};
use std::panic::catch_unwind;
use super::*;

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

impl convert::Into<Option<Providers>> for Ffi__Providers {
    fn into(self) -> Option<Providers> {
        match self {
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
pub extern "C" fn package_new(host_ptr: *const Host, name_ptr: *const c_char, ffi_providers: Ffi__Providers) -> *mut Package {
    let mut host = Leaky::new(trynull!(readptr!(host_ptr, "Host pointer")));
    let name = trynull!(ptrtostr!(name_ptr, "name string"));
    let providers: Option<Providers> = ffi_providers.into();

    let pkg = trynull!(Package::new(&mut host, name, providers));
    Box::into_raw(Box::new(pkg))
}

#[no_mangle]
pub extern "C" fn package_is_installed(pkg_ptr: *const Package) -> int8_t {
    let pkg = Leaky::new(tryrc!(readptr!(pkg_ptr, "Package pointer"), -1));
    if pkg.installed {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn package_install(pkg_ptr: *mut Package, host_ptr: *const Host) -> *mut Ffi__CommandResult {
    let mut pkg = Leaky::new(trynull!(boxptr!(pkg_ptr, "Package pointer")));
    let mut host = Leaky::new(trynull!(readptr!(host_ptr, "Host pointer")));

    let result = trynull!(pkg.install(&mut host));
    match result {
        Some(r) => {
            let ffi_r: Ffi__CommandResult = trynull!(catch_unwind(|| r.into()));
            Box::into_raw(Box::new(ffi_r))
        },
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn package_uninstall(pkg_ptr: *mut Package, host_ptr: *const Host) -> *mut Ffi__CommandResult {
    let mut pkg = Leaky::new(trynull!(boxptr!(pkg_ptr, "Package pointer")));
    let mut host = Leaky::new(trynull!(readptr!(host_ptr, "Host pointer")));

    let result = trynull!(pkg.uninstall(&mut host));
    match result {
        Some(r) => {
            let ffi_r: Ffi__CommandResult = trynull!(catch_unwind(|| r.into()));
            Box::into_raw(Box::new(ffi_r))
        },
        None => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn package_free(pkg_ptr: *mut Package) -> uint8_t {
    tryrc!(boxptr!(pkg_ptr, "Package pointer"));
    0
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    use Host;
    #[cfg(feature = "remote-run")]
    use host::ffi::host_close;
    use std::ffi::CString;
    use std::str;
    #[cfg(feature = "remote-run")]
    use std::thread;
    use super::*;

    #[cfg(feature = "local-run")]
    #[test]
    fn test_package_new_default() {
        let path: Option<String> = None;
        let host = Host::local(path).unwrap();
        let name = CString::new("nginx").unwrap().into_raw();

        let pkg = readptr!(package_new(&host, name, Ffi__Providers::Default), "Package pointer").unwrap();
        assert_eq!(pkg.name, "nginx");
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_new_default() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("Homebrew").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("/usr/local/bin/brew").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let name = CString::new("nginx").unwrap().into_raw();
        let pkg = readptr!(package_new(host, name, Ffi__Providers::Default), "Package pointer").unwrap();
        assert_eq!(pkg.name, "nginx");

        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_new_homebrew() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("/usr/local/bin/brew").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let name = CString::new("nginx").unwrap().into_raw();
        let pkg = readptr!(package_new(host, name, Ffi__Providers::Homebrew), "Package pointer").unwrap();
        assert_eq!(pkg.name, "nginx");
        assert!(!pkg.is_installed());

        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "local-run")]
    #[test]
    fn test_package_is_installed() {
        let path: Option<String> = None;
        let host = Box::into_raw(Box::new(Host::local(path).unwrap()));

        let name = CString::new("thisisafakepackage123abc").unwrap().into_raw();
        let pkg = package_new(host, name, Ffi__Providers::Default);
        assert_eq!(package_is_installed(pkg), 0);
        assert_eq!(package_free(pkg), 0);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_install() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("/usr/local/bin/brew").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!(req.popstr().unwrap().unwrap(), "command::exec");
            assert_eq!(req.popstr().unwrap().unwrap(), "brew install nginx");

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let name = CString::new("nginx").unwrap().into_raw();
        let pkg = package_new(host, name, Ffi__Providers::Homebrew);
        assert!(!pkg.is_null());

        let result = readptr!(package_install(pkg, host), "CommandResult pointer").unwrap();
        assert_eq!(result.exit_code, 0);

        let p = readptr!(pkg, "Package pointer").unwrap();
        assert!(p.installed);

        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_uninstall() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("/usr/local/bin/brew").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("nginx ").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!(req.popstr().unwrap().unwrap(), "command::exec");
            assert_eq!(req.popstr().unwrap().unwrap(), "brew uninstall nginx");

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let name = CString::new("nginx").unwrap().into_raw();
        let pkg = package_new(host, name, Ffi__Providers::Homebrew);
        assert!(!pkg.is_null());

        let result = readptr!(package_uninstall(pkg, host), "CommandResult pointer").unwrap();
        assert_eq!(result.exit_code, 0);

        let p = readptr!(pkg, "Package pointer").unwrap();
        assert!(!p.installed);

        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }
}
