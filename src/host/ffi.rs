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
use error::{Error, self};
use ffi_helpers::Ffi__Array;
use libc::c_char;
#[cfg(feature = "remote-run")]
use libc::{uint8_t, uint32_t};
use serde_json::Value;
use std::convert;
use std::ptr;
use std::ffi::CString;
use std::panic::catch_unwind;
use std::os::raw::c_void;
use std::rc::Rc;
use super::*;

#[cfg(feature = "local-run")]
#[repr(C)]
pub struct Ffi__Host {
    data: *mut c_void,
}

#[cfg(feature = "remote-run")]
#[repr(C)]
#[derive(Debug)]
pub struct Ffi__Host {
    hostname: *mut c_char,
    api_sock: *mut c_void,
    file_sock: *mut c_void,
    data: *mut c_void,
}

#[cfg(feature = "local-run")]
impl convert::From<Host> for Ffi__Host {
    fn from(host: Host) -> Ffi__Host {
        let data = Rc::try_unwrap(host.data).unwrap();

        Ffi__Host {
            data: Box::into_raw(Box::new(data)) as *mut c_void,
        }
    }
}

#[cfg(feature = "remote-run")]
impl convert::From<Host> for Ffi__Host {
    fn from(host: Host) -> Ffi__Host {
        let data = Rc::try_unwrap(host.data).unwrap();

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
            data: Box::into_raw(Box::new(data)) as *mut c_void,
        }
    }
}

#[cfg(feature = "local-run")]
impl convert::Into<Host> for Ffi__Host {
    fn into(self) -> Host {
        let value = trypanic!(readptr!(self.data as *mut Value, "Value pointer"));

        Host {
            data: Rc::new(value),
        }
    }
}

#[cfg(feature = "remote-run")]
impl convert::Into<Host> for Ffi__Host {
    fn into(self) -> Host {
        let value = trypanic!(readptr!(self.data as *mut Value, "Value pointer"));

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
            data: Rc::new(value),
        }
    }
}

#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum Ffi__DataType {
    Null,
    Bool,
    Int64,
    Uint64,
    Float,
    String,
    Array,
    Object,
}

#[cfg(feature = "local-run")]
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

#[cfg(feature = "remote-run")]
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

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_close(host_ptr: *mut Ffi__Host) -> uint8_t {
    // Don't use the convert trait as we want owned ZSocks
    let ffi_host: Ffi__Host = tryrc!(readptr!(host_ptr, "Host struct"));

    if !ffi_host.api_sock.is_null() {
        unsafe { ZSock::from_raw(ffi_host.api_sock, true) };
    }
    if !ffi_host.file_sock.is_null() {
        unsafe { ZSock::from_raw(ffi_host.file_sock, true) };
    }

    let _: Box<Value> = tryrc!(boxptr!(ffi_host.data as *mut Value, "Value struct"));
    0
}

#[no_mangle]
pub extern "C" fn get_value_type(mut value_ptr: *mut c_void, pointer_ptr: *const c_char) -> *const Ffi__DataType {
    let value: Box<Value> = trynull!(boxptr!(value_ptr as *mut Value, "Value struct"));
    let retval = {
        let mut v_ref = &*value;

        if !pointer_ptr.is_null() {
            let ptr = trynull!(ptrtostr!(pointer_ptr, ""));
            match value.pointer(ptr) {
                Some(v) => v_ref = v,
                None => {
                    error::seterr(Error::Generic(format!("Could not find {} in data", ptr)));
                    return ptr::null();
                },
            }
        }

        match *v_ref {
            Value::Null => Box::into_raw(Box::new(Ffi__DataType::Null)),
            Value::Bool(_) => Box::into_raw(Box::new(Ffi__DataType::Bool)),
            Value::I64(_) => Box::into_raw(Box::new(Ffi__DataType::Int64)),
            Value::U64(_) => Box::into_raw(Box::new(Ffi__DataType::Uint64)),
            Value::F64(_) => Box::into_raw(Box::new(Ffi__DataType::Float)),
            Value::String(_) => Box::into_raw(Box::new(Ffi__DataType::String)),
            Value::Array(_) => Box::into_raw(Box::new(Ffi__DataType::Array)),
            Value::Object(_) => Box::into_raw(Box::new(Ffi__DataType::Object)),
        }
    };

    unsafe { ptr::write(&mut value_ptr, Box::into_raw(Box::new(value)) as *mut c_void) };
    retval
}

