// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for File

#[cfg(feature = "remote-run")]
use error;
use ffi_helpers::Leaky;
use host::Host;
use libc::{c_char, int8_t, int16_t, uint8_t, uint16_t, uint64_t};
#[cfg(feature = "remote-run")]
use libc::c_int;
use std::{convert, ptr};
use std::ffi::CString;
#[cfg(feature = "remote-run")]
use std::fs;
#[cfg(feature = "remote-run")]
use std::os::unix::io::FromRawFd;
use std::panic::catch_unwind;
use super::*;
use zfilexfer::FileOptions;

#[repr(C)]
#[derive(Debug)]
pub struct Ffi__FileOptions {
    backup_existing: *const c_char,
    chunk_size: uint64_t,
}

impl convert::Into<Vec<FileOptions>> for Ffi__FileOptions {
    fn into(self) -> Vec<FileOptions> {
        let mut opts = vec![];
        if self.backup_existing != ptr::null() {
            let suffix: String = trypanic!(ptrtostr!(self.backup_existing, "suffix string")).into();
            opts.push(FileOptions::BackupExisting(suffix));
        }
        if self.chunk_size > 0 {
            opts.push(FileOptions::ChunkSize(self.chunk_size));
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

#[no_mangle]
pub extern "C" fn file_new(host_ptr: *const Host, path_ptr: *const c_char) -> *mut File {
    let mut host = Leaky::new(trynull!(readptr!(host_ptr, "Host pointer")));
    let path = trynull!(ptrtostr!(path_ptr, "path string"));

    let file = trynull!(File::new(&mut host, path));
    Box::into_raw(Box::new(file))
}

#[no_mangle]
pub extern "C" fn file_exists(file_ptr: *const File, host_ptr: *const Host) -> int8_t {
    let file = Leaky::new(tryrc!(readptr!(file_ptr, "File pointer"), -1));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer"), -1));

    if tryrc!(file.exists(&mut host), -1) {
        1
    } else {
        0
    }
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn file_upload(file_ptr: *const File,
                              host_ptr: *const Host,
                              local_path_ptr: *const c_char,
                              file_options_ptr: *const Ffi__FileOptions) -> uint8_t {
    let file = Leaky::new(tryrc!(readptr!(file_ptr, "File pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));
    let local_path = tryrc!(ptrtostr!(local_path_ptr, "local path string"));
    let opts = match readptr!(file_options_ptr; Vec<FileOptions>, "FileOptions array") {
        Ok(o) => o,
        Err(_) => Vec::new(),
    };

    tryrc!(file.upload(&mut host, local_path, if opts.is_empty() { None } else { Some(&opts) }));

    0
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn file_upload_file(file_ptr: *const File,
                                   host_ptr: *const Host,
                                   file_descriptor: c_int,
                                   file_options_ptr: *const Ffi__FileOptions) -> uint8_t {
    let file = Leaky::new(tryrc!(readptr!(file_ptr, "File pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));

    if file_descriptor == 0 {
        error::seterr(error::Error::InvalidFileDescriptor);
        return 1;
    }

    let fh = unsafe { fs::File::from_raw_fd(file_descriptor) };
    let opts = match readptr!(file_options_ptr; Vec<FileOptions>, "FileOptions array") {
        Ok(o) => o,
        Err(_) => Vec::new(),
    };

    tryrc!(file.upload_file(&mut host, fh, if opts.is_empty() { None } else { Some(&opts) }));

    0
}

#[no_mangle]
pub extern "C" fn file_delete(file_ptr: *const File, host_ptr: *const Host) -> uint8_t {
    let file = Leaky::new(tryrc!(readptr!(file_ptr, "File pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));

    tryrc!(file.delete(&mut host));

    0
}

#[no_mangle]
pub extern "C" fn file_mv(file_ptr: *mut File, host_ptr: *const Host, new_path_ptr: *const c_char) -> uint8_t {
    let mut file = Leaky::new(tryrc!(boxptr!(file_ptr, "File pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));
    let new_path = tryrc!(ptrtostr!(new_path_ptr, "new path string"));

    tryrc!(file.mv(&mut host, new_path));

    0
}

#[no_mangle]
pub extern "C" fn file_copy(file_ptr: *const File, host_ptr: *const Host, new_path_ptr: *const c_char) -> uint8_t {
    let file = Leaky::new(tryrc!(readptr!(file_ptr, "File pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));
    let new_path = tryrc!(ptrtostr!(new_path_ptr, "new path string"));

    tryrc!(file.copy(&mut host, new_path));

    0
}

#[no_mangle]
pub extern "C" fn file_get_owner(file_ptr: *const File, host_ptr: *const Host) -> *mut Ffi__FileOwner {
    let file = Leaky::new(trynull!(readptr!(file_ptr, "File pointer")));
    let mut host = Leaky::new(trynull!(readptr!(host_ptr, "Host pointer")));

    let owner = trynull!(file.get_owner(&mut host));
    let ffi_owner: Ffi__FileOwner = trynull!(catch_unwind(|| owner.into()));

    Box::into_raw(Box::new(ffi_owner))
}

#[no_mangle]
pub extern "C" fn file_set_owner(file_ptr: *const File,
                                 host_ptr: *const Host,
                                 user_ptr: *const c_char,
                                 group_ptr: *const c_char) -> uint8_t {
    let file = Leaky::new(tryrc!(readptr!(file_ptr, "File pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));
    let user = tryrc!(ptrtostr!(user_ptr, "user string"));
    let group = tryrc!(ptrtostr!(group_ptr, "group string"));

    tryrc!(file.set_owner(&mut host, user, group));

    0
}

#[no_mangle]
pub extern "C" fn file_get_mode(file_ptr: *const File, host_ptr: *const Host) -> int16_t {
    let file = Leaky::new(tryrc!(readptr!(file_ptr, "File pointer"), -1));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer"), -1));

    tryrc!(file.get_mode(&mut host), -1) as i16
}

#[no_mangle]
pub extern "C" fn file_set_mode(file_ptr: *const File, host_ptr: *const Host, mode: uint16_t) -> uint8_t {
    let file = Leaky::new(tryrc!(readptr!(file_ptr, "File pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));

    tryrc!(file.set_mode(&mut host, mode as u16));

    0
}

#[no_mangle]
pub extern "C" fn file_free(file_ptr: *mut File) -> uint8_t {
    tryrc!(boxptr!(file_ptr, "File pointer"));
    0
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    use file::FileOwner;
    #[cfg(feature = "remote-run")]
    use host::ffi::host_close;
    #[cfg(feature = "remote-run")]
    use host::Host;
    use std::ffi::{CStr, CString};
    #[cfg(feature = "remote-run")]
    use std::path::Path;
    use std::str;
    use super::*;
    #[cfg(feature = "remote-run")]
    use std::thread;
    use zfilexfer::FileOptions;

    #[test]
    fn test_convert_ffi_file_options() {
        let ffi_file_options = Ffi__FileOptions {
            backup_existing: CString::new("_bak").unwrap().into_raw(),
            chunk_size: 123,
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
        let ffi_owner: Ffi__FileOwner = owner.into();

        assert_eq!(unsafe { CStr::from_ptr(ffi_owner.user_name).to_str().unwrap() }, "Moo");
        assert_eq!(ffi_owner.user_uid, 123);
        assert_eq!(unsafe { CStr::from_ptr(ffi_owner.group_name).to_str().unwrap() }, "Cow");
        assert_eq!(ffi_owner.group_gid, 456);
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_ok() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let msg = ZMsg::new();
            msg.addstr("Ok").unwrap();
            msg.addstr("1").unwrap();
            msg.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = readptr!(file_new(host, path), "File pointer").unwrap();
        assert_eq!(file.path, Path::new("/path/to/file"));

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

            let msg = ZMsg::new();
            msg.addstr("Ok").unwrap();
            msg.addstr("0").unwrap();
            msg.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        assert!(file_new(host, path).is_null());

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

            let msg = ZMsg::new();
            msg.addstr("Ok").unwrap();
            msg.addstr("1").unwrap();
            msg.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let msg = ZMsg::new();
            msg.addstr("Ok").unwrap();
            msg.addstr("0").unwrap();
            msg.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(host, path);
        assert!(!file.is_null());

        assert_eq!(file_exists(file, host), 0);

        assert_eq!(file_free(file), 0);
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

            let msg = ZMsg::new();
            msg.addstr("Ok").unwrap();
            msg.addstr("1").unwrap();
            msg.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();
            server.send_str("Ok").unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(host, path);
        assert!(!file.is_null());

        assert_eq!(file_delete(file, host), 0);

        assert_eq!(file_free(file), 0);
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

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("user").unwrap();
            reply.addstr("123").unwrap();
            reply.addstr("group").unwrap();
            reply.addstr("456").unwrap();
            reply.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(host, path);
        assert!(!file.is_null());

        let owner = readptr!(file_get_owner(file, host), "FileOwner struct").unwrap();
        assert_eq!(unsafe { CStr::from_ptr(owner.user_name).to_str().unwrap() }, "user");
        assert_eq!(owner.user_uid, 123);
        assert_eq!(unsafe { CStr::from_ptr(owner.group_name).to_str().unwrap() }, "group");
        assert_eq!(owner.group_gid, 456);

        assert_eq!(file_free(file), 0);
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

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::set_owner", msg.popstr().unwrap().unwrap());
            assert_eq!("/path/to/file", msg.popstr().unwrap().unwrap());
            assert_eq!("user", msg.popstr().unwrap().unwrap());
            assert_eq!("group", msg.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap()
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(host, path);
        assert!(!file.is_null());

        let user = CString::new("user").unwrap().into_raw();
        let group = CString::new("group").unwrap().into_raw();
        assert_eq!(file_set_owner(file, host, user, group), 0);

        assert_eq!(file_free(file), 0);
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

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("755").unwrap();
            reply.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(host, path);
        assert!(!file.is_null());

        assert_eq!(file_get_mode(file, host), 755);

        assert_eq!(file_free(file), 0);
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

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::set_mode", msg.popstr().unwrap().unwrap());
            assert_eq!("/path/to/file", msg.popstr().unwrap().unwrap());
            assert_eq!("644", msg.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap()
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(host, path);
        assert!(!file.is_null());

        assert_eq!(file_set_mode(file, host, 644), 0);

        assert_eq!(file_free(file), 0);
        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }
}
