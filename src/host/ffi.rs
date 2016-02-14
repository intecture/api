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
use std::ffi::{CStr, CString};
use super::*;
#[cfg(feature = "remote-run")]
use zmq;

#[cfg(feature = "local-run")]
#[repr(C)]
pub struct Ffi__Host;

#[cfg(feature = "remote-run")]
#[repr(C)]
#[derive(Debug)]
pub struct Ffi__Host {
    hostname: *mut c_char,
    api_sock: *mut c_void,
    upload_sock: *mut c_void,
    download_port: uint32_t,
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
            hostname: if host.hostname.is_some() {
                CString::new(host.hostname.unwrap()).unwrap().into_raw()
            } else {
                CString::new("").unwrap().into_raw()
            },
            api_sock: if host.api_sock.is_some() {
                host.api_sock.unwrap().to_raw()
            } else {
                ptr::null_mut()
            },
            upload_sock: if host.upload_sock.is_some() {
                host.upload_sock.unwrap().to_raw()
            } else {
                ptr::null_mut()
            },
            download_port: if host.download_port.is_some() {
                host.download_port.unwrap()
            } else {
                0
            },
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
        let hostname = unsafe { str::from_utf8(CStr::from_ptr(ffi_host.hostname).to_bytes()).unwrap().to_string() };

        Host {
            hostname: if hostname == "" {
                Some(String::new())
            } else {
                Some(hostname)
            },
            api_sock: if ffi_host.api_sock == ptr::null_mut() {
                None
            } else {
                Some(zmq::Socket::from_raw(ffi_host.api_sock))
            },
            upload_sock: if ffi_host.upload_sock == ptr::null_mut() {
                None
            } else {
                Some(zmq::Socket::from_raw(ffi_host.upload_sock))
            },
            download_port: if ffi_host.download_port == 0 {
                None
            } else {
                Some(ffi_host.download_port)
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
                               ip: *const c_char,
                               api_port: uint32_t,
                               upload_port: uint32_t,
                               download_port: uint32_t) {
    let slice = unsafe { CStr::from_ptr(ip) };
    let ip_str = str::from_utf8(slice.to_bytes()).unwrap();

    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    host.connect(ip_str, api_port, upload_port, download_port).unwrap();

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
    use Host;
    #[cfg(feature = "remote-run")]
    use std::ffi::CString;
    use std::ptr;
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
        assert!(host.connect("127.0.0.1", 7101, 7102, 7103).is_ok());
        Ffi__Host::from(host);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_convert_ffi_host() {
        let mut ctx = zmq::Context::new();
        let mut sock = ctx.socket(zmq::REQ).unwrap();

        let ffi_host = Ffi__Host {
            hostname: CString::new("localhost").unwrap().into_raw(),
            api_sock: sock.to_raw(),
            upload_sock: ptr::null_mut(),
            download_port: 0,
        };

        Host::from(ffi_host);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_host_fns() {
        let mut host = host_new();
        host_connect(&mut host as *mut Ffi__Host,
                     CString::new("localhost").unwrap().as_ptr(),
                     7101,
                     7102,
                     7103);
        host_close(&mut host as *mut Ffi__Host);
    }
}
