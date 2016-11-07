// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use error::Error;
use libc::size_t;
use std::{convert, mem};
use std::borrow::{Borrow, BorrowMut};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::ops::{Deref, DerefMut};

#[repr(C)]
pub struct Ffi__Array<T> {
    pub ptr: *mut T,
    pub length: size_t,
    pub capacity: size_t,
}

impl <T>convert::From<Vec<T>> for Ffi__Array<T> {
    fn from(mut item: Vec<T>) -> Ffi__Array<T> {
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

impl <T>convert::Into<Vec<T>> for Ffi__Array<T> {
    fn into(self) -> Vec<T> {
        if self.ptr.is_null() {
            panic!(Error::NullPtr("array"));
        }
        unsafe { Vec::from_raw_parts(self.ptr, self.length, self.capacity) }
    }
}

pub struct Leaky<T> {
    inner: Option<T>,
}

impl<T> Leaky<T> {
    pub fn new(inner: T) -> Leaky<T> {
        Leaky {
            inner: Some(inner),
        }
    }
}

impl<T> AsRef<T> for Leaky<T> {
    fn as_ref(&self) -> &T {
        self.inner.as_ref().unwrap()
    }
}

impl<T> AsMut<T> for Leaky<T> {
    fn as_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }
}

impl<T> Borrow<T> for Leaky<T> {
    fn borrow(&self) -> &T {
        self.inner.as_ref().unwrap()
    }
}

impl<T> BorrowMut<T> for Leaky<T> {
    fn borrow_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }
}

impl<T> Debug for Leaky<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.inner.as_ref().unwrap().fmt(f)
    }
}

impl<T> Deref for Leaky<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.as_ref().unwrap()
    }
}

impl<T> DerefMut for Leaky<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.as_mut().unwrap()
    }
}

impl<T> Drop for Leaky<T> {
    fn drop(&mut self) {
        if let Some(i) = self.inner.take() {
            mem::forget(i);
        }
    }
}

macro_rules! trynull {
    ($e:expr) => (match $e {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            ::error::seterr(err);
            return ::std::ptr::null_mut()
        },
    });
}

macro_rules! tryrc {
    ($e:expr) => (tryrc!($e, 1));
    ($e:expr, $r:expr) => (match $e {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => {
            ::error::seterr(err);
            return $r
        },
    });
}

macro_rules! trypanic {
    ($e:expr) => (match $e {
        ::std::result::Result::Ok(val) => val,
        ::std::result::Result::Err(err) => panic!(err),
    });
}

macro_rules! ptrtostr {
    ($p:expr, $e:expr) => ({
        let r = $p; // Evaluate $p before consuming the result
        if r.is_null() {
            ::std::result::Result::Err(::error::Error::NullPtr($e))
        } else {
            match unsafe { ::std::ffi::CStr::from_ptr(r).to_str() } {
                ::std::result::Result::Ok(s) => ::std::result::Result::Ok(s),
                ::std::result::Result::Err(e) => ::std::result::Result::Err(::std::convert::Into::into(e)),
            }
        }
    });
}

macro_rules! readptr {
    ($p:expr, $e:expr) => ({
        let r = $p; // Evaluate $p before consuming the result
        if r.is_null() {
            ::std::result::Result::Err(::error::Error::NullPtr($e))
        } else {
            Ok(unsafe { ::std::ptr::read(r) })
        }
    });

    ($p:expr; $t:ty, $e:expr) => ({
        match readptr!($p, $e) {
            Ok(data) => {
                match ::std::panic::catch_unwind(|| ::std::convert::Into::<$t>::into(data)) {
                    ::std::result::Result::Ok(val) => ::std::result::Result::Ok(val),
                    ::std::result::Result::Err(e) => ::std::result::Result::Err(::std::convert::Into::into(e)),
                }
            },
            Err(e) => Err(e),
        }
    });
}

macro_rules! boxptr {
    ($p:expr, $e:expr) => ({
        let r = $p; // Evaluate $p before consuming the result
        if r.is_null() {
            ::std::result::Result::Err(::error::Error::NullPtr($e))
        } else {
            Ok(unsafe { ::std::boxed::Box::from_raw(r) })
        }
    });

    ($p:expr; $t:ty, $e:expr) => ({
        match readptr!($p, $e) {
            Ok(data) => {
                match ::std::panic::catch_unwind(|| ::std::convert::Into::<$t>::into(data)) {
                    ::std::result::Result::Ok(val) => ::std::result::Result::Ok(val),
                    ::std::result::Result::Err(e) => ::std::result::Result::Err(::std::convert::Into::into(e)),
                }
            },
            Err(e) => Err(e),
        }
    });
}
