// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
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
pub struct Ffi__FileOpts {
    backup_existing_file: *const c_char,
}

impl convert::From<Vec<FileOpts>> for Ffi__FileOpts {
    fn from(opts: Vec<FileOpts>) -> Ffi__FileOpts {
        let mut ffi_opts = Ffi__FileOpts {
            backup_existing_file: CString::new("").unwrap().into_raw(),
        };

        for opt in opts {
            match opt {
                FileOpts::BackupExistingFile(suffix) => ffi_opts.backup_existing_file = CString::new(suffix).unwrap().into_raw(),
            }
        }

        ffi_opts
    }
}

impl convert::From<Ffi__FileOpts> for Vec<FileOpts> {
    fn from(ffi_opts: Ffi__FileOpts) -> Vec<FileOpts> {
        let mut opts = vec![];
        if ffi_opts.backup_existing_file != ptr::null() {
            let suffix = unsafe { str::from_utf8(CStr::from_ptr(ffi_opts.backup_existing_file).to_bytes()).unwrap().to_string() };
            opts.push(FileOpts::BackupExistingFile(suffix));
        }
        opts
    }
}

#[repr(C)]
pub struct Ffi__FileOwner {
    user_name: *const c_char,
    user_uid: uint64_t,
    group_name: *const c_char,
    group_gid: uint64_t,
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
pub extern "C" fn file_upload(ffi_file_ptr: *const Ffi__File, ffi_host_ptr: *const Ffi__Host, local_path_ptr: *const c_char, ffi_fileopts_ptr: *const Ffi__FileOpts) {
    let file = File::from(unsafe { ptr::read(ffi_file_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let local_path = unsafe { str::from_utf8(CStr::from_ptr(local_path_ptr).to_bytes()).unwrap() };
    let mut opts: Vec<FileOpts> = vec![];
    if ffi_fileopts_ptr != ptr::null() {
        opts = Vec::<FileOpts>::from(unsafe { ptr::read(ffi_fileopts_ptr) });
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
    use {File, FileOpts, FileOwner, Host};
    #[cfg(feature = "remote-run")]
    use host::ffi::Ffi__Host;
    use std::ffi::{CStr, CString};
    use std::str;
    use super::*;
    #[cfg(feature = "remote-run")]
    use std::thread;
    #[cfg(feature = "remote-run")]
    use zmq;

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
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_convert_file").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_convert_file").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

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
    fn test_convert_fileopts() {
        let fileopts = vec![FileOpts::BackupExistingFile("_bak".to_string())];
        let ffi_fileopts = Ffi__FileOpts::from(fileopts);

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_fileopts.backup_existing_file).to_bytes()).unwrap() }, "_bak");
    }

    #[test]
    fn test_convert_ffi_fileopts() {
        let ffi_fileopts = Ffi__FileOpts {
            backup_existing_file: CString::new("_bak").unwrap().into_raw(),
        };
        let fileopts = Vec::<FileOpts>::from(ffi_fileopts);

        for opt in fileopts {
            match opt {
                FileOpts::BackupExistingFile(suffix) => assert_eq!(suffix, "_bak"),
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
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_new_ok").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_new_ok").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

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
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_new_fail").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_new_fail").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        file_new(&host as *const Ffi__Host, path);

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_exists() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_exists").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::exists", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_exists").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

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
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_delete").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::delete", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_delete").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        file_delete(&file as *const Ffi__File, &host as *const Ffi__Host);

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_owner() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_get_owner").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::get_owner", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Moo", zmq::SNDMORE).unwrap();
            agent_sock.send_str("123", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Cow", zmq::SNDMORE).unwrap();
            agent_sock.send_str("456", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_get_owner").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);

        let owner = file_get_owner(&file as *const Ffi__File, &host as *const Ffi__Host);
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(owner.user_name).to_bytes()).unwrap() }, "Moo");
        assert_eq!(owner.user_uid, 123);
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(owner.group_name).to_bytes()).unwrap() }, "Cow");
        assert_eq!(owner.group_gid, 456);

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_owner() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_set_owner").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::set_owner", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("Moo", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("Cow", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_set_owner").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        let user = CString::new("Moo").unwrap().into_raw();
        let group = CString::new("Cow").unwrap().into_raw();
        file_set_owner(&file as *const Ffi__File, &host as *const Ffi__Host, user, group);

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_mode() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_get_mode").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::get_mode", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("755", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_get_mode").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        assert_eq!(file_get_mode(&file as *const Ffi__File, &host as *const Ffi__Host), 755);

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_mode() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_set_mode").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::set_mode", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("644", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_set_mode").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/file").unwrap().into_raw();
        let file = file_new(&host as *const Ffi__Host, path);
        file_set_mode(&file as *const Ffi__File, &host as *const Ffi__Host, 644);

        Host::from(host);
        agent_mock.join().unwrap();
    }
}
