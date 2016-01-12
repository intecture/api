// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
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
use std::{convert, ptr, str};
use std::boxed::Box;
use std::ffi::{CStr, CString};
use super::*;
use super::providers::Provider;

#[repr(C)]
pub struct Ffi__Package {
    name: *mut c_char,
    provider: *mut Provider,
    installed: uint8_t,
}

impl convert::From<Package> for Ffi__Package {
    fn from(pkg: Package) -> Ffi__Package {
        Ffi__Package {
            name: CString::new(pkg.name).unwrap().into_raw(),
            provider: Box::into_raw(pkg.provider),
            installed: if pkg.installed { 1 } else { 0 },
        }
    }
}

impl convert::From<Ffi__Package> for Package {
    fn from(ffi_pkg: Ffi__Package) -> Package {
        Package {
            name: unsafe { str::from_utf8(CStr::from_ptr(ffi_pkg.name).to_bytes()).unwrap().to_string() },
            provider: unsafe { Box::from_raw(ffi_pkg.provider) },
            installed: if ffi_pkg.installed == 1 { true } else { false },
        }
    }
}

#[repr(C)]
pub enum Ffi__PackageResult {
    Result,
    NoAction,
}

impl convert::From<PackageResult> for Ffi__PackageResult {
    fn from(result: PackageResult) -> Ffi__PackageResult {
        match result {
            PackageResult::Result(r) => Ffi__PackageResult::Result,
            PackageResult::NoAction => Ffi__PackageResult::NoAction,
        }
    }
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
pub extern "C" fn package_new(ffi_host_ptr: *const Ffi__Host, name_ptr: *const c_char, ffi_providers: Ffi__Providers) -> Ffi__Package {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let name = unsafe { str::from_utf8(CStr::from_ptr(name_ptr).to_bytes()).unwrap() };
    let providers = Option::<Providers>::from(ffi_providers);

    let result = Ffi__Package::from(Package::new(&mut host, name, providers).unwrap());

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    result
}

#[no_mangle]
pub extern "C" fn package_is_installed(ffi_pkg_ptr: *const Ffi__Package) -> uint8_t {
    let pkg = Package::from(unsafe { ptr::read(ffi_pkg_ptr) });
    if pkg.installed { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn package_install(ffi_pkg_ptr: *mut Ffi__Package, ffi_host_ptr: *const Ffi__Host, ffi_result_ptr: *mut Ffi__CommandResult) -> Ffi__PackageResult {
    let mut pkg = Package::from(unsafe { ptr::read(ffi_pkg_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = pkg.install(&mut host).unwrap();

    match result {
        PackageResult::Result(r) => unsafe { ptr::write(&mut *ffi_result_ptr, Ffi__CommandResult::from(r)); },
        _ => (),
    }

    unsafe { ptr::write(&mut *ffi_pkg_ptr, Ffi__Package::from(pkg)); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    Ffi__PackageResult::from(result)
}

#[no_mangle]
pub extern "C" fn package_uninstall(ffi_pkg_ptr: *mut Ffi__Package, ffi_host_ptr: *const Ffi__Host, ffi_result_ptr: *mut Ffi__CommandResult) -> Ffi__PackageResult {
    let mut pkg = Package::from(unsafe { ptr::read(ffi_pkg_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = pkg.uninstall(&mut host).unwrap();

    match result {
        PackageResult::Result(r) => unsafe { ptr::write(&mut *ffi_result_ptr, Ffi__CommandResult::from(r)); },
        _ => (),
    }

    unsafe { ptr::write(&mut *ffi_pkg_ptr, Ffi__Package::from(pkg)); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    Ffi__PackageResult::from(result)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "remote-run")]
    use Host;
    use Package;
    use host::ffi::Ffi__Host;
    use libc::uint8_t;
    use package::providers::Homebrew;
    use std::ffi::{CStr, CString};
    #[cfg(feature = "remote-run")]
    use std::thread;
    use std::str;
    use super::*;
    #[cfg(feature = "remote-run")]
    use zmq;

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
            provider: Box::into_raw(Box::new(Homebrew)),
            installed: 1 as uint8_t,
        };
        Package::from(ffi_pkg);
    }

    #[cfg(feature = "local-run")]
    #[test]
    fn test_package_new_default() {
        let host = Ffi__Host;
        let name = CString::new("nginx").unwrap().into_raw();
        let provider = CString::new("default").unwrap().into_raw();

        let ffi_pkg = package_new(&host as *const Ffi__Host, name, provider);

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_pkg.name).to_bytes()).unwrap() }, "nginx");
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_new_default() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_package_new_default").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("package::default_provider", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Homebrew", 0).unwrap();

            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("which brew", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("/usr/local/bin/brew", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();

            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("brew list | grep nginx", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test_package_new_default").unwrap();

        let ffi_host = Ffi__Host::from(Host::test_new(sock));

        let name = CString::new("nginx").unwrap().into_raw();

        let ffi_pkg = package_new(&ffi_host as *const Ffi__Host, name, Ffi__Providers::Default);

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_pkg.name).to_bytes()).unwrap() }, "nginx");

        Host::from(ffi_host);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "local-run")]
    #[test]
    fn test_package_new_homebrew() {
        let host = Ffi__Host;
        let name = CString::new("nginx").unwrap().into_raw();
        let provider = CString::new("Homebrew").unwrap().into_raw();

        let ffi_pkg = package_new(&host as *const Ffi__Host, name, provider);

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_pkg.name).to_bytes()).unwrap() }, "nginx");
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_new_homebrew() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_package_new_homebrew").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("which brew", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("/usr/local/bin/brew", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();

            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("brew list | grep nginx", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test_package_new_homebrew").unwrap();

        let ffi_host = Ffi__Host::from(Host::test_new(sock));
        let name = CString::new("nginx").unwrap().into_raw();

        let ffi_pkg = package_new(&ffi_host as *const Ffi__Host, name, Ffi__Providers::Homebrew);
        let pkg = Package::from(ffi_pkg);

        assert_eq!(pkg.name, "nginx");
        assert!(!pkg.is_installed());

        Host::from(ffi_host);

        agent_mock.join().unwrap();
    }

    #[test]
    fn test_package_is_installed() {
        let pkg = Ffi__Package {
            name: CString::new("nginx").unwrap().into_raw(),
            provider: Box::into_raw(Box::new(Homebrew)),
            installed: 1 as uint8_t,
        };
        let result = package_is_installed(&pkg as *const Ffi__Package);
        assert_eq!(result, 1);
    }

    // XXX This requires mocking shell commands
    // #[cfg(feature = "local-run")]
    // #[test]
    // fn test_package_install() {
    //     let host = Ffi__Host;
    //     let pkg = Ffi__Package {
    //         name: CString::new("nginx").unwrap().as_ptr(),
    //         provider: Ffi__Provider {
    //             provider: Box::into_raw(Box::new(Homebrew)),
    //         },
    //     };
    //     let result = package_install(&pkg as *const Ffi__Package, &host as *const Ffi__Host);
    //     assert_eq!(result.exit_code, 0);
    // }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_install() {
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_package_install").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("brew install nginx", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();
            agent_sock.close().unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test_package_install").unwrap();

        let ffi_host = Ffi__Host::from(Host::test_new(sock));
        let ffi_pkg = Ffi__Package {
            name: CString::new("nginx").unwrap().into_raw(),
            provider: Box::into_raw(Box::new(Homebrew)),
            installed: 0,
        };

        let result = package_install(&ffi_pkg as *const Ffi__Package, &ffi_host as *const Ffi__Host);

        match result {
            Ffi__PackageResult::Result(r) => assert_eq!(r.exit_code, 0),
            _ => panic!("Package install not attempted"),
        }

        Host::from(ffi_host);

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
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_package_uninstall").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("brew uninstall nginx", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();
            agent_sock.close().unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test_package_uninstall").unwrap();

        let ffi_host = Ffi__Host::from(Host::test_new(sock));
        let ffi_pkg = Ffi__Package {
            name: CString::new("nginx").unwrap().into_raw(),
            provider: Box::into_raw(Box::new(Homebrew)),
            installed: 1,
        };

        let result = package_uninstall(&ffi_pkg as *const Ffi__Package, &ffi_host as *const Ffi__Host);

        match result {
            Ffi__PackageResult::Result(r) => assert_eq!(r.exit_code, 0),
            _ => panic!("Package uninstall not attempted"),
        }

        Host::from(ffi_host);

        agent_mock.join().unwrap();
    }
}
