// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for File

use error::Error;
use libc::{c_char, c_void, uint8_t};
use mustache;
use std::{convert, ptr};
use std::os::raw::c_int;
use std::os::unix::io::IntoRawFd;
use std::panic::catch_unwind;
use super::*;

#[repr(C)]
pub struct Ffi__Template {
    inner: *mut c_void,
}

impl convert::From<Template> for Ffi__Template {
    fn from(template: Template) -> Ffi__Template {
        Ffi__Template {
            inner: Box::into_raw(Box::new(template.inner)) as *mut c_void,
        }
    }
}

impl convert::Into<Template> for Ffi__Template {
    fn into(self) -> Template {
        Template {
            inner: if self.inner.is_null() {
                panic!(Error::NullPtr("Template inner ptr"));
            } else {
                unsafe { ptr::read(self.inner as *mut mustache::Template) }
            }
        }
    }
}

#[repr(C)]
pub struct Ffi__MapBuilder {
    inner: *mut c_void,
}

impl convert::From<mustache::MapBuilder> for Ffi__MapBuilder {
    fn from(builder: mustache::MapBuilder) -> Ffi__MapBuilder {
        Ffi__MapBuilder {
            inner: Box::into_raw(Box::new(builder)) as *mut c_void,
        }
    }
}

impl convert::Into<mustache::MapBuilder> for Ffi__MapBuilder {
    fn into(self) -> mustache::MapBuilder {
        if self.inner.is_null() {
            panic!(Error::NullPtr("MapBuilder struct"));
        } else {
            unsafe { ptr::read(self.inner as *mut mustache::MapBuilder) }
        }
    }
}

#[repr(C)]
pub struct Ffi__VecBuilder {
    inner: *mut c_void,
}

impl convert::From<mustache::VecBuilder> for Ffi__VecBuilder {
    fn from(builder: mustache::VecBuilder) -> Ffi__VecBuilder {
        Ffi__VecBuilder {
            inner: Box::into_raw(Box::new(builder)) as *mut c_void,
        }
    }
}

