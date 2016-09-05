// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for File

use error::{Error, seterr};
use libc::{c_char, c_double, c_void, int32_t, uint8_t};
use rustache::{HashBuilder, VecBuilder};
use std::{convert, ptr};
use std::ffi::CString;
use std::os::unix::io::IntoRawFd;
use std::os::raw::c_int;
use std::panic::catch_unwind;
use std::path::PathBuf;
use super::*;

#[repr(C)]
pub struct Ffi__Template {
    path: *const c_char,
}

impl convert::From<Template> for Ffi__Template {
    fn from(template: Template) -> Ffi__Template {
        Ffi__Template {
            path: CString::new(template.path.to_str().unwrap()).unwrap().into_raw(),
        }
    }
}

impl convert::Into<Template> for Ffi__Template {
    fn into(self) -> Template {
        Template {
            path: PathBuf::from(trypanic!(ptrtostr!(self.path, "path string"))),
        }
    }
}

#[repr(C)]
pub struct Ffi__HashBuilder {
    inner: *mut c_void,
}

impl<'a> convert::From<HashBuilder<'a>> for Ffi__HashBuilder {
    fn from(builder: HashBuilder) -> Ffi__HashBuilder {
        Ffi__HashBuilder {
            inner: Box::into_raw(Box::new(builder)) as *mut c_void,
        }
    }
}

impl<'a> convert::Into<HashBuilder<'a>> for Ffi__HashBuilder {
    fn into(self) -> HashBuilder<'a> {
        // Using `readptr!` results in an "Overflow evaluating the
        // requirement" ICE.
        if self.inner.is_null() {
            panic!(Error::NullPtr("HashBuilder struct"));
        } else {
            unsafe { ptr::read(self.inner as *mut HashBuilder) }
        }
    }
}

#[repr(C)]
pub struct Ffi__VecBuilder {
    inner: *mut c_void,
}

impl<'a> convert::From<VecBuilder<'a>> for Ffi__VecBuilder {
    fn from(builder: VecBuilder) -> Ffi__VecBuilder {
        Ffi__VecBuilder {
            inner: Box::into_raw(Box::new(builder)) as *mut c_void,
        }
    }
}

impl<'a> convert::Into<VecBuilder<'a>> for Ffi__VecBuilder {
    fn into(self) -> VecBuilder<'a> {
        // Using `readptr!` results in an "Overflow evaluating the
        // requirement" ICE.
        if self.inner.is_null() {
            panic!(Error::NullPtr("VecBuilder struct"));
        } else {
            unsafe { ptr::read(self.inner as *mut VecBuilder) }
        }
    }
}

#[no_mangle]
pub extern "C" fn template_new(path_ptr: *const c_char) -> *mut Ffi__Template {
    let path = trynull!(ptrtostr!(path_ptr, "path string"));
    let template = trynull!(Template::new(path));
    let ffi_template: Ffi__Template = trynull!(catch_unwind(|| template.into()));
    Box::into_raw(Box::new(ffi_template))
}

#[no_mangle]
pub extern "C" fn template_render_to_file(template_ptr: *const Ffi__Template) -> c_int {
    let template: Template = tryrc!(readptr!(template_ptr, "Template struct"));
    let data = ::HashBuilder::new().insert_string("name", "Jasper Beardly");
    let fh = tryrc!(template.render_to_file(data));
    fh.into_raw_fd()
}

#[no_mangle]
pub extern "C" fn hb_new() -> *mut Ffi__HashBuilder {
    let builder: Ffi__HashBuilder = HashBuilder::new().into();
    Box::into_raw(Box::new(builder))
}

