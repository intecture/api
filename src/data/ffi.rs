// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Data

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
pub extern "C" fn get_value(value_ptr: *mut Value, data_type: Ffi__DataType, pointer_ptr: *const c_char) -> *mut c_void {
    let value: Value = trynull!(readptr!(value_ptr, "Value struct"));
    let pointer = if pointer_ptr.is_null() {
        None
    } else {
        ptrtostr!(pointer_ptr, "").ok()
    };

    match data_type {
        Ffi__DataType::Bool => {
            let b = if let Some(p) = pointer { trynull!(needbool!(value => p)) } else { trynull!(needbool!(value)) };
            Box::into_raw(Box::new(if b { 1u8 } else { 0u8 })) as *mut c_void
        },
        Ffi__DataType::Int64 => {
            let i = if let Some(p) = pointer { trynull!(needi64!(value => p)) } else { trynull!(needi64!(value)) };
            Box::into_raw(Box::new(i)) as *mut c_void
        },
        Ffi__DataType::Uint64 => {
            let i = if let Some(p) = pointer { trynull!(needu64!(value => p)) } else { trynull!(needu64!(value)) };
            Box::into_raw(Box::new(i)) as *mut c_void
        },
        Ffi__DataType::Float => {
            let i = if let Some(p) = pointer { trynull!(needf64!(value => p)) } else { trynull!(needf64!(value)) };
            Box::into_raw(Box::new(i)) as *mut c_void
        },
        Ffi__DataType::String => {
            let s = if let Some(p) = pointer { trynull!(needstr!(value => p)) } else { trynull!(needstr!(value)) };
            trynull!(CString::new(s)).into_raw() as *mut c_void
        },
        Ffi__DataType::Array => {
            let v = if let Some(p) = pointer { trynull!(needarray!(value => p)) } else { trynull!(needarray!(value)) };
            let mut retval = Vec::new();
            for val in v {
                retval.push(val as *const _ as *mut c_void);
            }

            Box::into_raw(Box::new(Ffi__Array::from(retval))) as *mut c_void
        },
        Ffi__DataType::Object => {
            if let Some(p) = pointer {
                if let Some(v) = value.pointer(p) {
                    v as *const _ as *mut c_void
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
    }
}

#[no_mangle]
pub extern "C" fn free_value(ffi_value_ptr: *mut c_void) -> uint8_t {
    let _: Box<Value> = tryrc!(boxptr!(ffi_value_ptr as *mut Value, "Value struct"));
    0
}
