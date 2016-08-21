// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use libc::size_t;
use std::{convert, mem};

#[repr(C)]
pub struct Ffi__Array<T> {
    pub ptr: *mut T,
    pub length: size_t,
    pub capacity: size_t,
}

impl <T>convert::From<Vec<T>> for Ffi__Array<T> {
    fn from(item: Vec<T>) -> Ffi__Array<T> {
        let mut item = item;

        item.shrink_to_fit();

        let ffi_item = Ffi__Array {
            ptr: item.as_mut_ptr(),
            length: item.len() as size_t,
            capacity: item.capacity() as size_t,
        };

        mem::forget(item);

        ffi_item
    }
}

macro_rules! trynull {
    ($e:expr) => (match $e {
        Ok(val) => val,
        Err(err) => {
            ::error::seterr(err);
            return ::std::ptr::null_mut()
        },
    });
}

macro_rules! tryrc {
    ($e:expr) => (match $e {
        Ok(val) => val,
        Err(err) => {
            ::error::seterr(err);
            return 1
        },
    });
}

macro_rules! ptrtostr {
    ($p:expr, $e:expr) => ({
        let r = $p; // Evaluate $p before consuming the result
        if r.is_null() {
            Err(::error::Error::NullPtr(::std::convert::Into::into($e)))
        } else {
            match unsafe { ::std::ffi::CStr::from_ptr(r).to_str() } {
                Ok(s) => Ok(s),
                Err(e) => Err(e.into()),
            }
        }
    });
}

macro_rules! readptr {
    ($p:expr, $e:expr) => ({
        let r = $p; // Evaluate $p before consuming the result
        if r.is_null() {
            Err(::error::Error::NullPtr(::std::convert::Into::into($e)))
        } else {
            let val = unsafe { ::std::ptr::read(r) };
            Ok(::std::convert::Into::into(val))
        }
    });
}
