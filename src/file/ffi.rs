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
use std::{convert, ptr, str};
use std::ffi::{CStr, CString};
use super::*;
use zfilexfer::FileOptions;

#[repr(C)]
pub struct Ffi__File {
    path: *const c_char,
}

impl convert::From<File> for Ffi__File {
    fn from(file: File) -> Ffi__File {
        Ffi__File {
            path: CString::new(file.path).unwrap().into_raw(),
        }
    }
}

impl convert::From<Ffi__File> for File {
    fn from(ffi_file: Ffi__File) -> File {
        File {
            path: unsafe { str::from_utf8(CStr::from_ptr(ffi_file.path).to_bytes()).unwrap().to_string() },
        }
    }
}

#[repr(C)]
pub struct Ffi__FileOptions {
    backup_existing: Option<*const c_char>,
    chunk_size: Option<uint64_t>,
}

impl convert::From<Ffi__FileOptions> for Vec<FileOptions> {
    fn from(ffi_opts: Ffi__FileOptions) -> Vec<FileOptions> {
        let mut opts = vec![];
        if let Some(c_suffix) = ffi_opts.backup_existing {
            let suffix = unsafe { str::from_utf8(CStr::from_ptr(c_suffix).to_bytes()).unwrap().to_string() };
            opts.push(FileOptions::BackupExisting(suffix));
        }
        if let Some(size) = ffi_opts.chunk_size {
            opts.push(FileOptions::ChunkSize(size));
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

impl convert::From<Ffi__FileOwner> for FileOwner {
    fn from(ffi_owner: Ffi__FileOwner) -> FileOwner {
        FileOwner {
            user_name: unsafe { str::from_utf8(CStr::from_ptr(ffi_owner.user_name).to_bytes()).unwrap().to_string() },
            user_uid: ffi_owner.user_uid as u64,
            group_name: unsafe { str::from_utf8(CStr::from_ptr(ffi_owner.group_name).to_bytes()).unwrap().to_string() },
            group_gid: ffi_owner.group_gid as u64,
        }
    }
}

#[no_mangle]
pub extern "C" fn file_new(ffi_host_ptr: *const Ffi__Host, path_ptr: *const c_char) -> Ffi__File {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let path = unsafe { str::from_utf8(CStr::from_ptr(path_ptr).to_bytes()).unwrap() };

    let result = Ffi__File::from(File::new(&mut host, path).unwrap());

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    result
}

#[no_mangle]
pub extern "C" fn file_exists(ffi_file_ptr: *const Ffi__File, ffi_host_ptr: *const Ffi__Host) -> uint8_t {
    let file = File::from(unsafe { ptr::read(ffi_file_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = file.exists(&mut host).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    if result { 1 } else { 0 }
}

#[cfg(feature = "remote-run")]
#[no_mangle]
pub extern "C" fn file_upload(ffi_file_ptr: *const Ffi__File, ffi_host_ptr: *const Ffi__Host, local_path_ptr: *const c_char, ffi_file_options_ptr: *const Ffi__FileOptions) {
    let file = File::from(unsafe { ptr::read(ffi_file_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let local_path = unsafe { str::from_utf8(CStr::from_ptr(local_path_ptr).to_bytes()).unwrap() };
    let mut opts: Vec<FileOptions> = vec![];
    if ffi_file_options_ptr != ptr::null() {
        opts = Vec::<FileOptions>::from(unsafe { ptr::read(ffi_file_options_ptr) });
    }

    file.upload(&mut host, local_path, if opts.is_empty() { None } else { Some(&opts) }).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[no_mangle]
pub extern "C" fn file_delete(ffi_file_ptr: *const Ffi__File, ffi_host_ptr: *const Ffi__Host) {
    let file = File::from(unsafe { ptr::read(ffi_file_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    file.delete(&mut host).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[no_mangle]
pub extern "C" fn file_mv(ffi_file_ptr: *mut Ffi__File, ffi_host_ptr: *const Ffi__Host, new_path_ptr: *const c_char) {
    let mut file = File::from(unsafe { ptr::read(ffi_file_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let new_path = unsafe { str::from_utf8(CStr::from_ptr(new_path_ptr).to_bytes()).unwrap() };

    file.mv(&mut host, new_path).unwrap();

    // Write mutated File path back to pointer
    unsafe { ptr::write(&mut *ffi_file_ptr, Ffi__File::from(file)); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[no_mangle]
pub extern "C" fn file_copy(ffi_file_ptr: *const Ffi__File, ffi_host_ptr: *const Ffi__Host, new_path_ptr: *const c_char) {
    let file = File::from(unsafe { ptr::read(ffi_file_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let new_path = unsafe { str::from_utf8(CStr::from_ptr(new_path_ptr).to_bytes()).unwrap() };

    file.copy(&mut host, new_path).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[no_mangle]
pub extern "C" fn file_get_owner(ffi_file_ptr: *const Ffi__File, ffi_host_ptr: *const Ffi__Host) -> Ffi__FileOwner {
    let file = File::from(unsafe { ptr::read(ffi_file_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = Ffi__FileOwner::from(file.get_owner(&mut host).unwrap());

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    result
}

#[no_mangle]
pub extern "C" fn file_set_owner(ffi_file_ptr: *const Ffi__File, ffi_host_ptr: *const Ffi__Host, user_ptr: *const c_char, group_ptr: *const c_char) {
    let file = File::from(unsafe { ptr::read(ffi_file_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let user = unsafe { str::from_utf8(CStr::from_ptr(user_ptr).to_bytes()).unwrap() };
    let group = unsafe { str::from_utf8(CStr::from_ptr(group_ptr).to_bytes()).unwrap() };

    file.set_owner(&mut host, user, group).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[no_mangle]
pub extern "C" fn file_get_mode(ffi_file_ptr: *const Ffi__File, ffi_host_ptr: *const Ffi__Host) -> uint16_t {
    let file = File::from(unsafe { ptr::read(ffi_file_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = file.get_mode(&mut host).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    result as uint16_t
}

#[no_mangle]
pub extern "C" fn file_set_mode(ffi_file_ptr: *const Ffi__File, ffi_host_ptr: *const Ffi__Host, mode: uint16_t) {
    let file = File::from(unsafe { ptr::read(ffi_file_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    file.set_mode(&mut host, mode as u16).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[cfg(test)]
mod tests {
    use {File, FileOptions, FileOwner, Host};
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    #[cfg(feature = "remote-run")]
    use host::ffi::Ffi__Host;
    use std::ffi::{CStr, CString};
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

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_file.path).to_bytes()).unwrap() }, "/path/to/file");
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

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_file.path).to_bytes()).unwrap() }, "/path/to/file");

        agent_mock.join().unwrap();
    }

    #[test]
    fn test_convert_ffi_file() {
        let ffi_file = Ffi__File {
            path: CString::new("/path/to/file").unwrap().into_raw(),
        };
        let file = File::from(ffi_file);

        assert_eq!(file.path, "/path/to/file");
    }

    #[test]
    fn test_convert_ffi_file_options() {
        let ffi_file_options = Ffi__FileOptions {
            backup_existing: Some(CString::new("_bak").unwrap().into_raw()),
            chunk_size: Some(123),
        };
        let file_options = Vec::<FileOptions>::from(ffi_file_options);

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
        let owner = FileOwner::from(ffi_owner);

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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(file.path).to_bytes()).unwrap() }, "/path/to/file");

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    #[should_panic()]
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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        file_new(&host as *const Ffi__Host, path);

        Host::from(host);
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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        assert_eq!(file_exists(&file as *const Ffi__File, &host as *const Ffi__Host), 0);

        Host::from(host);
        agent_mock.join().unwrap();
    }

    // XXX Need to mock FS before we can test upload effectively
    // #[test]
    // fn test_upload() {
    // }

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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        file_delete(&file as *const Ffi__File, &host as *const Ffi__Host);

        Host::from(host);
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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);

        let owner = file_get_owner(&file as *const Ffi__File, &host as *const Ffi__Host);
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(owner.user_name).to_bytes()).unwrap() }, "user");
        assert_eq!(owner.user_uid, 123);
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(owner.group_name).to_bytes()).unwrap() }, "group");
        assert_eq!(owner.group_gid, 123);

        Host::from(host);
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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        let user = CString::new("user").unwrap().into_raw();
        let group = CString::new("group").unwrap().into_raw();
        file_set_owner(&file as *const Ffi__File, &host as *const Ffi__Host, user, group);

        Host::from(host);
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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        assert_eq!(file_get_mode(&file as *const Ffi__File, &host as *const Ffi__Host), 755);

        Host::from(host);
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

        let host = Ffi__Host::from(Host::test_new(None, Some(client), None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        file_set_mode(&file as *const Ffi__File, &host as *const Ffi__Host, 644);

        Host::from(host);
        agent_mock.join().unwrap();
    }
}
