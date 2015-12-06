// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Host

use libc::{c_char, c_void, uint32_t};
use std::{convert, ptr, str};
use std::ffi::CStr;
use super::*;
use zmq;

#[repr(C)]
pub struct Ffi__Host {
    zmq_sock: *mut c_void,
}

impl convert::From<Host> for Ffi__Host {
    fn from(host: Host) -> Ffi__Host {
        let mut host = host;

        Ffi__Host {
            zmq_sock: host.zmq_sock.to_raw(),
        }
    }
}

impl convert::From<Ffi__Host> for Host {
    fn from(ffi_host: Ffi__Host) -> Host {
        Host {
            zmq_sock: zmq::Socket::from_raw(ffi_host.zmq_sock),
        }
    }
}

#[no_mangle]
pub extern "C" fn host_new(ip: *const c_char, port: uint32_t) -> Ffi__Host {
    let slice = unsafe { CStr::from_ptr(ip) };
    let ip_str = str::from_utf8(slice.to_bytes()).unwrap();
    Ffi__Host::from(Host::new(ip_str, port).unwrap())
}

#[no_mangle]
pub extern "C" fn host_close(ffi_host_ptr: *const Ffi__Host) {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    host.close().unwrap();
}

#[cfg(test)]
mod tests {
    use Host;
    use libc::uint32_t;
    use std::ffi::CString;
    use super::*;
    use zmq;

    #[test]
    fn test_convert_host() {
        let host = Host::new("localhost", 7101).unwrap();
        Ffi__Host::from(host);
    }

    #[test]
    fn test_convert_ffi_host() {
        let mut ctx = zmq::Context::new();
        let mut sock = ctx.socket(zmq::REQ).unwrap();

        let ffi_host = Ffi__Host {
            zmq_sock: sock.to_raw(),
        };

        Host::from(ffi_host);
    }

    #[test]
    fn test_host_new() {
        host_new(CString::new("localhost").unwrap().as_ptr(), 7101 as uint32_t);
    }
}