#[no_mangle]
pub extern "C" fn hb_insert_string(builder_ptr: *mut Ffi__HashBuilder, key_ptr: *const c_char, val_ptr: *const c_char) -> uint8_t {
    let builder: HashBuilder = tryrc!(readptr!(builder_ptr, "HashBuilder struct"));
    let key = tryrc!(ptrtostr!(key_ptr, "key string"));
    let value = tryrc!(ptrtostr!(val_ptr, "value string"));

    let ffi_builder = builder.insert_string(key, value).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn hb_insert_bool(builder_ptr: *mut Ffi__HashBuilder, key_ptr: *const c_char, value: uint8_t) -> uint8_t {
    let builder: HashBuilder = tryrc!(readptr!(builder_ptr, "HashBuilder struct"));
    let key = tryrc!(ptrtostr!(key_ptr, "key string"));

    let ffi_builder = builder.insert_bool(key, value == 1).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn hb_insert_int(builder_ptr: *mut Ffi__HashBuilder, key_ptr: *const c_char, value: int32_t) -> uint8_t {
    let builder: HashBuilder = tryrc!(readptr!(builder_ptr, "HashBuilder struct"));
    let key = tryrc!(ptrtostr!(key_ptr, "key string"));

    let ffi_builder = builder.insert_int(key, value).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn hb_insert_float(builder_ptr: *mut Ffi__HashBuilder, key_ptr: *const c_char, value: c_double) -> uint8_t {
    let builder: HashBuilder = tryrc!(readptr!(builder_ptr, "HashBuilder struct"));
    let key = tryrc!(ptrtostr!(key_ptr, "key string"));

    let ffi_builder = builder.insert_float(key, value).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn hb_insert_vector(builder_ptr: *mut Ffi__HashBuilder, key_ptr: *const c_char, val_ptr: *mut Ffi__VecBuilder) -> uint8_t {
    let builder: HashBuilder = tryrc!(readptr!(builder_ptr, "HashBuilder struct"));
    let key = tryrc!(ptrtostr!(key_ptr, "key string"));

    // Cannot use `readptr!` here as the resulting value cannot be
    // moved into the closure. Argh!!
    // Note: This is addressed by PR #140. When it is released, this
    // code should disappear.
    // https://github.com/rustache/rustache/pull/140
    if val_ptr.is_null() {
        seterr(Error::NullPtr("value struct"));
        return 1
    }

    let ffi_builder = builder.insert_vector(key, |_| unsafe { ptr::read(val_ptr).into() }).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn hb_insert_hash(builder_ptr: *mut Ffi__HashBuilder, key_ptr: *const c_char, val_ptr: *mut Ffi__HashBuilder) -> uint8_t {
    let builder: HashBuilder = tryrc!(readptr!(builder_ptr, "HashBuilder struct"));
    let key = tryrc!(ptrtostr!(key_ptr, "key string"));

    // Cannot use `readptr!` here as the resulting value cannot be
    // moved into the closure. Argh!!
    // Note: This is addressed by PR #140. When it is released, this
    // code should disappear.
    // https://github.com/rustache/rustache/pull/140
    if val_ptr.is_null() {
        seterr(Error::NullPtr("value struct"));
        return 1
    }

    let ffi_builder = builder.insert_hash(key, |_| unsafe { ptr::read(val_ptr).into() }).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn hb_set_partials_path(builder_ptr: *mut Ffi__HashBuilder, path_ptr: *const c_char) -> uint8_t {
    let builder: HashBuilder = tryrc!(readptr!(builder_ptr, "HashBuilder struct"));
    let path = tryrc!(ptrtostr!(path_ptr, "path string"));

    let ffi_builder = builder.set_partials_path(&path).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn vec_new() -> *mut Ffi__VecBuilder {
    let builder: Ffi__VecBuilder = VecBuilder::new().into();
    Box::into_raw(Box::new(builder))
}

#[no_mangle]
pub extern "C" fn vec_push_string(builder_ptr: *mut Ffi__VecBuilder, val_ptr: *const c_char) -> uint8_t {
    let builder: VecBuilder = tryrc!(readptr!(builder_ptr, "VecBuilder struct"));
    let value = tryrc!(ptrtostr!(val_ptr, "value string"));

    let ffi_builder = builder.push_string(value).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn vec_push_bool(builder_ptr: *mut Ffi__VecBuilder, value: uint8_t) -> uint8_t {
    let builder: VecBuilder = tryrc!(readptr!(builder_ptr, "VecBuilder struct"));

    let ffi_builder = builder.push_bool(value == 1).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn vec_push_int(builder_ptr: *mut Ffi__VecBuilder, value: int32_t) -> uint8_t {
    let builder: VecBuilder = tryrc!(readptr!(builder_ptr, "VecBuilder struct"));

    let ffi_builder = builder.push_int(value).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn vec_push_float(builder_ptr: *mut Ffi__VecBuilder, value: c_double) -> uint8_t {
    let builder: VecBuilder = tryrc!(readptr!(builder_ptr, "VecBuilder struct"));

    let ffi_builder = builder.push_float(value).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn vec_push_vector(builder_ptr: *mut Ffi__VecBuilder, val_ptr: *mut Ffi__VecBuilder) -> uint8_t {
    let builder: VecBuilder = tryrc!(readptr!(builder_ptr, "VecBuilder struct"));

    // Cannot use `readptr!` here as the resulting value cannot be
    // moved into the closure. Argh!!
    // Note: This is addressed by PR #140. When it is released, this
    // code should disappear.
    // https://github.com/rustache/rustache/pull/140
    if val_ptr.is_null() {
        seterr(Error::NullPtr("value struct"));
        return 1
    }

    let ffi_builder = builder.push_vector(|_| unsafe { ptr::read(val_ptr).into() }).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn vec_push_hash(builder_ptr: *mut Ffi__VecBuilder, val_ptr: *mut Ffi__HashBuilder) -> uint8_t {
    let builder: VecBuilder = tryrc!(readptr!(builder_ptr, "VecBuilder struct"));

    // Cannot use `readptr!` here as the resulting value cannot be
    // moved into the closure. Argh!!
    // Note: This is addressed by PR #140. When it is released, this
    // code should disappear.
    // https://github.com/rustache/rustache/pull/140
    if val_ptr.is_null() {
        seterr(Error::NullPtr("value struct"));
        return 1
    }

    let ffi_builder = builder.push_hash(|_| unsafe { ptr::read(val_ptr).into() }).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[cfg(test)]
mod tests {
    use rustache::{HashBuilder, Render};
    use std::ffi::CString;
    use std::fs;
    use std::io::{Read, Write};
    use std::os::unix::io::FromRawFd;
    use std::path::{Path, PathBuf};
    use std::str;
    use super::*;
    use tempdir::TempDir;
    use template::Template;

    #[test]
    fn test_convert_template() {
        let template = Template {
            path: PathBuf::from("/path/to/tpl"),
        };

        let ffi_template = Ffi__Template::from(template);
        assert_eq!(ptrtostr!(ffi_template.path, "path string").unwrap(), "/path/to/tpl");

        let template: Template = ffi_template.into();
        assert_eq!(template.path, Path::new("/path/to/tpl"));
    }

    #[test]
    fn test_new() {
        let path = CString::new("/path/to/nowhere").unwrap().into_raw();
        assert!(template_new(path).is_null());

        let tempdir = TempDir::new("test_template_ffi_new").unwrap();
        let path = format!("{}/tpl", tempdir.path().to_str().unwrap());
        fs::File::create(&path).unwrap();

        let path_ptr = CString::new(path.as_bytes()).unwrap().into_raw();
        let template: Template = readptr!(template_new(path_ptr), "Template struct").unwrap();
        assert_eq!(template.path.to_str().unwrap(), path);
    }

    #[test]
    fn test_render_to_file() {
        let tempdir = TempDir::new("template_test_render").unwrap();
        let template_path = format!("{}/template", tempdir.path().to_str().unwrap());

        let template_str = "Hello, {{name}}!".to_string();

        let mut fh = fs::File::create(&template_path).unwrap();
        fh.write_all(template_str.as_bytes()).unwrap();

        let template = Ffi__Template {
            path: CString::new(template_path.as_bytes()).unwrap().into_raw(),
        };
        let fd = template_render_to_file(Box::into_raw(Box::new(template)));
        assert!(fd != 0);
        let mut fh = unsafe { fs::File::from_raw_fd(fd) };
        let mut content = String::new();
        fh.read_to_string(&mut content).unwrap();
        assert_eq!(content, "Hello, Jasper Beardly!");
    }

    #[test]
    fn test_hb_insert_string() {
        let b = hb_new();
        assert_eq!(hb_insert_string(b, CString::new("key").unwrap().into_raw(), CString::new("value").unwrap().into_raw()), 0);
        assert!(!b.is_null());

        let b: HashBuilder = readptr!(b, "HashBuilder struct").unwrap();
        let stream = b.render("{{key}}").unwrap();
        assert_eq!(stream.as_slice(), b"value");
    }

    #[test]
    fn test_hb_insert_bool() {
        let b = hb_new();
        assert_eq!(hb_insert_bool(b, CString::new("key").unwrap().into_raw(), 1), 0);
        assert!(!b.is_null());

        let b: HashBuilder = readptr!(b, "HashBuilder struct").unwrap();
        let stream = b.render("{{key}}").unwrap();
        assert_eq!(stream.as_slice(), b"true");
    }

    #[test]
    fn test_hb_insert_int() {
        let b = hb_new();
        assert_eq!(hb_insert_int(b, CString::new("key").unwrap().into_raw(), -58), 0);
        assert!(!b.is_null());

        let b: HashBuilder = readptr!(b, "HashBuilder struct").unwrap();
        let stream = b.render("{{key}}").unwrap();
        assert_eq!(stream.as_slice(), b"-58");
    }

    #[test]
    fn test_hb_insert_float() {
        let b = hb_new();
        assert_eq!(hb_insert_float(b, CString::new("key").unwrap().into_raw(), 123.45), 0);
        assert!(!b.is_null());

        let b: HashBuilder = readptr!(b, "HashBuilder struct").unwrap();
        let stream = b.render("{{key}}").unwrap();
        assert_eq!(stream.as_slice(), b"123.45");
    }

    #[test]
    fn test_hb_insert_vector() {
        let v = vec_new();
        assert_eq!(vec_push_string(v, CString::new("val1").unwrap().into_raw()), 0);
        assert_eq!(vec_push_string(v, CString::new("val2").unwrap().into_raw()), 0);

        let b = hb_new();
        assert_eq!(hb_insert_vector(b, CString::new("list").unwrap().into_raw(), v), 0);

        let b: HashBuilder = readptr!(b, "HashBuilder struct").unwrap();
        let stream = b.render("{{#list}}{{.}}{{/list}}").unwrap();
        assert_eq!(stream.as_slice(), b"val1val2");
    }

    #[test]
    fn test_hb_insert_hash() {
        let c = hb_new();
        assert_eq!(hb_insert_string(c, CString::new("key").unwrap().into_raw(), CString::new("value").unwrap().into_raw()), 0);

        let b = hb_new();
        assert_eq!(hb_insert_string(b, CString::new("one").unwrap().into_raw(), CString::new("two").unwrap().into_raw()), 0);
        assert_eq!(hb_insert_hash(b, CString::new("nested").unwrap().into_raw(), c), 0);

        let b: HashBuilder = readptr!(b, "HashBuilder struct").unwrap();
        let stream = b.render("{{#nested}}{{key}}{{/nested}}{{one}}").unwrap();
        assert_eq!(stream.as_slice(), b"valuetwo");
    }

    #[test]
    fn test_hb_set_partials_path() {
        let b = hb_new();
        assert_eq!(hb_set_partials_path(b, CString::new("/path/to/partials").unwrap().into_raw()), 0);
    }
}