#[no_mangle]
pub extern "C" fn get_value_keys(mut value_ptr: *mut c_void, pointer_ptr: *const c_char) -> *mut Ffi__Array<*mut c_char> {
    let value: Box<Value> = trynull!(boxptr!(value_ptr as *mut Value, "Value struct"));
    let retval = {
        let mut v_ref = &*value;

        if !pointer_ptr.is_null() {
            let ptr = trynull!(ptrtostr!(pointer_ptr, ""));
            match value.pointer(ptr) {
                Some(v) => v_ref = v,
                None => {
                    error::seterr(Error::Generic(format!("Could not find {} in data", ptr)));
                    return ptr::null_mut();
                },
            }
        }

        match *v_ref {
            Value::Object(ref o) => {
                let mut keys = Vec::new();
                for key in o.keys().cloned() {
                    keys.push(trynull!(CString::new(key)).into_raw());
                }

                Box::into_raw(Box::new(Ffi__Array::from(keys)))
            },
            _ => ptr::null_mut(),
        }
    };

    unsafe { ptr::write(&mut value_ptr, Box::into_raw(Box::new(value)) as *mut c_void) };
    retval
}

#[no_mangle]
pub extern "C" fn get_value(mut value_ptr: *mut c_void, data_type: Ffi__DataType, pointer_ptr: *const c_char) -> *mut c_void {
    let value: Box<Value> = trynull!(boxptr!(value_ptr as *mut Value, "Value struct"));
    let pointer = if pointer_ptr.is_null() {
        None
    } else {
        ptrtostr!(pointer_ptr, "").ok()
    };

    match data_type {
        Ffi__DataType::Null => {
            let n = if let Some(p) = pointer { neednull!(value => p) } else { neednull!(value) };
            unsafe { ptr::write(&mut value_ptr, Box::into_raw(Box::new(value)) as *mut c_void) };

            trynull!(n);
            ptr::null_mut()
        },
        Ffi__DataType::Bool => {
            let b = if let Some(p) = pointer { needbool!(value => p) } else { needbool!(value) };
            unsafe { ptr::write(&mut value_ptr, Box::into_raw(Box::new(value)) as *mut c_void) };

            Box::into_raw(Box::new(if trynull!(b) { 1u8 } else { 0u8 })) as *mut c_void
        },
        Ffi__DataType::Int64 => {
            let i = if let Some(p) = pointer { needi64!(value => p) } else { needi64!(value) };
            unsafe { ptr::write(&mut value_ptr, Box::into_raw(Box::new(value)) as *mut c_void) };

            Box::into_raw(Box::new(trynull!(i))) as *mut c_void
        },
        Ffi__DataType::Uint64 => {
            let i = if let Some(p) = pointer { needu64!(value => p) } else { needu64!(value) };
            unsafe { ptr::write(&mut value_ptr, Box::into_raw(Box::new(value)) as *mut c_void) };

            Box::into_raw(Box::new(trynull!(i))) as *mut c_void
        },
        Ffi__DataType::Float => {
            let i = if let Some(p) = pointer { needf64!(value => p) } else { needf64!(value) };
            unsafe { ptr::write(&mut value_ptr, Box::into_raw(Box::new(value)) as *mut c_void) };

            Box::into_raw(Box::new(trynull!(i))) as *mut c_void
        },
        Ffi__DataType::String => {
            let retval = {
                let s = if let Some(p) = pointer { needstr!(value => p) } else { needstr!(value) };
                match s {
                    Ok(s) => trynull!(CString::new(s)).into_raw() as *mut c_void,
                    Err(e) => {
                        error::seterr(e);
                        ptr::null_mut()
                    },
                }
            };

            unsafe { ptr::write(&mut value_ptr, Box::into_raw(Box::new(value)) as *mut c_void) };
            retval
        },
        Ffi__DataType::Array => {
            let retval = {
                let v = if let Some(p) = pointer { needarray!(value => p) } else { needarray!(value) };
                match v {
                    Ok(v) => {
                        let mut retval = Vec::new();
                        for val in v {
                            retval.push(Box::into_raw(Box::new(val.clone())) as *mut c_void);
                        }

                        let ffi_a = Ffi__Array::from(retval);
                        Box::into_raw(Box::new(ffi_a)) as *mut c_void
                    },
                    Err(e) => {
                        error::seterr(e);
                        ptr::null_mut()
                    }
                }
            };

            unsafe { ptr::write(&mut value_ptr, Box::into_raw(Box::new(value)) as *mut c_void) };
            retval
        },
        Ffi__DataType::Object => {
            let retval = if let Some(p) = pointer {
                if let Some(v) = value.pointer(p) {
                    Box::into_raw(Box::new(v.clone())) as *mut c_void
                } else {
                    ptr::null_mut()
                }
            }
            else if value.is_object() {
                value_ptr as *mut c_void
            } else {
                ptr::null_mut()
            };

            unsafe { ptr::write(&mut value_ptr, Box::into_raw(Box::new(value)) as *mut c_void) };
            retval
        }
    }
}

