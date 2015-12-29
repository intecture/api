// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Host

#[cfg(feature = "remote-run")]
use libc::{c_char, c_void, uint32_t};
use std::convert;
#[cfg(feature = "remote-run")]
use std::{ptr, str};
#[cfg(feature = "remote-run")]
use std::ffi::CStr;
use super::*;
#[cfg(feature = "remote-run")]
use zmq;

#[cfg(feature = "local-run")]
#[repr(C)]
pub struct Ffi__Host;

#[cfg(feature = "remote-run")]
#[repr(C)]
pub struct Ffi__Host {
    zmq_sock: *mut c_void,
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
            zmq_sock: if host.zmq_sock.is_some() {
                host.zmq_sock.unwrap().to_raw()
            } else {
                ptr::null_mut()
            }
        }
    }
}

impl convert::From<Ffi__Host> for Host {
    #[cfg(feature = "local-run")]
    #[allow(unused_variables)]
    fn from(ffi_host: Ffi__Host) -> Host {
        Host
    }

    #[cfg(feature = "remote-run")]
    fn from(ffi_host: Ffi__Host) -> Host {
        Host {
            zmq_sock: if ffi_host.zmq_sock == ptr::null_mut() {
                None
            } else {
                Some(zmq::Socket::from_raw(ffi_host.zmq_sock))
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
pub extern "C" fn host_connect(ffi_host_ptr: *mut Ffi__Host, ip: *const c_char, port: uint32_t) {
    let slice = unsafe { CStr::from_ptr(ip) };
    let ip_str = str::from_utf8(slice.to_bytes()).unwrap();

    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    host.connect(ip_str, port).unwrap();

    unsafe {
        ptr::write(&mut *ffi_host_ptr, Ffi__Host::from(host));
    }
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_close(ffi_host_ptr: *mut Ffi__Host) {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    host.close().unwrap();
}

#[cfg(test)]
mod tests {
    use Host;
    #[cfg(feature = "remote-run")]
    use libc::uint32_t;
    #[cfg(feature = "remote-run")]
    use std::ffi::CString;
    use super::*;
    #[cfg(feature = "remote-run")]
    use zmq;

    #[test]
    fn test_convert_host() {
        let host = Host::new();
        Ffi__Host::from(host);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_convert_host_connected() {
        let mut host = Host::new();
        assert!(host.connect("127.0.0.1", 7101).is_ok());
        Ffi__Host::from(host);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_convert_ffi_host() {
        let mut ctx = zmq::Context::new();
        let mut sock = ctx.socket(zmq::REQ).unwrap();

        let ffi_host = Ffi__Host {
            zmq_sock: sock.to_raw(),
        };

        Host::from(ffi_host);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_host_fns() {
        let mut host = host_new();
        host_connect(&mut host as *mut Ffi__Host, CString::new("localhost").unwrap().as_ptr(), 7101 as uint32_t);
        host_close(&mut host as *mut Ffi__Host);
    }
}
