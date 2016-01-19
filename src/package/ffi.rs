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
use std::ffi::{CStr, CString};
use super::*;
use super::providers::{Provider, ProviderFactory};

#[repr(C)]
pub struct Ffi__Package {
    name: *mut c_char,
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
            name: unsafe { str::from_utf8(CStr::from_ptr(ffi_pkg.name).to_bytes()).unwrap().to_string() },
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
    let pkg = unsafe { ptr::read(ffi_pkg_ptr) };
    pkg.installed
}

#[no_mangle]
pub extern "C" fn package_install(ffi_pkg_ptr: *mut Ffi__Package, ffi_host_ptr: *const Ffi__Host, ffi_result_ptr: *mut Ffi__CommandResult) -> Ffi__PackageResult {
    let mut pkg = Package::from(unsafe { ptr::read(ffi_pkg_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = pkg.install(&mut host).unwrap();

    // Write mutated Package state back to pointer
    unsafe { ptr::write(&mut *ffi_pkg_ptr, Ffi__Package::from(pkg)); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    match result {
        PackageResult::Result(r) => {
            unsafe { ptr::write(&mut *ffi_result_ptr, Ffi__CommandResult::from(r)); };
            Ffi__PackageResult::Result
        },
        PackageResult::NoAction => Ffi__PackageResult::NoAction,
    }
}

#[no_mangle]
pub extern "C" fn package_uninstall(ffi_pkg_ptr: *mut Ffi__Package, ffi_host_ptr: *const Ffi__Host, ffi_result_ptr: *mut Ffi__CommandResult) -> Ffi__PackageResult {
    let mut pkg = Package::from(unsafe { ptr::read(ffi_pkg_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = pkg.uninstall(&mut host).unwrap();

    // Write mutated Package state back to pointer
    unsafe { ptr::write(&mut *ffi_pkg_ptr, Ffi__Package::from(pkg)); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    match result {
        PackageResult::Result(r) => {
            unsafe { ptr::write(&mut *ffi_result_ptr, Ffi__CommandResult::from(r)); };
            Ffi__PackageResult::Result
        },
        PackageResult::NoAction => Ffi__PackageResult::NoAction,
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "remote-run")]
    use command::ffi::Ffi__CommandResult;
    #[cfg(feature = "remote-run")]
    use Host;
    use host::ffi::Ffi__Host;
    use Package;
    use package::providers::Homebrew;
    use std::ffi::{CStr, CString};
    use std::str;
    #[cfg(feature = "remote-run")]
    use std::thread;
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

        let ffi_pkg = package_new(&host as *const Ffi__Host, name, Ffi__Providers::Default);

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

        let ffi_pkg = package_new(&host as *const Ffi__Host, name, providers);

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
            provider: Ffi__Providers::Homebrew,
            installed: 1,
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
    //         provider: Box::into_raw(Box::new(Homebrew)),
    //         installed: 0,
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
        let action = package_install(&mut ffi_pkg as *mut Ffi__Package, &ffi_host as *const Ffi__Host, &mut result as *mut Ffi__CommandResult);

        match action {
            Ffi__PackageResult::Result => assert_eq!(result.exit_code, 0),
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
        let action = package_uninstall(&mut ffi_pkg as *mut Ffi__Package, &ffi_host as *const Ffi__Host, &mut result as *mut Ffi__CommandResult);

        match action {
            Ffi__PackageResult::Result => assert_eq!(result.exit_code, 0),
            _ => panic!("Package uninstall not attempted"),
        }

        Host::from(ffi_host);

        agent_mock.join().unwrap();
    }
}