#[cfg(test)]
mod tests {
    use ffi_helpers::Ffi__Array;
    use host::Host;
    use libc::c_char;
    #[cfg(feature = "remote-run")]
    use serde_json;
    use std::ffi::CString;
    #[cfg(feature = "local-run")]
    use std::fs::File;
    #[cfg(feature = "local-run")]
    use std::io::Write;
    use std::os::raw::c_void;
    use std::ptr;
    use super::*;
    #[cfg(feature = "local-run")]
    use tempdir::TempDir;

    #[cfg(feature = "local-run")]
    #[test]
    fn test_convert_host() {
        let path: Option<String> = None;
        let host = Host::local(path).unwrap();
        let ffi: Ffi__Host = host.into();
        let _: Host = ffi.into();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_convert_host() {
        let host = Host::test_new(None, None, None, None);
        let ffi: Ffi__Host = host.into();
        let _: Host = ffi.into();
    }

    #[test]
    fn test_open_get_value() {
        let host = create_host();

        // Test bool
        let json_ptr = CString::new("/bool").unwrap();
        let dt = get_value_type(host.data, json_ptr.as_ptr());
        assert!(!dt.is_null());
        assert_eq!(unsafe { ptr::read(dt) }, Ffi__DataType::Bool);
        let ptr = get_value(host.data, Ffi__DataType::Bool, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let b = unsafe { ptr::read(ptr as *mut bool) };
        assert!(b);

        // Test i64
        let json_ptr = CString::new("/i64").unwrap();
        let dt = get_value_type(host.data, json_ptr.as_ptr());
        assert!(!dt.is_null());
        assert_eq!(unsafe { ptr::read(dt) }, Ffi__DataType::Int64);
        let ptr = get_value(host.data, Ffi__DataType::Int64, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let i = unsafe { ptr::read(ptr as *mut i64) };
        assert_eq!(i, -5i64);

        // Test u64
        let json_ptr = CString::new("/u64").unwrap();
        let dt = get_value_type(host.data, json_ptr.as_ptr());
        assert!(!dt.is_null());
        assert_eq!(unsafe { ptr::read(dt) }, Ffi__DataType::Uint64);
        let ptr = get_value(host.data, Ffi__DataType::Uint64, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let i = unsafe { ptr::read(ptr as *mut u64) };
        assert_eq!(i, 10u64);

        // Test f64
        let json_ptr = CString::new("/f64").unwrap();
        let dt = get_value_type(host.data, json_ptr.as_ptr());
        assert!(!dt.is_null());
        assert_eq!(unsafe { ptr::read(dt) }, Ffi__DataType::Float);
        let ptr = get_value(host.data, Ffi__DataType::Float, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let i = unsafe { ptr::read(ptr as *mut f64) };
        assert_eq!(i, 1.2f64);

        // Test string
        let json_ptr = CString::new("/string").unwrap();
        let dt = get_value_type(host.data, json_ptr.as_ptr());
        assert!(!dt.is_null());
        assert_eq!(unsafe { ptr::read(dt) }, Ffi__DataType::String);
        let ptr = get_value(host.data, Ffi__DataType::String, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let s = ptrtostr!(ptr as *const c_char, "string").unwrap();
        assert_eq!(s, "abc");

        // Test array
        let json_ptr = CString::new("/array").unwrap();
        let dt = get_value_type(host.data, json_ptr.as_ptr());
        assert!(!dt.is_null());
        assert_eq!(unsafe { ptr::read(dt) }, Ffi__DataType::Array);
        let ptr = get_value(host.data, Ffi__DataType::Array, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let a: Vec<*mut c_void> = readptr!(ptr as *mut Ffi__Array<*mut c_void>, "array").unwrap();
        let mut iter = a.into_iter();

        // Test array int
        let ptr = get_value(iter.next().unwrap(), Ffi__DataType::Uint64, ptr::null());
        assert!(!ptr.is_null());
        let i = unsafe { ptr::read(ptr as *mut u64) };
        assert_eq!(i, 123);

        // Test array string
        let ptr = get_value(iter.next().unwrap(), Ffi__DataType::String, ptr::null());
        let s = ptrtostr!(ptr as *const c_char, "string").unwrap();
        assert_eq!(s, "def");

        // Test object
        let json_ptr = CString::new("/obj").unwrap();
        let dt = get_value_type(host.data, json_ptr.as_ptr());
        assert!(!dt.is_null());
        assert_eq!(unsafe { ptr::read(dt) }, Ffi__DataType::Object);
        let o = get_value(host.data, Ffi__DataType::Object, json_ptr.as_ptr());
        assert!(!ptr.is_null());

        let json_ptr = CString::new("/a").unwrap();
        let ptr = get_value(o, Ffi__DataType::String, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let s = ptrtostr!(ptr as *const c_char, "obj string").unwrap();
        assert_eq!(s, "b");
    }

    #[test]
    fn test_get_value_keys() {
        let host = create_host();

        let ffi_a_ptr = get_value_keys(host.data, ptr::null());
        assert!(!ffi_a_ptr.is_null());
        let ffi_a: Ffi__Array<*mut c_char> = unsafe { ptr::read(ffi_a_ptr) };
        let a: Vec<_> = ffi_a.into();
        let a1: Vec<_> = a.into_iter().map(|ptr| ptrtostr!(ptr, "key string").unwrap()).collect();
        check_array(a1);
    }

    #[cfg(feature = "local-run")]
    fn check_array(v: Vec<&str>) {
        let mut iter = v.into_iter();
        assert_eq!(iter.next().unwrap(), "array");
        assert_eq!(iter.next().unwrap(), "bool");
        assert_eq!(iter.next().unwrap(), "cpu");
        assert_eq!(iter.next().unwrap(), "f64");
        assert_eq!(iter.next().unwrap(), "fs");
        assert_eq!(iter.next().unwrap(), "hostname");
        assert_eq!(iter.next().unwrap(), "i64");
        assert_eq!(iter.next().unwrap(), "memory");
        assert_eq!(iter.next().unwrap(), "net");
        assert_eq!(iter.next().unwrap(), "obj");
        assert_eq!(iter.next().unwrap(), "os");
        assert_eq!(iter.next().unwrap(), "string");
        assert_eq!(iter.next().unwrap(), "u64");
    }

    #[cfg(feature = "remote-run")]
    fn check_array(v: Vec<&str>) {
        let mut iter = v.into_iter();
        assert_eq!(iter.next().unwrap(), "array");
        assert_eq!(iter.next().unwrap(), "bool");
        assert_eq!(iter.next().unwrap(), "f64");
        assert_eq!(iter.next().unwrap(), "i64");
        assert_eq!(iter.next().unwrap(), "obj");
        assert_eq!(iter.next().unwrap(), "string");
        assert_eq!(iter.next().unwrap(), "u64");
    }

    #[cfg(feature = "local-run")]
    fn create_host() -> Ffi__Host {
        let td = TempDir::new("test_data_ffi").unwrap();
        let mut path = td.path().to_owned();
        path.push("data.json");

        let mut fh = File::create(&path).unwrap();
        fh.write_all(b"{
            \"bool\": true,
            \"i64\": -5,
            \"u64\": 10,
            \"f64\": 1.2,
            \"string\": \"abc\",
            \"array\": [
                123,
                \"def\"
            ],
            \"obj\": {
                \"a\": \"b\"
            }
        }").unwrap();

        Host::local(Some(&path)).unwrap().into()
    }

    #[cfg(feature = "remote-run")]
    fn create_host() -> Ffi__Host {
        let v = serde_json::from_str("{
            \"bool\": true,
            \"i64\": -5,
            \"u64\": 10,
            \"f64\": 1.2,
            \"string\": \"abc\",
            \"array\": [
                123,
                \"def\"
            ],
            \"obj\": {
                \"a\": \"b\"
            }
        }").unwrap();
        Host::test_new(None, None, None, Some(v)).into()
    }
}
