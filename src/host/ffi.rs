// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Host

use error::{Error, self};
use ffi_helpers::{Ffi__Array, Leaky};
use libc::{c_char, int8_t};
#[cfg(feature = "remote-run")]
use libc::{uint8_t, uint32_t};
use serde_json::Value;
use std::{mem, ptr};
use std::ffi::CString;
use std::os::raw::c_void;
use super::*;

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
pub extern "C" fn host_local(path_ptr: *const c_char) -> *mut Host {
    let path = if path_ptr.is_null() {
        None
    } else {
        Some(trynull!(ptrtostr!(path_ptr, "path string")))
    };

    let host = trynull!(Host::local(path));
    Box::into_raw(Box::new(host))
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_connect(path_ptr: *const c_char) -> *mut Host {
    let path = trynull!(ptrtostr!(path_ptr, "path string"));
    let host = trynull!(Host::connect(path));
    Box::into_raw(Box::new(host))
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_connect_endpoint(hostname_ptr: *const c_char,
                                        api_port: uint32_t,
                                        upload_port: uint32_t) -> *mut Host {
    let hostname = trynull!(ptrtostr!(hostname_ptr, "hostname string"));
    let host = trynull!(Host::connect_endpoint(hostname, api_port, upload_port));
    Box::into_raw(Box::new(host))
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_connect_payload(api_endpoint_ptr: *const c_char, file_endpoint_ptr: *const c_char) -> *mut Host {
    let api_endpoint = trynull!(ptrtostr!(api_endpoint_ptr, "api endpoint string"));
    let file_endpoint = trynull!(ptrtostr!(file_endpoint_ptr, "file endpoint string"));
    let host = trynull!(Host::connect_payload(api_endpoint, file_endpoint));
    Box::into_raw(Box::new(host))
}

#[no_mangle]
pub extern "C" fn host_data(host_ptr: *mut Host) -> *const c_void {
    let host = Leaky::new(trynull!(readptr!(host_ptr, "Host pointer")));
    let data_ref: *const Value = &*host.data;
    data_ref as *const c_void
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn host_close(host_ptr: *mut Host) -> uint8_t {
    tryrc!(boxptr!(host_ptr, "Host pointer"));
    0
}

#[no_mangle]
pub extern "C" fn get_value_type(value_ptr: *const c_void, pointer_ptr: *const c_char) -> int8_t {
    let value: Value = tryrc!(readptr!(value_ptr as *mut Value, "Value pointer"), -1);
    let retval = {
        let mut v_ref = &value;

        if !pointer_ptr.is_null() {
            let ptr = tryrc!(ptrtostr!(pointer_ptr, ""), -1);
            match value.pointer(ptr) {
                Some(v) => v_ref = v,
                None => {
                    error::seterr(Error::Generic(format!("Could not find {} in data", ptr)));
                    return -1;
                },
            }
        }

        match *v_ref {
            Value::Null => Ffi__DataType::Null,
            Value::Bool(_) => Ffi__DataType::Bool,
            Value::I64(_) => Ffi__DataType::Int64,
            Value::U64(_) => Ffi__DataType::Uint64,
            Value::F64(_) => Ffi__DataType::Float,
            Value::String(_) => Ffi__DataType::String,
            Value::Array(_) => Ffi__DataType::Array,
            Value::Object(_) => Ffi__DataType::Object,
        }
    };

    mem::forget(value);

    retval as int8_t
}

#[no_mangle]
pub extern "C" fn get_value_keys(value_ptr: *const c_void, pointer_ptr: *const c_char) -> *const Ffi__Array<*mut c_char> {
    let value: Value = trynull!(readptr!(value_ptr as *mut Value, "Value pointer"));
    let retval = {
        let mut v_ref = &value;

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
            Value::Object(ref o) => {
                let mut keys = Vec::new();
                for key in o.keys().cloned() {
                    keys.push(trynull!(CString::new(key)).into_raw());
                }

                Box::into_raw(Box::new(Ffi__Array::from(keys)))
            },
            _ => ptr::null(),
        }
    };

    mem::forget(value);

    retval
}

#[no_mangle]
pub extern "C" fn get_value(value_ptr: *const c_void, data_type: Ffi__DataType, pointer_ptr: *const c_char) -> *mut c_void {
    let value = trynull!(readptr!(value_ptr as *mut Value, "Value pointer"));
    let pointer = if pointer_ptr.is_null() {
        None
    } else {
        ptrtostr!(pointer_ptr, "").ok()
    };

    let result = match data_type {
        Ffi__DataType::Null => {
            let n = if let Some(p) = pointer { neednull!(value => p) } else { neednull!(value) };
            trynull!(n);
            ptr::null_mut()
        },
        Ffi__DataType::Bool => {
            let b = if let Some(p) = pointer { needbool!(value => p) } else { needbool!(value) };
            let i = if trynull!(b) { 1u8 } else { 0u8 };
            Box::into_raw(Box::new(i)) as *mut c_void
        },
        Ffi__DataType::Int64 => {
            let i = if let Some(p) = pointer { needi64!(value => p) } else { needi64!(value) };
            Box::into_raw(Box::new(trynull!(i))) as *mut c_void
        },
        Ffi__DataType::Uint64 => {
            let i = if let Some(p) = pointer { needu64!(value => p) } else { needu64!(value) };
            Box::into_raw(Box::new(trynull!(i))) as *mut c_void
        },
        Ffi__DataType::Float => {
            let i = if let Some(p) = pointer { needf64!(value => p) } else { needf64!(value) };
            Box::into_raw(Box::new(trynull!(i))) as *mut c_void
        },
        Ffi__DataType::String => {
            let s = if let Some(p) = pointer { needstr!(value => p) } else { needstr!(value) };
            match s {
                Ok(s) => trynull!(CString::new(s)).into_raw() as *mut c_void,
                Err(e) => {
                    error::seterr(e);
                    ptr::null_mut()
                },
            }
        },
        Ffi__DataType::Array => {
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
        },
        Ffi__DataType::Object => {
            if let Some(p) = pointer {
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
            }
        }
    };

    mem::forget(value);

    result
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

    #[test]
    fn test_open_get_value() {
        let host = create_host();
        let data = host_data(host);

        // Test bool
        let json_ptr = CString::new("/bool").unwrap();
        let dt = get_value_type(data, json_ptr.as_ptr());
        assert!(dt > -1);
        assert_eq!(dt, Ffi__DataType::Bool as i8);
        let ptr = get_value(data, Ffi__DataType::Bool, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let b = unsafe { ptr::read(ptr as *mut bool) };
        assert!(b);

        // Test i64
        let json_ptr = CString::new("/i64").unwrap();
        let dt = get_value_type(data, json_ptr.as_ptr());
        assert!(dt > -1);
        assert_eq!(dt, Ffi__DataType::Int64 as i8);
        let ptr = get_value(data, Ffi__DataType::Int64, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let i = unsafe { ptr::read(ptr as *mut i64) };
        assert_eq!(i, -5i64);

        // Test u64
        let json_ptr = CString::new("/u64").unwrap();
        let dt = get_value_type(data, json_ptr.as_ptr());
        assert!(dt > -1);
        assert_eq!(dt, Ffi__DataType::Uint64 as i8);
        let ptr = get_value(data, Ffi__DataType::Uint64, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let i = unsafe { ptr::read(ptr as *mut u64) };
        assert_eq!(i, 10u64);

        // Test f64
        let json_ptr = CString::new("/f64").unwrap();
        let dt = get_value_type(data, json_ptr.as_ptr());
        assert!(dt > -1);
        assert_eq!(dt, Ffi__DataType::Float as i8);
        let ptr = get_value(data, Ffi__DataType::Float, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let i = unsafe { ptr::read(ptr as *mut f64) };
        assert_eq!(i, 1.2f64);

        // Test string
        let json_ptr = CString::new("/string").unwrap();
        let dt = get_value_type(data, json_ptr.as_ptr());
        assert!(dt > -1);
        assert_eq!(dt, Ffi__DataType::String as i8);
        let ptr = get_value(data, Ffi__DataType::String, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let s = ptrtostr!(ptr as *const c_char, "string").unwrap();
        assert_eq!(s, "abc");

        // Test array
        let json_ptr = CString::new("/array").unwrap();
        let dt = get_value_type(data, json_ptr.as_ptr());
        assert!(dt > -1);
        assert_eq!(dt, Ffi__DataType::Array as i8);
        let ptr = get_value(data, Ffi__DataType::Array, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let a = readptr!(ptr as *mut Ffi__Array<*mut c_void>; Vec<*mut c_void>, "array").unwrap();
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
        let dt = get_value_type(data, json_ptr.as_ptr());
        assert!(dt > -1);
        assert_eq!(dt, Ffi__DataType::Object as i8);
        let o = get_value(data, Ffi__DataType::Object, json_ptr.as_ptr());
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

        let ffi_a_ptr = get_value_keys(host_data(host), ptr::null());
        assert!(!ffi_a_ptr.is_null());
        let ffi_a: Ffi__Array<*mut c_char> = unsafe { ptr::read(ffi_a_ptr) };
        let a: Vec<_> = ffi_a.into();
        let a1: Vec<_> = a.into_iter().map(|ptr| ptrtostr!(ptr, "key string").unwrap()).collect();
        check_array(a1);
    }

    #[cfg(feature = "local-run")]
    fn check_array(v: Vec<&str>) {
        let mut iter = v.into_iter();
        assert_eq!(iter.next().unwrap(), "_telemetry");
        assert_eq!(iter.next().unwrap(), "array");
        assert_eq!(iter.next().unwrap(), "bool");
        assert_eq!(iter.next().unwrap(), "f64");
        assert_eq!(iter.next().unwrap(), "i64");
        assert_eq!(iter.next().unwrap(), "obj");
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
    fn create_host() -> *mut Host {
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

        Box::into_raw(Box::new(Host::local(Some(&path)).unwrap()))
    }

    #[cfg(feature = "remote-run")]
    fn create_host() -> *mut Host {
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
        Box::into_raw(Box::new(Host::test_new(None, None, None, Some(v))))
    }
}
