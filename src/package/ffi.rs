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
    name: *const c_char,
    provider: Ffi__Provider,
    installed: uint8_t,
}

impl convert::From<Package> for Ffi__Package {
    fn from(pkg: Package) -> Ffi__Package {
        Ffi__Package {
            name: CString::new(pkg.name).unwrap().into_raw(),
            provider: Ffi__Provider::from(pkg.provider),
            installed: if pkg.installed { 1 } else { 0 },
        }
    }
}

impl convert::From<Ffi__Package> for Package {
    fn from(ffi_pkg: Ffi__Package) -> Package {
        let slice = unsafe { CStr::from_ptr(ffi_pkg.name) };
        let name_str = str::from_utf8(slice.to_bytes()).unwrap();

        Package {
            name: name_str.to_string(),
            provider: convert_ffi_provider(ffi_pkg.provider),
            installed: if ffi_pkg.installed == 1 { true } else { false },
        }
    }
}

#[repr(C)]
pub struct Ffi__Provider {
    provider: *mut Provider,
}

impl convert::From<Box<Provider + 'static>> for Ffi__Provider {
    fn from(provider: Box<Provider + 'static>) -> Ffi__Provider {
        Ffi__Provider {
            provider: Box::into_raw(provider),
        }
    }
}

fn convert_ffi_provider(ffi_provider: Ffi__Provider) -> Box<Provider + 'static> {
    unsafe { Box::from_raw(ffi_provider.provider) }
}

#[no_mangle]
pub extern "C" fn package_new(ffi_host_ptr: *const Ffi__Host, name_ptr: *const c_char, provider_ptr: *const c_char) -> Ffi__Package {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let name = unsafe { str::from_utf8(CStr::from_ptr(name_ptr).to_bytes()).unwrap() };
    let provider = unsafe { str::from_utf8(CStr::from_ptr(provider_ptr).to_bytes()).unwrap() };

    let p_arg: Option<Providers>;
    if provider == "Default" || provider == "default" {
        p_arg = None;
    } else {
        p_arg = Some(Providers::from(provider.to_string()));
    }

    let result = Ffi__Package::from(Package::new(&mut host, name, p_arg).unwrap());

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
pub extern "C" fn package_install(ffi_pkg_ptr: *const Ffi__Package, ffi_host_ptr: *const Ffi__Host) -> Ffi__CommandResult {
    let pkg = Package::from(unsafe { ptr::read(ffi_pkg_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = Ffi__CommandResult::from(pkg.install(&mut host).unwrap());

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    result
}

#[no_mangle]
pub extern "C" fn package_uninstall(ffi_pkg_ptr: *const Ffi__Package, ffi_host_ptr: *const Ffi__Host) -> Ffi__CommandResult {
    let pkg = Package::from(unsafe { ptr::read(ffi_pkg_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = Ffi__CommandResult::from(pkg.uninstall(&mut host).unwrap());

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    result
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "remote-run")]
    use Host;
    use Package;
    use host::ffi::Ffi__Host;
    use libc::uint8_t;
    use package::providers::Homebrew;
    use std::str;
    #[cfg(feature = "remote-run")]
    use std::thread;
    use std::ffi::{CStr, CString};
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
            name: CString::new("nginx").unwrap().as_ptr(),
            provider: Ffi__Provider {
                provider: Box::into_raw(Box::new(Homebrew)),
            },
            installed: 1 as uint8_t,
        };
        Package::from(ffi_pkg);
    }

    #[cfg(feature = "local-run")]
    #[test]
    fn test_package_new_default() {
        let host = Ffi__Host;
        let name = CString::new("nginx").unwrap().as_ptr();
        let provider = CString::new("default").unwrap().as_ptr();
        let ffi_pkg = package_new(&host as *const Ffi__Host, name, provider);
        assert_eq!(ffi_pkg.name, name);
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
            agent_sock.close().unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test_package_new_default").unwrap();

        let ffi_host = Ffi__Host::from(Host::test_new(sock));

        let name = CString::new("nginx").unwrap().as_ptr();
        let provider = CString::new("default").unwrap().as_ptr();
        let ffi_pkg = package_new(&ffi_host as *const Ffi__Host, name, provider);

        assert_eq!(ffi_pkg.name, name);

        Host::from(ffi_host);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "local-run")]
    #[test]
    fn test_package_new_homebrew() {
        let host = Ffi__Host;
        let name = CString::new("nginx").unwrap().as_ptr();
        let provider = CString::new("Homebrew").unwrap().as_ptr();
        let ffi_pkg = package_new(&host as *const Ffi__Host, name, provider);
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_pkg.name).to_bytes()).unwrap() }, unsafe { str::from_utf8(CStr::from_ptr(name).to_bytes()).unwrap() });
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_package_new_homebrew() {
        let host = Ffi__Host::from(Host::new());
        let name = CString::new("nginx").unwrap().as_ptr();
        let provider = CString::new("Homebrew").unwrap().as_ptr();
        let ffi_pkg = package_new(&host as *const Ffi__Host, name, provider);
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_pkg.name).to_bytes()).unwrap() }, unsafe { str::from_utf8(CStr::from_ptr(name).to_bytes()).unwrap() });
    }

    #[test]
    fn test_package_is_installed() {
        let pkg = Ffi__Package {
            name: CString::new("nginx").unwrap().as_ptr(),
            provider: Ffi__Provider {
                provider: Box::into_raw(Box::new(Homebrew)),
            },
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
            assert_eq!("which brew", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("/usr/local/bin/brew", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();

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

        let name = CString::new("nginx").unwrap().into_raw();
        let provider = CString::new("Homebrew").unwrap().into_raw();

        let ffi_pkg = package_new(&ffi_host as *const Ffi__Host, name, provider);
        let result = package_install(&ffi_pkg as *const Ffi__Package, &ffi_host as *const Ffi__Host);

        assert_eq!(result.exit_code, 0);

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

        let name = CString::new("nginx").unwrap().into_raw();
        let provider = CString::new("Homebrew").unwrap().into_raw();

        let ffi_pkg = package_new(&ffi_host as *const Ffi__Host, name, provider);
        let result = package_uninstall(&ffi_pkg as *const Ffi__Package, &ffi_host as *const Ffi__Host);

        assert_eq!(result.exit_code, 0);

        Host::from(ffi_host);

        agent_mock.join().unwrap();
    }
}
