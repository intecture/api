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
use libc::{c_char, uint8_t, uint32_t};
use std::convert;
#[cfg(feature = "remote-run")]
use std::ptr;
#[cfg(feature = "remote-run")]
use std::ffi::CString;
#[cfg(feature = "remote-run")]
use std::os::raw::c_void;
use std::panic::catch_unwind;
use super::*;

#[cfg(feature = "local-run")]
#[repr(C)]
pub struct Ffi__Host;

#[cfg(feature = "remote-run")]
#[repr(C)]
#[derive(Debug)]
pub struct Ffi__Host {
    hostname: *mut c_char,
    api_sock: *mut c_void,
    file_sock: *mut c_void,
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
                Some(hostname) => CString::new(hostname).unwrap().into_raw(),
                None => ptr::null_mut(),
            },
            api_sock: match host.api_sock {
                Some(sock) => sock.into_raw(),
                None => ptr::null_mut(),
            },
            file_sock: match host.file_sock {
                Some(sock) => sock.into_raw(),
                None => ptr::null_mut(),
            },
        }
    }
}

impl convert::Into<Host> for Ffi__Host {
    #[cfg(feature = "local-run")]
    fn into(self) -> Host {
        Host
    }

    #[cfg(feature = "remote-run")]
    fn into(self) -> Host {
        Host {
            hostname: if self.hostname.is_null() { None } else { Some(trypanic!(ptrtostr!(self.hostname, "hostname string")).into()) },
            api_sock: if self.api_sock.is_null() { None } else { Some(ZSock::from_raw(self.api_sock, false)) },
            file_sock: if self.file_sock.is_null() { None } else { Some(ZSock::from_raw(self.file_sock, false)) },
        }
    }
}

#[no_mangle]
pub extern "C" fn host_new() -> *mut Ffi__Host {
    let ffi_host = trynull!(catch_unwind(|| Host::new().into()));
    Box::into_raw(Box::new(ffi_host))
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_connect(host_ptr: *mut Ffi__Host,
                               hostname_ptr: *const c_char,
                               api_port: uint32_t,
                               upload_port: uint32_t,
                               auth_server_ptr: *const c_char) -> uint8_t {
    let hostname = tryrc!(ptrtostr!(hostname_ptr, "hostname string"));
    let auth_server = tryrc!(ptrtostr!(auth_server_ptr, "auth server string"));
    let mut host: Host = tryrc!(readptr!(host_ptr, "Host struct"));
    tryrc!(host.connect(hostname, api_port, upload_port, auth_server));

    unsafe { ptr::write(&mut *host_ptr, Ffi__Host::from(host)); }

    0
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_close(host_ptr: *mut Ffi__Host) -> uint8_t {
    // Don't use the convert trait as we want owned ZSocks
    let ffi_host: Ffi__Host = tryrc!(readptr!(host_ptr, "Host struct"));
    let mut host = Host {
        hostname: if ffi_host.hostname.is_null() { None } else { Some(tryrc!(ptrtostr!(ffi_host.hostname, "hostname string")).into()) },
        api_sock: if ffi_host.api_sock.is_null() { None } else { Some(ZSock::from_raw(ffi_host.api_sock, true)) },
        file_sock: if ffi_host.file_sock.is_null() { None } else { Some(ZSock::from_raw(ffi_host.file_sock, true)) },
    };
    tryrc!(host.close());

    0
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
        let ffi: Ffi__Host = Host::new().into();
        let _: Host = ffi.into();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_convert_host_connected() {
        ZSys::init();

        create_project_fs();
        let (handle, auth_server) = mock_auth_server();

        let mut host = Host::new();
        assert!(host.connect("localhost", 7101, 7102, &auth_server).is_ok());
        let mut ffi = Ffi__Host::from(host);
        assert_eq!(host_close(&mut ffi), 0);

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

        let host = host_new();
        assert!(!host.is_null());
        host_connect(host, hostname, 7101, 7102, auth_server);
        host_close(host);

        handle.join().unwrap();
    }
}
