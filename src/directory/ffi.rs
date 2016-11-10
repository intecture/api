// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT directory at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This directory may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Directory

use ffi_helpers::Leaky;
use file::ffi::Ffi__FileOwner;
use host::Host;
use libc::{c_char, int8_t, int16_t, uint8_t, uint16_t};
use std::convert;
use std::panic::catch_unwind;
use super::*;

#[repr(C)]
pub struct Ffi__DirectoryOpts {
    do_recursive: uint8_t,
}

impl convert::Into<Vec<DirectoryOpts>> for Ffi__DirectoryOpts {
    fn into(self) -> Vec<DirectoryOpts> {
        let mut opts = vec![];
        if self.do_recursive == 1 {
            opts.push(DirectoryOpts::DoRecursive);
        }
        opts
    }
}

#[no_mangle]
pub extern "C" fn directory_new(host_ptr: *const Host, path_ptr: *const c_char) -> *mut Directory {
    let mut host = Leaky::new(trynull!(readptr!(host_ptr, "Host pointer")));
    let path = trynull!(ptrtostr!(path_ptr, "path string"));

    let dir = trynull!(Directory::new(&mut host, path));
    Box::into_raw(Box::new(dir))
}

#[no_mangle]
pub extern "C" fn directory_exists(dir_ptr: *const Directory, host_ptr: *const Host) -> int8_t {
    let directory = Leaky::new(tryrc!(readptr!(dir_ptr, "Directory pointer"), -1));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer"), -1));

    if tryrc!(directory.exists(&mut host), -1) {
        1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn directory_create(dir_ptr: *const Directory,
                                   host_ptr: *const Host,
                                   ffi_directoryopts_ptr: *const Ffi__DirectoryOpts) -> uint8_t {
    let directory = Leaky::new(tryrc!(readptr!(dir_ptr, "Directory pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));
    let opts = match readptr!(ffi_directoryopts_ptr; Vec<DirectoryOpts>, "DirectoryOpts array") {
        Ok(o) => o,
        Err(_) => Vec::new(),
    };

    tryrc!(directory.create(&mut host, if opts.is_empty() { None } else { Some(opts.as_ref()) }));
    0
}

#[no_mangle]
pub extern "C" fn directory_delete(dir_ptr: *const Directory,
                                   host_ptr: *const Host,
                                   ffi_directoryopts_ptr: *const Ffi__DirectoryOpts) -> uint8_t {
    let directory = Leaky::new(tryrc!(readptr!(dir_ptr, "Directory pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));
    let opts = match readptr!(ffi_directoryopts_ptr; Vec<DirectoryOpts>, "DirectoryOpts array") {
        Ok(o) => o,
        Err(_) => Vec::new(),
    };

    tryrc!(directory.delete(&mut host, if opts.is_empty() { None } else { Some(opts.as_ref()) }));
    0
}

#[no_mangle]
pub extern "C" fn directory_mv(dir_ptr: *mut Directory, host_ptr: *const Host, new_path_ptr: *const c_char) -> uint8_t {
    let mut directory = Leaky::new(tryrc!(boxptr!(dir_ptr, "Directory pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));
    let new_path = tryrc!(ptrtostr!(new_path_ptr, "new path string"));

    tryrc!(directory.mv(&mut host, new_path));
    0
}

#[no_mangle]
pub extern "C" fn directory_get_owner(dir_ptr: *const Directory, host_ptr: *const Host) -> *mut Ffi__FileOwner {
    let directory = Leaky::new(trynull!(readptr!(dir_ptr, "Directory pointer")));
    let mut host = Leaky::new(trynull!(readptr!(host_ptr, "Host pointer")));

    let owner = trynull!(directory.get_owner(&mut host));
    let ffi_owner = trynull!(catch_unwind(|| owner.into()));

    Box::into_raw(Box::new(ffi_owner))
}

#[no_mangle]
pub extern "C" fn directory_set_owner(dir_ptr: *const Directory,
                                      host_ptr: *const Host,
                                      user_ptr: *const c_char,
                                      group_ptr: *const c_char) -> uint8_t {
    let directory = Leaky::new(tryrc!(readptr!(dir_ptr, "Directory pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));
    let user = tryrc!(ptrtostr!(user_ptr, "user string"));
    let group = tryrc!(ptrtostr!(group_ptr, "group string"));

    tryrc!(directory.set_owner(&mut host, user, group));
    0
}

#[no_mangle]
pub extern "C" fn directory_get_mode(dir_ptr: *const Directory, host_ptr: *const Host) -> int16_t {
    let directory = Leaky::new(tryrc!(readptr!(dir_ptr, "Directory pointer"), -1));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer"), -1));

    tryrc!(directory.get_mode(&mut host), -1) as i16
}

#[no_mangle]
pub extern "C" fn directory_set_mode(dir_ptr: *const Directory, host_ptr: *const Host, mode: uint16_t) -> uint8_t {
    let directory = Leaky::new(tryrc!(readptr!(dir_ptr, "Directory pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));

    tryrc!(directory.set_mode(&mut host, mode as u16));
    0
}

#[no_mangle]
pub extern "C" fn directory_free(dir_ptr: *mut Directory) -> uint8_t {
    tryrc!(boxptr!(dir_ptr, "Directory pointer"));
    0
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    use directory::DirectoryOpts;
    #[cfg(feature = "remote-run")]
    use host::ffi::host_close;
    #[cfg(feature = "remote-run")]
    use host::Host;
    #[cfg(feature = "remote-run")]
    use std::ffi::{CStr, CString};
    #[cfg(feature = "remote-run")]
    use std::ptr;
    #[cfg(feature = "remote-run")]
    use std::path::Path;
    use std::str;
    use super::*;
    #[cfg(feature = "remote-run")]
    use std::thread;

    #[test]
    fn test_convert_ffi_directoryopts() {
        let ffi_directoryopts = Ffi__DirectoryOpts {
            do_recursive: 1,
        };
        let directory_opts: Vec<DirectoryOpts> = ffi_directoryopts.into();

        let mut found = false;
        for opt in directory_opts {
            match opt {
                DirectoryOpts::DoRecursive => found = true,
            }
        }

        assert!(found);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_ok() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = readptr!(directory_new(host, path), "Directory pointer").unwrap();
        assert_eq!(directory.path, Path::new("/path/to/dir"));

        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_fail() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        assert!(directory_new(host, path).is_null());

        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_exists() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(host, path);
        assert!(!directory.is_null());

        assert_eq!(directory_exists(directory, host), 0);

        assert_eq!(directory_free(directory), 0);
        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_create() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();
            server.send_str("Ok").unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(host, path);
        assert!(!directory.is_null());

        let opts = Ffi__DirectoryOpts {
            do_recursive: 0
        };
        assert_eq!(directory_create(directory, host, &opts), 0);

        assert_eq!(directory_free(directory), 0);
        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_delete() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();
            server.send_str("Ok").unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(host, path);
        assert!(!directory.is_null());

        assert_eq!(directory_delete(directory, host, ptr::null()), 0);

        assert_eq!(directory_free(directory), 0);
        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_owner() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("user").unwrap();
            rep.addstr("123").unwrap();
            rep.addstr("group").unwrap();
            rep.addstr("456").unwrap();
            rep.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(host, path);
        assert!(!directory.is_null());

        let owner = readptr!(directory_get_owner(directory, host), "Directory owner").unwrap();
        assert_eq!(unsafe { CStr::from_ptr(owner.user_name).to_str().unwrap() }, "user");
        assert_eq!(owner.user_uid, 123);
        assert_eq!(unsafe { CStr::from_ptr(owner.group_name).to_str().unwrap() }, "group");
        assert_eq!(owner.group_gid, 456);

        assert_eq!(directory_free(directory), 0);
        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_owner() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();
            server.send_str("Ok").unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(host, path);
        assert!(!directory.is_null());

        let user = CString::new("Moo").unwrap().into_raw();
        let group = CString::new("Cow").unwrap().into_raw();
        let result = directory_set_owner(directory, host, user, group);
        assert_eq!(result, 0);

        assert_eq!(directory_free(directory), 0);
        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_mode() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("755").unwrap();
            rep.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(host, path);
        assert!(!directory.is_null());

        assert_eq!(directory_get_mode(directory, host), 755);

        assert_eq!(directory_free(directory), 0);
        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_mode() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();
            server.send_str("Ok").unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(host, path);
        assert!(!directory.is_null());

        assert_eq!(directory_set_mode(directory, host, 644), 0);

        assert_eq!(directory_free(directory), 0);
        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }
}
