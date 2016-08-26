// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for File

use Host;
use host::ffi::Ffi__Host;
use libc::{c_char, uint8_t, uint16_t, uint64_t};
use std::{convert, ptr};
use std::ffi::CString;
use std::panic::catch_unwind;
use std::path::PathBuf;
use super::*;
use zfilexfer::FileOptions;

#[repr(C)]
pub struct Ffi__File {
    path: *const c_char,
}

impl convert::From<File> for Ffi__File {
    fn from(file: File) -> Ffi__File {
        Ffi__File {
            path: CString::new(file.path.to_str().unwrap()).unwrap().into_raw(),
        }
    }
}

impl convert::Into<File> for Ffi__File {
    fn into(self) -> File {
        File {
            path: PathBuf::from(trypanic!(ptrtostr!(self.path, "path string"))),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Ffi__FileOptions {
    backup_existing: *const c_char,
    chunk_size: *const uint64_t,
}

impl convert::Into<Vec<FileOptions>> for Ffi__FileOptions {
    fn into(self) -> Vec<FileOptions> {
        let mut opts = vec![];
        if self.backup_existing != ptr::null() {
            let suffix: String = trypanic!(ptrtostr!(self.backup_existing, "suffix string")).into();
            opts.push(FileOptions::BackupExisting(suffix));
        }
        if self.chunk_size != ptr::null() {
            opts.push(FileOptions::ChunkSize(trypanic!(readptr!(self.chunk_size, "chunk size int"))));
        }
        opts
    }
}

#[repr(C)]
pub struct Ffi__FileOwner {
    pub user_name: *const c_char,
    pub user_uid: uint64_t,
    pub group_name: *const c_char,
    pub group_gid: uint64_t,
}

impl convert::From<FileOwner> for Ffi__FileOwner {
    fn from(owner: FileOwner) -> Ffi__FileOwner {
        Ffi__FileOwner {
            user_name: CString::new(owner.user_name).unwrap().into_raw(),
            user_uid: owner.user_uid as uint64_t,
            group_name: CString::new(owner.group_name).unwrap().into_raw(),
            group_gid: owner.group_gid as uint64_t
        }
    }
}

impl convert::Into<FileOwner> for Ffi__FileOwner {
    fn into(self) -> FileOwner {
        FileOwner {
            user_name: trypanic!(ptrtostr!(self.user_name, "user name string")).into(),
            user_uid: self.user_uid as u64,
            group_name: trypanic!(ptrtostr!(self.group_name, "group name string")).into(),
            group_gid: self.group_gid as u64,
        }
    }
}

#[no_mangle]
pub extern "C" fn file_new(host_ptr: *const Ffi__Host, path_ptr: *const c_char) -> *mut Ffi__File {
    let mut host: Host = trynull!(readptr!(host_ptr, "Host struct"));
    let path = trynull!(ptrtostr!(path_ptr, "path string"));

    let file = trynull!(File::new(&mut host, path));
    let ffi_file: Ffi__File = trynull!(catch_unwind(|| file.into()));
    Box::into_raw(Box::new(ffi_file))
}

#[no_mangle]
pub extern "C" fn file_exists(file_ptr: *const Ffi__File, host_ptr: *const Ffi__Host) -> *mut uint8_t {
    let file: File = trynull!(readptr!(file_ptr, "File struct"));
    let mut host: Host = trynull!(readptr!(host_ptr, "Host struct"));

    let result = if trynull!(file.exists(&mut host)) { 1 } else { 0 };
    Box::into_raw(Box::new(result))
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn file_upload(file_ptr: *const Ffi__File,
                              host_ptr: *const Ffi__Host,
                              local_path_ptr: *const c_char,
                              file_options_ptr: *const Ffi__FileOptions) -> uint8_t {
    let file: File = tryrc!(readptr!(file_ptr, "File struct"));
    let mut host: Host = tryrc!(readptr!(host_ptr, "Host struct"));
    let local_path = tryrc!(ptrtostr!(local_path_ptr, "local path string"));
    let opts: Vec<FileOptions> = match readptr!(file_options_ptr, "FileOptions array") {
        Ok(o) => o,
        Err(_) => Vec::new(),
    };

    tryrc!(file.upload(&mut host, local_path, if opts.is_empty() { None } else { Some(&opts) }));

    0
}

#[no_mangle]
pub extern "C" fn file_delete(file_ptr: *const Ffi__File, host_ptr: *const Ffi__Host) -> uint8_t {
    let file: File = tryrc!(readptr!(file_ptr, "File struct"));
    let mut host: Host = tryrc!(readptr!(host_ptr, "Host struct"));

    tryrc!(file.delete(&mut host));

    0
}

#[no_mangle]
pub extern "C" fn file_mv(file_ptr: *mut Ffi__File, host_ptr: *const Ffi__Host, new_path_ptr: *const c_char) -> uint8_t {
    let mut file: File = tryrc!(readptr!(file_ptr, "File struct"));
    let mut host: Host = tryrc!(readptr!(host_ptr, "Host struct"));
    let new_path = tryrc!(ptrtostr!(new_path_ptr, "new path string"));

    tryrc!(file.mv(&mut host, new_path));

    // Write mutated File path back to pointer
    let ffi_file = tryrc!(catch_unwind(|| file.into()));
    unsafe { ptr::write(&mut *file_ptr, ffi_file); }

    0
}

#[no_mangle]
pub extern "C" fn file_copy(file_ptr: *const Ffi__File, host_ptr: *const Ffi__Host, new_path_ptr: *const c_char) -> uint8_t {
    let file: File = tryrc!(readptr!(file_ptr, "File struct"));
    let mut host: Host = tryrc!(readptr!(host_ptr, "Host struct"));
    let new_path = tryrc!(ptrtostr!(new_path_ptr, "new path string"));

    tryrc!(file.copy(&mut host, new_path));

    0
}

#[no_mangle]
pub extern "C" fn file_get_owner(file_ptr: *const Ffi__File, host_ptr: *const Ffi__Host) -> *mut Ffi__FileOwner {
    let file: File = trynull!(readptr!(file_ptr, "File struct"));
    let mut host: Host = trynull!(readptr!(host_ptr, "Host struct"));

    let owner = trynull!(file.get_owner(&mut host));
    let ffi_owner: Ffi__FileOwner = trynull!(catch_unwind(|| owner.into()));
    Box::into_raw(Box::new(ffi_owner))
}

#[no_mangle]
pub extern "C" fn file_set_owner(file_ptr: *const Ffi__File,
                                 host_ptr: *const Ffi__Host,
                                 user_ptr: *const c_char,
                                 group_ptr: *const c_char) -> uint8_t {
    let file: File = tryrc!(readptr!(file_ptr, "File struct"));
    let mut host: Host = tryrc!(readptr!(host_ptr, "Host struct"));
    let user = tryrc!(ptrtostr!(user_ptr, "user string"));
    let group = tryrc!(ptrtostr!(group_ptr, "group string"));

    tryrc!(file.set_owner(&mut host, user, group));

    0
}

#[no_mangle]
pub extern "C" fn file_get_mode(file_ptr: *const Ffi__File, host_ptr: *const Ffi__Host) -> *mut uint16_t {
    let file: File = trynull!(readptr!(file_ptr, "File struct"));
    let mut host: Host = trynull!(readptr!(host_ptr, "Host struct"));

    let result = trynull!(file.get_mode(&mut host));
    Box::into_raw(Box::new(result))
}

#[no_mangle]
pub extern "C" fn file_set_mode(file_ptr: *const Ffi__File, host_ptr: *const Ffi__Host, mode: uint16_t) -> uint8_t {
    let file: File = tryrc!(readptr!(file_ptr, "File struct"));
    let mut host: Host = tryrc!(readptr!(host_ptr, "Host struct"));

    tryrc!(file.set_mode(&mut host, mode as u16));

    0
}

#[cfg(test)]
mod tests {
    use {File, FileOptions, FileOwner, Host};
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    #[cfg(feature = "remote-run")]
    use host::ffi::{Ffi__Host, host_close};
    #[cfg(feature = "remote-run")]
    use libc::{uint8_t, uint16_t};
    use std::ffi::{CStr, CString};
    use std::path::Path;
    use std::str;
    use super::*;
    #[cfg(feature = "remote-run")]
    use std::thread;

    // XXX local-run tests require FS mocking

    #[cfg(feature = "local-run")]
    #[test]
    fn test_convert_file() {
        let mut host = Host::new();
        // XXX Without FS mocking this could potentially fail where
        // /path/to/file is a real path to a directory.
        let file = File::new(&mut host, "/path/to/file").unwrap();
        let ffi_file = Ffi__File::from(file);

        assert_eq!(ptrtostr!(ffi_file.path, "path string").unwrap(), "/path/to/file");
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_convert_file() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let msg = ZMsg::new();
            msg.addstr("Ok").unwrap();
            msg.addstr("1").unwrap();
            msg.send(&server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None);

        let file = File::new(&mut host, "/path/to/file").unwrap();
        let ffi_file = Ffi__File::from(file);

        assert_eq!(ptrtostr!(ffi_file.path, "path string").unwrap(), "/path/to/file");

        agent_mock.join().unwrap();
    }

    #[test]
    fn test_convert_ffi_file() {
        let ffi_file = Ffi__File {
            path: CString::new("/path/to/file").unwrap().into_raw(),
        };
        let file: File = ffi_file.into();

        assert_eq!(file.path, Path::new("/path/to/file"));
    }

    #[test]
    fn test_convert_ffi_file_options() {
        let ffi_file_options = Ffi__FileOptions {
            backup_existing: CString::new("_bak").unwrap().into_raw(),
            chunk_size: &123,
        };
        let file_options: Vec<FileOptions> = ffi_file_options.into();

        for opt in file_options {
            match opt {
                FileOptions::BackupExisting(suffix) => assert_eq!(suffix, "_bak"),
                FileOptions::ChunkSize(size) => assert_eq!(size, 123),
            }
        }
    }

    #[test]
    fn test_convert_fileowner() {
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
    fn test_convert_ffi_fileowner() {
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

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let msg = ZMsg::new();
            msg.addstr("Ok").unwrap();
            msg.addstr("1").unwrap();
            msg.send(&server).unwrap();
        });

        let mut host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file: File = readptr!(file_new(&host, path), "File struct").unwrap();
        assert_eq!(file.path, Path::new("/path/to/file"));

        assert_eq!(host_close(&mut host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_fail() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let msg = ZMsg::new();
            msg.addstr("Ok").unwrap();
            msg.addstr("0").unwrap();
            msg.send(&server).unwrap();
        });

        let mut host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        assert!(file_new(&host, path).is_null());

        assert_eq!(host_close(&mut host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_exists() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let msg = ZMsg::new();
            msg.addstr("Ok").unwrap();
            msg.addstr("1").unwrap();
            msg.send(&server).unwrap();

            server.recv_str().unwrap().unwrap();

            let msg = ZMsg::new();
            msg.addstr("Ok").unwrap();
            msg.addstr("0").unwrap();
            msg.send(&server).unwrap();
        });

        let mut host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host, path);
        assert!(!file.is_null());
        let exists: uint8_t = readptr!(file_exists(file, &host), "bool").unwrap();
        assert_eq!(exists, 0);

        assert_eq!(host_close(&mut host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_delete() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let msg = ZMsg::new();
            msg.addstr("Ok").unwrap();
            msg.addstr("1").unwrap();
            msg.send(&server).unwrap();

            server.recv_str().unwrap().unwrap();
            server.send_str("Ok").unwrap();
        });

        let mut host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host, path);
        assert!(!file.is_null());
        assert_eq!(file_delete(file, &host), 0);

        assert_eq!(host_close(&mut host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_owner() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&server).unwrap();

            server.recv_str().unwrap().unwrap();

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("user").unwrap();
            reply.addstr("123").unwrap();
            reply.addstr("group").unwrap();
            reply.addstr("123").unwrap();
            reply.send(&server).unwrap();
        });

        let mut host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host, path);
        assert!(!file.is_null());

