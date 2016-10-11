// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT directory at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This directory may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Directory

use file::ffi::Ffi__FileOwner;
use host::Host;
use host::ffi::Ffi__Host;
use libc::{c_char, uint8_t, uint16_t};
use std::{convert, ptr};
use std::ffi::CString;
use std::panic::catch_unwind;
use std::path::PathBuf;
use super::*;

#[repr(C)]
pub struct Ffi__Directory {
    path: *const c_char,
}

impl convert::From<Directory> for Ffi__Directory {
    fn from(dir: Directory) -> Ffi__Directory {
        Ffi__Directory {
            path: CString::new(dir.path.to_str().unwrap()).unwrap().into_raw(),
        }
    }
}

impl convert::Into<Directory> for Ffi__Directory {
    fn into(self) -> Directory {
        Directory {
            path: PathBuf::from(trypanic!(ptrtostr!(self.path, "path string"))),
        }
    }
}

#[repr(C)]
pub struct Ffi__DirectoryOpts {
    do_recursive: uint8_t,
}

impl convert::From<Vec<DirectoryOpts>> for Ffi__DirectoryOpts {
    fn from(opts: Vec<DirectoryOpts>) -> Ffi__DirectoryOpts {
        let mut ffi_opts = Ffi__DirectoryOpts {
            do_recursive: 0,
        };

        for opt in opts {
            match opt {
                DirectoryOpts::DoRecursive => ffi_opts.do_recursive = 1,
            }
        }

        ffi_opts
    }
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
pub extern "C" fn directory_new(ffi_host_ptr: *const Ffi__Host, path_ptr: *const c_char) -> *mut Ffi__Directory {
    let mut host: Host = trynull!(readptr!(ffi_host_ptr, "Host struct"));
    let path = trynull!(ptrtostr!(path_ptr, "path string"));

    let dir = trynull!(Directory::new(&mut host, path));
    let ffi_dir = trynull!(catch_unwind(|| dir.into()));
    Box::into_raw(Box::new(ffi_dir))
}

#[no_mangle]
pub extern "C" fn directory_exists(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host) -> *const uint8_t {
    let directory: Directory = trynull!(readptr!(ffi_directory_ptr, "Directory struct"));
    let mut host: Host = trynull!(readptr!(ffi_host_ptr, "Host struct"));

    let result = if trynull!(directory.exists(&mut host)) { 1 } else { 0 };
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn directory_create(ffi_directory_ptr: *const Ffi__Directory,
                                   ffi_host_ptr: *const Ffi__Host,
                                   ffi_directoryopts_ptr: *const Ffi__DirectoryOpts) -> uint8_t {
    let directory: Directory = tryrc!(readptr!(ffi_directory_ptr, "Directory struct"));
    let mut host: Host = tryrc!(readptr!(ffi_host_ptr, "Host struct"));
    let opts: Vec<DirectoryOpts> = match readptr!(ffi_directoryopts_ptr, "DirectoryOpts array") {
        Ok(o) => o,
        Err(_) => Vec::new(),
    };

    tryrc!(directory.create(&mut host, if opts.is_empty() { None } else { Some(opts.as_ref()) }));

    0
}

#[no_mangle]
pub extern "C" fn directory_delete(ffi_directory_ptr: *const Ffi__Directory,
                                   ffi_host_ptr: *const Ffi__Host,
                                   ffi_directoryopts_ptr: *const Ffi__DirectoryOpts) -> uint8_t {
    let directory: Directory = tryrc!(readptr!(ffi_directory_ptr, "Directory struct"));
    let mut host: Host = tryrc!(readptr!(ffi_host_ptr, "Host struct"));
    let opts: Vec<DirectoryOpts> = match readptr!(ffi_directoryopts_ptr, "DirectoryOpts array") {
        Ok(o) => o,
        Err(_) => Vec::new(),
    };

    tryrc!(directory.delete(&mut host, if opts.is_empty() { None } else { Some(opts.as_ref()) }));

    0
}

#[no_mangle]
pub extern "C" fn directory_mv(ffi_directory_ptr: *mut Ffi__Directory, ffi_host_ptr: *const Ffi__Host, new_path_ptr: *const c_char) -> uint8_t {
    let mut directory: Directory = tryrc!(readptr!(ffi_directory_ptr, "Directory struct"));
    let mut host: Host = tryrc!(readptr!(ffi_host_ptr, "Host struct"));
    let new_path = tryrc!(ptrtostr!(new_path_ptr, "new path string"));

    tryrc!(directory.mv(&mut host, new_path));

    // Write mutated Directory path back to pointer
    let ffi_dir = tryrc!(catch_unwind(|| directory.into()));
    unsafe { ptr::write(&mut *ffi_directory_ptr, ffi_dir); }

    0
}

#[no_mangle]
pub extern "C" fn directory_get_owner(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host) -> *mut Ffi__FileOwner {
    let directory: Directory = trynull!(readptr!(ffi_directory_ptr, "Directory struct"));
    let mut host: Host = trynull!(readptr!(ffi_host_ptr, "Host struct"));

    let owner = trynull!(directory.get_owner(&mut host));
    let ffi_owner = trynull!(catch_unwind(|| owner.into()));
    Box::into_raw(Box::new(ffi_owner))
}

#[no_mangle]
pub extern "C" fn directory_set_owner(ffi_directory_ptr: *const Ffi__Directory,
                                      ffi_host_ptr: *const Ffi__Host,
                                      user_ptr: *const c_char,
                                      group_ptr: *const c_char) -> uint8_t {
    let directory: Directory = tryrc!(readptr!(ffi_directory_ptr, "Directory struct"));
    let mut host: Host = tryrc!(readptr!(ffi_host_ptr, "Host struct"));
    let user = tryrc!(ptrtostr!(user_ptr, "user string"));
    let group = tryrc!(ptrtostr!(group_ptr, "group string"));

    tryrc!(directory.set_owner(&mut host, user, group));

    0
}

#[no_mangle]
pub extern "C" fn directory_get_mode(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host) -> *const uint16_t {
    let directory: Directory = trynull!(readptr!(ffi_directory_ptr, "Directory struct"));
    let mut host: Host = trynull!(readptr!(ffi_host_ptr, "Host struct"));

    let result = trynull!(directory.get_mode(&mut host));
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn directory_set_mode(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host, mode: uint16_t) -> uint8_t {
    let directory: Directory = tryrc!(readptr!(ffi_directory_ptr, "Directory struct"));
    let mut host: Host = tryrc!(readptr!(ffi_host_ptr, "Host struct"));

    tryrc!(directory.set_mode(&mut host, mode as u16));

    0
}

#[cfg(test)]
mod tests {
    use {Directory, DirectoryOpts, Host};
    use FileOwner;
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    use file::ffi::Ffi__FileOwner;
    #[cfg(feature = "remote-run")]
    use host::ffi::Ffi__Host;
    use std::ffi::{CStr, CString};
    #[cfg(feature = "remote-run")]
    use std::ptr;
    use std::path::Path;
    use std::str;
    use super::*;
    #[cfg(feature = "local-run")]
    use tempdir::TempDir;
    #[cfg(feature = "remote-run")]
    use std::thread;

    #[cfg(feature = "local-run")]
    #[test]
    fn test_convert_directory() {
        let tempdir = TempDir::new("directory_ffi_convert").unwrap();
        let mut path = tempdir.path().to_owned();
        path.push("path/to/dir");

        let mut host = Host::local(None);
        let directory = Directory::new(&mut host, &path).unwrap();
        let ffi_directory = Ffi__Directory::from(directory);

        assert_eq!(Path::new(ptrtostr!(ffi_directory.path, "path string").unwrap()), path);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_convert_directory() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::is_directory", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let directory = Directory::new(&mut host, "/path/to/dir").unwrap();
        let ffi_directory = Ffi__Directory::from(directory);

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_directory.path).to_bytes()).unwrap() }, "/path/to/dir");