impl convert::Into<mustache::VecBuilder> for Ffi__VecBuilder {
    fn into(self) -> mustache::VecBuilder {
        // Using `readptr!` results in an "Overflow evaluating the
        // requirement" ICE.
        if self.inner.is_null() {
            panic!(Error::NullPtr("VecBuilder struct"));
        } else {
            unsafe { ptr::read(self.inner as *mut mustache::VecBuilder) }
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

pub extern "C" fn template_render_map(template_ptr: *const Ffi__Template, builder_ptr: *mut Ffi__MapBuilder) -> c_int {
    let template: Template = tryrc!(readptr!(template_ptr, "Template struct"));
    let builder: mustache::MapBuilder = tryrc!(readptr!(builder_ptr as *mut Ffi__MapBuilder, "MapBuilder struct"));
    let fh = tryrc!(template.render_data(&builder.build()));
    fh.into_raw_fd()
}

pub extern "C" fn template_render_vec(template_ptr: *const Ffi__Template, builder_ptr: *mut Ffi__VecBuilder) -> c_int {
    let template: Template = tryrc!(readptr!(template_ptr, "Template struct"));
    let builder: mustache::VecBuilder = tryrc!(readptr!(builder_ptr as *mut Ffi__VecBuilder, "VecBuilder struct"));
    let fh = tryrc!(template.render_data(&builder.build()));
    fh.into_raw_fd()
}

#[no_mangle]
pub extern "C" fn map_new() -> *mut Ffi__MapBuilder {
    let builder: Ffi__MapBuilder = mustache::MapBuilder::new().into();
    Box::into_raw(Box::new(builder))
}

#[no_mangle]
pub extern "C" fn map_insert_str(builder_ptr: *mut Ffi__MapBuilder, key_ptr: *const c_char, val_ptr: *const c_char) -> uint8_t {
    let builder: mustache::MapBuilder = tryrc!(readptr!(builder_ptr, "MapBuilder struct"));
    let key = tryrc!(ptrtostr!(key_ptr, "key string"));
    let value = tryrc!(ptrtostr!(val_ptr, "value string"));

    let ffi_builder = builder.insert_str(key, value).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn map_insert_bool(builder_ptr: *mut Ffi__MapBuilder, key_ptr: *const c_char, value: uint8_t) -> uint8_t {
    let builder: mustache::MapBuilder = tryrc!(readptr!(builder_ptr, "MapBuilder struct"));
    let key = tryrc!(ptrtostr!(key_ptr, "key string"));

    let ffi_builder = builder.insert_bool(key, value == 1).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn map_insert_vec(builder_ptr: *mut Ffi__MapBuilder, key_ptr: *const c_char, val_ptr: *mut Ffi__VecBuilder) -> uint8_t {
    let builder: mustache::MapBuilder = tryrc!(readptr!(builder_ptr, "MapBuilder struct"));
    let key = tryrc!(ptrtostr!(key_ptr, "key string"));
    let mut value: Option<mustache::VecBuilder> = Some(tryrc!(readptr!(val_ptr, "value struct")));

    // XXX Work around FnMut
    let ffi_builder = builder.insert_vec(key, move |_| value.take().unwrap()).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn map_insert_map(builder_ptr: *mut Ffi__MapBuilder, key_ptr: *const c_char, val_ptr: *mut Ffi__MapBuilder) -> uint8_t {
    let builder: mustache::MapBuilder = tryrc!(readptr!(builder_ptr, "MapBuilder struct"));
    let key = tryrc!(ptrtostr!(key_ptr, "key string"));
    let mut value: Option<mustache::MapBuilder> = Some(tryrc!(readptr!(val_ptr, "value struct")));

    // XXX Work around FnMut
    let ffi_builder = builder.insert_map(key, move |_| value.take().unwrap()).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn vec_new() -> *mut Ffi__VecBuilder {
    let builder: Ffi__VecBuilder = mustache::VecBuilder::new().into();
    Box::into_raw(Box::new(builder))
}

#[no_mangle]
pub extern "C" fn vec_push_str(builder_ptr: *mut Ffi__VecBuilder, val_ptr: *const c_char) -> uint8_t {
    let builder: mustache::VecBuilder = tryrc!(readptr!(builder_ptr, "VecBuilder struct"));
    let value = tryrc!(ptrtostr!(val_ptr, "value string"));

    let ffi_builder = builder.push_str(value).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn vec_push_bool(builder_ptr: *mut Ffi__VecBuilder, value: uint8_t) -> uint8_t {
    let builder: mustache::VecBuilder = tryrc!(readptr!(builder_ptr, "VecBuilder struct"));

    let ffi_builder = builder.push_bool(value == 1).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn vec_push_vec(builder_ptr: *mut Ffi__VecBuilder, val_ptr: *mut Ffi__VecBuilder) -> uint8_t {
    let builder: mustache::VecBuilder = tryrc!(readptr!(builder_ptr, "VecBuilder struct"));
    let mut value: Option<mustache::VecBuilder> = Some(tryrc!(readptr!(val_ptr, "value struct")));

    let ffi_builder = builder.push_vec(move |_| value.take().unwrap()).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[no_mangle]
pub extern "C" fn vec_push_hash(builder_ptr: *mut Ffi__VecBuilder, val_ptr: *mut Ffi__MapBuilder) -> uint8_t {
    let builder: mustache::VecBuilder = tryrc!(readptr!(builder_ptr, "VecBuilder struct"));
    let mut value: Option<mustache::MapBuilder> = Some(tryrc!(readptr!(val_ptr, "value struct")));

    // XXX Work around FnMut
    let ffi_builder = builder.push_map(move |_| value.take().unwrap()).into();
    unsafe { ptr::write(&mut *builder_ptr, ffi_builder); }

    0
}

#[cfg(test)]
mod tests {
    use mustache;
    use std::ffi::CString;
    use std::fs;
    use std::io::{Read, Write};
    use std::os::unix::io::FromRawFd;
    use std::str;
    use super::*;
    use tempdir::TempDir;
    use template::Template;

    #[test]
    fn test_convert_template() {
        let tempdir = TempDir::new("test_template_ffi_new").unwrap();
        let path = format!("{}/tpl.mustache", tempdir.path().to_str().unwrap());
        fs::File::create(&path).unwrap();

        let template = Template::new(&path).unwrap();
        let ffi_template = Ffi__Template::from(template);
        let _: Template = ffi_template.into();
    }

    #[test]
    fn test_new() {
        let path = CString::new("/path/to/nowhere").unwrap().into_raw();
        assert!(template_new(path).is_null());

        let tempdir = TempDir::new("test_template_ffi_new").unwrap();
        let path = format!("{}/tpl.mustache", tempdir.path().to_str().unwrap());
        fs::File::create(&path).unwrap();

        let path_ptr = CString::new(path.as_bytes()).unwrap().into_raw();
        let _: Template = readptr!(template_new(path_ptr), "Template struct").unwrap();
    }

    #[test]
    fn test_render_map() {
        let tempdir = TempDir::new("template_test_render").unwrap();
        let template_path = format!("{}/template.mustache", tempdir.path().to_str().unwrap());

        let template_str = "Hello, {{name}}!".to_string();
        let m = map_new();
        assert_eq!(map_insert_str(m, CString::new("name").unwrap().into_raw(), CString::new("Jasper Beardly").unwrap().into_raw()), 0);
        assert!(!m.is_null());

        let mut fh = fs::File::create(&template_path).unwrap();
        fh.write_all(template_str.as_bytes()).unwrap();

        let template = Template::new(&template_path).unwrap().into();
        let fd = template_render_map(Box::into_raw(Box::new(template)), m);
        assert!(fd != 0);
        let mut fh = unsafe { fs::File::from_raw_fd(fd) };
        let mut content = String::new();
        fh.read_to_string(&mut content).unwrap();
        assert_eq!(content, "Hello, Jasper Beardly!");
    }

    #[test]
    fn test_map_insert_str() {
        let m = map_new();
        assert_eq!(map_insert_str(m, CString::new("key").unwrap().into_raw(), CString::new("value").unwrap().into_raw()), 0);
        assert!(!m.is_null());

        let template = mustache::compile_str("{{key}}");
        let m: mustache::MapBuilder = readptr!(m, "MapBuilder struct").unwrap();
        let mut result = Vec::new();
        template.render_data(&mut result, &m.build());
        assert_eq!(result, b"value");
    }

    #[test]
    fn test_map_insert_bool() {
        let m = map_new();
        assert_eq!(map_insert_bool(m, CString::new("key").unwrap().into_raw(), 1), 0);
        assert!(!m.is_null());

        let template = mustache::compile_str("{{#key}}true{{/key}}");
        let m: mustache::MapBuilder = readptr!(m, "MapBuilder struct").unwrap();
        let mut result = Vec::new();
        template.render_data(&mut result, &m.build());
        assert_eq!(result, b"true");
    }

    #[test]
    fn test_map_insert_vec() {
        let v = vec_new();
        assert_eq!(vec_push_str(v, CString::new("val1").unwrap().into_raw()), 0);
        assert_eq!(vec_push_str(v, CString::new("val2").unwrap().into_raw()), 0);

        let m = map_new();
        assert_eq!(map_insert_vec(m, CString::new("list").unwrap().into_raw(), v), 0);

        let template = mustache::compile_str("{{#list}}{{.}}{{/list}}");
        let m: mustache::MapBuilder = readptr!(m, "MapBuilder struct").unwrap();
        let mut result = Vec::new();
        template.render_data(&mut result, &m.build());
        assert_eq!(result, b"val1val2");
    }

    #[test]
    fn test_map_insert_map() {
        let c = map_new();
        assert_eq!(map_insert_str(c, CString::new("key").unwrap().into_raw(), CString::new("value").unwrap().into_raw()), 0);

        let m = map_new();
        assert_eq!(map_insert_str(m, CString::new("one").unwrap().into_raw(), CString::new("two").unwrap().into_raw()), 0);
        assert_eq!(map_insert_map(m, CString::new("nested").unwrap().into_raw(), c), 0);

        let template = mustache::compile_str("{{#nested}}{{key}}{{/nested}}{{one}}");
        let m: mustache::MapBuilder = readptr!(m, "MapBuilder struct").unwrap();
        let mut result = Vec::new();
        template.render_data(&mut result, &m.build());
        assert_eq!(result, b"valuetwo");
    }

    //
    // XXX These tests are not passing, I think because of bugs in
    // Mustache lib.
    // https://github.com/nickel-org/rust-mustache/issues/41
    //
    //
    // #[test]
    // fn test_render_vec() {
    //     let tempdir = TempDir::new("template_test_render").unwrap();
    //     let template_path = format!("{}/template.mustache", tempdir.path().to_str().unwrap());
    //
    //     let template_str = "Hello, {{#list}}{{.}},{{/list}}!".to_string();
    //     let v = vec_new();
    //     assert_eq!(vec_push_str(v, CString::new("Jasper Beardly").unwrap().into_raw()), 0);
    //     assert!(!v.is_null());
    //     assert_eq!(vec_push_str(v, CString::new("Sea Capt'n").unwrap().into_raw()), 0);
    //     assert!(!v.is_null());
    //
    //     let mut fh = fs::File::create(&template_path).unwrap();
    //     fh.write_all(template_str.as_bytes()).unwrap();
    //
    //     let template = Template::new(&template_path).unwrap().into();
    //     let fd = template_render_vec(Box::into_raw(Box::new(template)), v);
    //     assert!(fd != 0);
    //     let mut fh = unsafe { fs::File::from_raw_fd(fd) };
    //     let mut content = String::new();
    //     fh.read_to_string(&mut content).unwrap();
    //     assert_eq!(content, "Hello, Jasper Beardly,Sea Capt'n,!");
    // }

    // #[test]
    // fn test_vec_push_str() {
    //     let v = vec_new();
    //     assert_eq!(vec_push_str(v, CString::new("value").unwrap().into_raw()), 0);
    //     assert!(!v.is_null());
    //
    //     let m = map_new();
    //     assert_eq!(map_insert_vec(m, CString::new("list").unwrap().into_raw(), v), 0);
    //     assert!(!m.is_null());
    //
    //     let template = mustache::compile_str("{{#list}}{{.}}{{/list}}");
    //     let m: mustache::MapBuilder = readptr!(m, "MapBuilder struct").unwrap();
    //     let mut result = Vec::new();
    //     template.render_data(&mut result, &m.build());
    //     assert_eq!(result, b"value");
    // }

    // #[test]
    // fn test_vec_push_bool() {
    //     let v = vec_new();
    //     assert_eq!(vec_push_bool(v, 1), 0);
    //     assert!(!v.is_null());
    //
    //     let m = map_new();
    //     assert_eq!(map_insert_vec(m, CString::new("list").unwrap().into_raw(), v), 0);
    //     assert!(!m.is_null());
    //
    //     let template = mustache::compile_str("{{#list}}{{#.}}true{{/.}}{{/list}}");
    //     let m: mustache::MapBuilder = readptr!(m, "MapBuilder struct").unwrap();
    //     let mut result = Vec::new();
    //     template.render_data(&mut result, &m.build());
    //     assert_eq!(result, b"true");
    // }

    // #[test]
    // fn test_vec_push_vec() {
    //     let v = vec_new();
    //     assert_eq!(vec_push_str(v, CString::new("val1").unwrap().into_raw()), 0);
    //     assert_eq!(vec_push_str(v, CString::new("val2").unwrap().into_raw()), 0);
    //
    //     let v1 = vec_new();
    //     assert_eq!(vec_push_vec(v1, v), 0);
    //
    //     let m = map_new();
    //     assert_eq!(map_insert_vec(m, CString::new("list").unwrap().into_raw(), v1), 0);
    //     assert!(!m.is_null());
    //
    //     let template = mustache::compile_str("{{#list}}{{#.}}{{.}}{{/.}}{{/list}}").unwrap();
    //     let m: mustache::MapBuilder = readptr!(m, "MapBuilder struct").unwrap();
    //     let mut result = Vec::new();
    //     template.render_data(&mut result, &m.build());
    //     assert_eq!(result, b"val1val2");
    // }

    // #[test]
    // fn test_vec_push_map() {
    //     let m = map_new();
    //     assert_eq!(map_insert_str(m, CString::new("one").unwrap().into_raw(), CString::new("two").unwrap().into_raw()), 0);
    //
    //     let v = vec_new();
    //     assert_eq!(vec_push_hash(v, m), 0);
    //
    //     let template = mustache::compile_str("{{#.}}{{one}}{{/.}}");
    //     let v: mustache::VecBuilder = readptr!(v, "VecBuilder struct").unwrap();
    //     let mut result = Vec::new();
    //     template.render_data(&mut result, &v.build());
    //     assert_eq!(result, b"two");
    // }
}