        let owner: FileOwner = readptr!(file_get_owner(file, &host), "FileOwner struct").unwrap();
        assert_eq!(owner.user_name, "user");
        assert_eq!(owner.user_uid, 123);
        assert_eq!(owner.group_name, "group");
        assert_eq!(owner.group_gid, 123);

        assert_eq!(host_close(&mut host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_owner() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&server).unwrap();

            let msg = ZMsg::recv(&server).unwrap();
            assert_eq!("file::set_owner", msg.popstr().unwrap().unwrap());
            assert_eq!("/path/to/file", msg.popstr().unwrap().unwrap());
            assert_eq!("user", msg.popstr().unwrap().unwrap());
            assert_eq!("group", msg.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap()
        });

        let mut host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        assert!(!file.is_null());
        let user = CString::new("user").unwrap().into_raw();
        let group = CString::new("group").unwrap().into_raw();
        assert_eq!(file_set_owner(file, &host, user, group), 0);

        assert_eq!(host_close(&mut host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_mode() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&server).unwrap();

            server.recv_str().unwrap().unwrap();

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("755").unwrap();
            reply.send(&server).unwrap();
        });

        let mut host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        assert!(!file.is_null());
        let mode: uint16_t = readptr!(file_get_mode(file, &host), "mode string").unwrap();
        assert_eq!(mode, 755);

        assert_eq!(host_close(&mut host), 0);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_mode() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&server).unwrap();

            let msg = ZMsg::recv(&server).unwrap();
            assert_eq!("file::set_mode", msg.popstr().unwrap().unwrap());
            assert_eq!("/path/to/file", msg.popstr().unwrap().unwrap());
            assert_eq!("644", msg.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap()
        });

        let mut host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        assert!(!file.is_null());
        assert_eq!(file_set_mode(file, &host, 644), 0);

        assert_eq!(host_close(&mut host), 0);
        agent_mock.join().unwrap();
    }
}