        agent_mock.join().unwrap();
    }

    #[test]
    fn test_convert_ffi_directory() {
        let ffi_directory = Ffi__Directory {
            path: CString::new("/path/to/dir").unwrap().into_raw(),
        };
        let directory: Directory = ffi_directory.into();

        assert_eq!(directory.path, Path::new("/path/to/dir"));
    }

    #[test]
    fn test_convert_directoryopts() {
        let directoryopts = vec![DirectoryOpts::DoRecursive];
        let ffi_directoryopts = Ffi__DirectoryOpts::from(directoryopts);

        assert_eq!(ffi_directoryopts.do_recursive, 1);
    }

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

    #[test]
    fn test_convert_directoryowner() {
        let owner = FileOwner {
            user_name: "Moo".to_string(),
            user_uid: 123,
            group_name: "Cow".to_string(),
            group_gid: 456
        };
        let ffi_owner = Ffi__FileOwner::from(owner);

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_owner.user_name).to_bytes()).unwrap() }, "Moo");
        assert_eq!(ffi_owner.user_uid, 123);
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_owner.group_name).to_bytes()).unwrap() }, "Cow");
        assert_eq!(ffi_owner.group_gid, 456);
    }

    #[test]
    fn test_convert_ffi_directoryowner() {
        let ffi_owner = Ffi__FileOwner {
            user_name: CString::new("Moo").unwrap().into_raw(),
            user_uid: 123,
            group_name: CString::new("Cow").unwrap().into_raw(),
            group_gid: 456
        };
        let owner: FileOwner = ffi_owner.into();

        assert_eq!(&owner.user_name, "Moo");
        assert_eq!(owner.user_uid, 123);
        assert_eq!(&owner.group_name, "Cow");
        assert_eq!(owner.group_gid, 456);
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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = unsafe { ptr::read(directory_new(&host as *const Ffi__Host, path)) };
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(directory.path).to_bytes()).unwrap() }, "/path/to/dir");

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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        assert!(directory_new(&host, path).is_null());

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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = unsafe { ptr::read(directory_new(&host as *const Ffi__Host, path)) };
        let exists = unsafe { ptr::read(directory_exists(&directory as *const Ffi__Directory, &host as *const Ffi__Host)) };
        assert_eq!(exists, 0);

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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host, path);
        let opts = Ffi__DirectoryOpts {
            do_recursive: 0
        };
        directory_create(directory, &host, &opts);

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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host, path);
        directory_delete(directory, &host, ptr::null());

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
            rep.addstr("123").unwrap();
            rep.send(&mut server).unwrap();
        });

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host as *const Ffi__Host, path);

        let owner: FileOwner = readptr!(directory_get_owner(directory, &host as *const Ffi__Host), "Directory owner").unwrap();
        assert_eq!(owner.user_name, "user");
        assert_eq!(owner.user_uid, 123);
        assert_eq!(owner.group_name, "group");
        assert_eq!(owner.group_gid, 123);

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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host, path);
        let user = CString::new("Moo").unwrap().into_raw();
        let group = CString::new("Cow").unwrap().into_raw();
        let result = directory_set_owner(directory, &host, user, group);
        assert_eq!(result, 0);

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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host, path);
        let mode = directory_get_mode(directory, &host);
        assert!(!mode.is_null());
        assert_eq!(unsafe { ptr::read(mode) }, 755);

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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host, path);
        directory_set_mode(directory, &host, 644);

        agent_mock.join().unwrap();
    }
}
