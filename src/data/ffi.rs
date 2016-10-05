// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Data

use error;
use ffi_helpers::Ffi__Array;
use libc::{c_char, c_void, uint8_t};
use serde_json::Value;
use std::ffi::CString;
use std::ptr;
use super::*;

#[repr(C)]
pub enum Ffi__DataType {
    Bool,
    Int64,
    Uint64,
    Float,
    String,
    Array,
    Object,
}

#[no_mangle]
pub extern "C" fn data_open(path_ptr: *const c_char) -> *mut c_void {
    let path = trynull!(ptrtostr!(path_ptr, "path string"));
    let value = trynull!(DataParser::open(path));
    Box::into_raw(Box::new(value)) as *mut c_void
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

#[no_mangle]
pub extern "C" fn free_value(value_ptr: *mut c_void) -> uint8_t {
    let _: Box<Value> = tryrc!(boxptr!(value_ptr as *mut Value, "Value struct"));
    0
}

#[cfg(test)]
mod tests {
    use ffi_helpers::Ffi__Array;
    use libc::{c_char, c_void};
    use std::ffi::CString;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use std::ptr;
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_open_get_value() {
        let (_td, path) = create_test_data();

        // Test open
        let ps = path.to_str().unwrap();
        let c_path = CString::new(ps).unwrap();
        let value_ptr = data_open(c_path.as_ptr());
        assert!(!value_ptr.is_null());

        // Test bool
        let json_ptr = CString::new("/bool").unwrap();
        let ptr = get_value(value_ptr, Ffi__DataType::Bool, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let b = unsafe { ptr::read(ptr as *mut bool) };
        assert!(b);

        // Test i64
        let json_ptr = CString::new("/i64").unwrap();
        let ptr = get_value(value_ptr, Ffi__DataType::Int64, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let i = unsafe { ptr::read(ptr as *mut i64) };
        assert_eq!(i, -5i64);

        // Test u64
        let json_ptr = CString::new("/u64").unwrap();
        let ptr = get_value(value_ptr, Ffi__DataType::Uint64, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let i = unsafe { ptr::read(ptr as *mut u64) };
        assert_eq!(i, 10u64);

        // Test f64
        let json_ptr = CString::new("/f64").unwrap();
        let ptr = get_value(value_ptr, Ffi__DataType::Float, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let i = unsafe { ptr::read(ptr as *mut f64) };
        assert_eq!(i, 1.2f64);

        // Test string
        let json_ptr = CString::new("/string").unwrap();
        let ptr = get_value(value_ptr, Ffi__DataType::String, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let s = ptrtostr!(ptr as *const c_char, "string").unwrap();
        assert_eq!(s, "abc");

        // Test array
        let json_ptr = CString::new("/array").unwrap();
        let ptr = get_value(value_ptr, Ffi__DataType::Array, json_ptr.as_ptr());
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
        let o = get_value(value_ptr, Ffi__DataType::Object, json_ptr.as_ptr());
        assert!(!ptr.is_null());

        let json_ptr = CString::new("/a").unwrap();
        let ptr = get_value(o, Ffi__DataType::String, json_ptr.as_ptr());
        assert!(!ptr.is_null());
        let s = ptrtostr!(ptr as *const c_char, "obj string").unwrap();
        assert_eq!(s, "b");
    }

    fn create_test_data() -> (TempDir, PathBuf) {
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

        (td, path)
    }
}
