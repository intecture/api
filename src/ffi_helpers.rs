// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
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
