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
use serde_json::Value;
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
    data: *mut c_void,
}

impl convert::From<Host> for Ffi__Host {
    fn from(host: Host) -> Ffi__Host {
        Ffi__Host {
            hostname: CString::new(host.hostname).unwrap().into_raw(),
            api_sock: match host.api_sock {
                Some(sock) => sock.into_raw(),
                None => ptr::null_mut(),
            },
            file_sock: match host.file_sock {
                Some(sock) => sock.into_raw(),
                None => ptr::null_mut(),
            },
            data: Box::into_raw(Box::new(host.data)) as *mut c_void,
        }
    }
}

impl convert::Into<Host> for Ffi__Host {
    fn into(self) -> Host {
        Host {
            hostname: trypanic!(ptrtostr!(self.hostname, "hostname string")).into(),
            api_sock: if self.api_sock.is_null() {
                None
            } else {
                Some(unsafe { ZSock::from_raw(self.api_sock, false) })
            },
            file_sock: if self.file_sock.is_null() {
                None
            } else {
                Some(unsafe { ZSock::from_raw(self.file_sock, false) })
            },
            data: trypanic!(readptr!(self.data as *mut Value, "Value pointer")),
        }
    }
}

#[no_mangle]
pub extern "C" fn host_local(path_ptr: *const c_char) -> *mut Ffi__Host {
    let path = if path_ptr.is_null() {
        None
    } else {
        Some(trynull!(ptrtostr!(path_ptr, "path string")))
    };

    let host = trynull!(Host::local(path));
    let ffi_host: Ffi__Host = trynull!(catch_unwind(|| host.into()));
    Box::into_raw(Box::new(ffi_host))
}

#[no_mangle]
pub extern "C" fn host_connect(path_ptr: *const c_char) -> *mut Ffi__Host {
    let path = trynull!(ptrtostr!(path_ptr, "path string"));
    let host = trynull!(Host::connect(path));
    let ffi_host: Ffi__Host = trynull!(catch_unwind(|| host.into()));
    Box::into_raw(Box::new(ffi_host))
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_connect_endpoint(hostname_ptr: *const c_char,
                                        api_port: uint32_t,
                                        upload_port: uint32_t) -> *mut Ffi__Host {
    let hostname = trynull!(ptrtostr!(hostname_ptr, "hostname string"));
    let host = trynull!(Host::connect_endpoint(hostname, api_port, upload_port));
    let ffi_host: Ffi__Host = trynull!(catch_unwind(|| host.into()));
    Box::into_raw(Box::new(ffi_host))
}

#[cfg(test)]
mod tests {
    use host::Host;
    #[cfg(feature = "remote-run")]
    use std::ffi::CString;
    use std::ptr;
    use super::*;

    #[test]
    fn test_convert_host() {
        let host = Host::local(None).unwrap();
        let ffi: Ffi__Host = host.into();
        let _: Host = ffi.into();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_host_fns() {
        let _ = ::_MOCK_ENV.init();

        let host = host_local(ptr::null());
        assert!(!host.is_null());

        let host = host_connect(ptr::null());
        assert!(!host.is_null());

        let host = host_connect_endpoint(CString::new("localhost").unwrap().as_ptr(), 7101, 7102);
        assert!(!host.is_null());
    }
}
