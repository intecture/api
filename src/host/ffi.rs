// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Host

#[cfg(feature = "remote-run")]
use czmq::{RawInterface, ZSock};
#[cfg(feature = "remote-run")]
use libc::{c_char, uint32_t};
use std::convert;
#[cfg(feature = "remote-run")]
use std::ptr;
#[cfg(feature = "remote-run")]
use std::ffi::{CStr, CString};
#[cfg(feature = "remote-run")]
use std::os::raw::c_void;
use super::*;

#[cfg(feature = "local-run")]
#[repr(C)]
pub struct Ffi__Host;

#[cfg(feature = "remote-run")]
#[repr(C)]
#[derive(Debug)]
pub struct Ffi__Host {
    hostname: Option<*mut c_char>,
    api_sock: Option<*mut c_void>,
    file_sock: Option<*mut c_void>,
}

impl convert::From<Host> for Ffi__Host {
    #[cfg(feature = "local-run")]
    #[allow(unused_variables)]
    fn from(host: Host) -> Ffi__Host {
        Ffi__Host
    }

    #[cfg(feature = "remote-run")]
    fn from(host: Host) -> Ffi__Host {
        Ffi__Host {
            hostname: match host.hostname {
                Some(hostname) => Some(CString::new(hostname).unwrap().into_raw()),
                None => None,
            },
            api_sock: match host.api_sock {
                Some(sock) => Some(sock.into_raw()),
                None => None,
            },
            file_sock: match host.file_sock {
                Some(sock) => Some(sock.into_raw()),
                None => None,
            },
        }
    }
}

impl convert::From<Ffi__Host> for Host {
    #[cfg(feature = "local-run")]
    fn from(_: Ffi__Host) -> Host {
        Host
    }

    #[cfg(feature = "remote-run")]
    fn from(ffi_host: Ffi__Host) -> Host {
        Host {
            hostname: match ffi_host.hostname {
                Some(ptr) => Some(unsafe { CStr::from_ptr(ptr) }.to_str().unwrap().into()),
                None => None,
            },
            api_sock: match ffi_host.api_sock {
                Some(sock) => Some(ZSock::from_raw(sock, true)),
                None => None,
            },
            file_sock: match ffi_host.file_sock {
                Some(sock) => Some(ZSock::from_raw(sock, true)),
                None => None,
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn host_new() -> Ffi__Host {
    Ffi__Host::from(Host::new())
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_connect(ffi_host_ptr: *mut Ffi__Host,
                               hostname_ptr: *const c_char,
                               api_port: uint32_t,
                               upload_port: uint32_t,
                               auth_server_ptr: *const c_char) {

    let hostname = unsafe { CStr::from_ptr(hostname_ptr) }.to_str().unwrap();
    let auth_server = unsafe { CStr::from_ptr(auth_server_ptr) }.to_str().unwrap();

    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    host.connect(hostname, api_port, upload_port, auth_server).unwrap();

    unsafe { ptr::write(&mut *ffi_host_ptr, Ffi__Host::from(host)); }
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_close(ffi_host_ptr: *mut Ffi__Host) {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    host.close().unwrap();
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "remote-run")]
    use {create_project_fs, mock_auth_server};
    #[cfg(feature = "remote-run")]
    use czmq::ZSys;
    use Host;
    #[cfg(feature = "remote-run")]
    use std::ffi::CString;
    use super::*;

    #[test]
    fn test_convert_host() {
        let host = Host::new();
        let ffi = Ffi__Host::from(host);
        Host::from(ffi);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_convert_host_connected() {
        ZSys::init();

        create_project_fs();
        let (handle, auth_server) = mock_auth_server();

        let mut host = Host::new();
        assert!(host.connect("localhost", 7101, 7102, &auth_server).is_ok());
        let ffi = Ffi__Host::from(host);
        Host::from(ffi);

        handle.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_host_fns() {
        ZSys::init();

        create_project_fs();
        let (handle, auth_server) = mock_auth_server();

        let hostname = CString::new("localhost").unwrap().as_ptr();
        let auth_server = CString::new(auth_server.as_bytes()).unwrap().as_ptr();

        let mut host = host_new();
        host_connect(&mut host as *mut Ffi__Host, hostname, 7101, 7102, auth_server);
        host_close(&mut host as *mut Ffi__Host);

        handle.join().unwrap();
    }
}
