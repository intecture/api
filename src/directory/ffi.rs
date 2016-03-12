// Copyright 2015 Intecture Developers. See the COPYRIGHT directory at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This directory may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Directory

use file::ffi::Ffi__FileOwner;
use Host;
use host::ffi::Ffi__Host;
use libc::{c_char, uint8_t, uint16_t};
use std::{convert, ptr, str};
use std::ffi::{CStr, CString};
use super::*;

#[repr(C)]
pub struct Ffi__Directory {
    path: *const c_char,
}

impl convert::From<Directory> for Ffi__Directory {
    fn from(dir: Directory) -> Ffi__Directory {
        Ffi__Directory {
            path: CString::new(dir.path).unwrap().into_raw(),
        }
    }
}

impl convert::From<Ffi__Directory> for Directory {
    fn from(ffi_dir: Ffi__Directory) -> Directory {
        Directory {
            path: unsafe { str::from_utf8(CStr::from_ptr(ffi_dir.path).to_bytes()).unwrap().to_string() },
        }
    }
}

#[repr(C)]
pub struct Ffi__DirectoryOpts {
    do_recursive: Option<uint8_t>,
}

impl convert::From<Vec<DirectoryOpts>> for Ffi__DirectoryOpts {
    fn from(opts: Vec<DirectoryOpts>) -> Ffi__DirectoryOpts {
        let mut ffi_opts = Ffi__DirectoryOpts {
            do_recursive: None,
        };

        for opt in opts {
            match opt {
                DirectoryOpts::DoRecursive => ffi_opts.do_recursive = Some(1),
            }
        }

        ffi_opts
    }
}

impl convert::From<Ffi__DirectoryOpts> for Vec<DirectoryOpts> {
    fn from(ffi_opts: Ffi__DirectoryOpts) -> Vec<DirectoryOpts> {
        let mut opts = vec![];
        if ffi_opts.do_recursive.is_some() && ffi_opts.do_recursive.unwrap() == 1 {
            opts.push(DirectoryOpts::DoRecursive);
        }
        opts
    }
}

#[no_mangle]
pub extern "C" fn directory_new(ffi_host_ptr: *const Ffi__Host, path_ptr: *const c_char) -> Ffi__Directory {
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let path = unsafe { str::from_utf8(CStr::from_ptr(path_ptr).to_bytes()).unwrap() };

    let result = Ffi__Directory::from(Directory::new(&mut host, path).unwrap());

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    result
}

#[no_mangle]
pub extern "C" fn directory_exists(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host) -> uint8_t {
    let directory = Directory::from(unsafe { ptr::read(ffi_directory_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = directory.exists(&mut host).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    if result { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn directory_create(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host, ffi_directoryopts_ptr: *const Ffi__DirectoryOpts) {
    let directory = Directory::from(unsafe { ptr::read(ffi_directory_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let mut opts: Vec<DirectoryOpts> = vec![];
    if ffi_directoryopts_ptr != ptr::null() {
        opts = Vec::<DirectoryOpts>::from(unsafe { ptr::read(ffi_directoryopts_ptr) });
    }

    directory.create(&mut host, if opts.is_empty() { None } else { Some(&opts) }).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[no_mangle]
pub extern "C" fn directory_delete(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host, ffi_directoryopts_ptr: *const Ffi__DirectoryOpts) {
    let directory = Directory::from(unsafe { ptr::read(ffi_directory_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let mut opts: Vec<DirectoryOpts> = vec![];
    if ffi_directoryopts_ptr != ptr::null() {
        opts = Vec::<DirectoryOpts>::from(unsafe { ptr::read(ffi_directoryopts_ptr) });
    }

    directory.delete(&mut host, if opts.is_empty() { None } else { Some(&opts) }).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[no_mangle]
pub extern "C" fn directory_mv(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host, new_path_ptr: *const c_char) {
    let directory = Directory::from(unsafe { ptr::read(ffi_directory_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let new_path = unsafe { str::from_utf8(CStr::from_ptr(new_path_ptr).to_bytes()).unwrap() };

    directory.mv(&mut host, new_path).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[no_mangle]
pub extern "C" fn directory_get_owner(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host) -> Ffi__FileOwner {
    let directory = Directory::from(unsafe { ptr::read(ffi_directory_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = Ffi__FileOwner::from(directory.get_owner(&mut host).unwrap());

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    result
}

#[no_mangle]
pub extern "C" fn directory_set_owner(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host, user_ptr: *const c_char, group_ptr: *const c_char) {
    let directory = Directory::from(unsafe { ptr::read(ffi_directory_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let user = unsafe { str::from_utf8(CStr::from_ptr(user_ptr).to_bytes()).unwrap() };
    let group = unsafe { str::from_utf8(CStr::from_ptr(group_ptr).to_bytes()).unwrap() };

    directory.set_owner(&mut host, user, group).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[no_mangle]
pub extern "C" fn directory_get_mode(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host) -> uint16_t {
    let directory = Directory::from(unsafe { ptr::read(ffi_directory_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    let result = directory.get_mode(&mut host).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    result as uint16_t
}

#[no_mangle]
pub extern "C" fn directory_set_mode(ffi_directory_ptr: *const Ffi__Directory, ffi_host_ptr: *const Ffi__Host, mode: uint16_t) {
    let directory = Directory::from(unsafe { ptr::read(ffi_directory_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    directory.set_mode(&mut host, mode as u16).unwrap();

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[cfg(test)]
mod tests {
    use {Directory, DirectoryOpts, Host};
    use FileOwner;
    use file::ffi::Ffi__FileOwner;
    #[cfg(feature = "remote-run")]
    use host::ffi::Ffi__Host;
    use std::ffi::{CStr, CString};
    #[cfg(feature = "remote-run")]
    use std::ptr;
    use std::str;
    use super::*;
    #[cfg(feature = "remote-run")]
    use std::thread;
    #[cfg(feature = "remote-run")]
    use zmq;

    // XXX local-run tests require FS mocking

    #[cfg(feature = "local-run")]
    #[test]
    fn test_convert_directory() {
        let mut host = Host::new();
        // XXX Without FS mocking this could potentially fail where
        // /path/to/dir is a real path to a directory.
        let directory = Directory::new(&mut host, "/path/to/dir").unwrap();
        let ffi_directory = Ffi__Directory::from(directory);

        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(ffi_directory.path).to_bytes()).unwrap() }, "/path/to/dir");
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_convert_directory() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

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
        let directory = Directory::from(ffi_directory);

        assert_eq!(directory.path, "/path/to/dir");
    }

    #[test]
    fn test_convert_directoryopts() {
        let directoryopts = vec![DirectoryOpts::DoRecursive];
        let ffi_directoryopts = Ffi__DirectoryOpts::from(directoryopts);

        assert_eq!(ffi_directoryopts.do_recursive.unwrap(), 1);
    }

    #[test]
    fn test_convert_ffi_directoryopts() {
        let ffi_directoryopts = Ffi__DirectoryOpts {
            do_recursive: Some(1),
        };
        let directoryopts = Vec::<DirectoryOpts>::from(ffi_directoryopts);

        let mut found = false;
        for opt in directoryopts {
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
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host as *const Ffi__Host, path);
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(directory.path).to_bytes()).unwrap() }, "/path/to/dir");

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    #[should_panic()]
    fn test_new_fail() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        directory_new(&host as *const Ffi__Host, path);

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_exists() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::exists", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host as *const Ffi__Host, path);
        assert_eq!(directory_exists(&directory as *const Ffi__Directory, &host as *const Ffi__Host), 0);

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_create() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::create", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("0", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host as *const Ffi__Host, path);
        let opts = Ffi__DirectoryOpts {
            do_recursive: Some(0)
        };
        directory_create(&directory as *const Ffi__Directory, &host as *const Ffi__Host, &opts as *const Ffi__DirectoryOpts);

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_delete() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::delete", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("0", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host as *const Ffi__Host, path);
        directory_delete(&directory as *const Ffi__Directory, &host as *const Ffi__Host, ptr::null());

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_owner() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::get_owner", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Moo", zmq::SNDMORE).unwrap();
            agent_sock.send_str("123", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Cow", zmq::SNDMORE).unwrap();
            agent_sock.send_str("456", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host as *const Ffi__Host, path);

        let owner = directory_get_owner(&directory as *const Ffi__Directory, &host as *const Ffi__Host);
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
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::set_owner", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("Moo", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("Cow", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host as *const Ffi__Host, path);
        let user = CString::new("Moo").unwrap().into_raw();
        let group = CString::new("Cow").unwrap().into_raw();
        directory_set_owner(&directory as *const Ffi__Directory, &host as *const Ffi__Host, user, group);

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_mode() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::get_mode", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("755", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host as *const Ffi__Host, path);
        assert_eq!(directory_get_mode(&directory as *const Ffi__Directory, &host as *const Ffi__Host), 755);

        Host::from(host);
        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_mode() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::set_mode", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("644", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));

        let path = CString::new("/path/to/dir").unwrap().into_raw();
        let directory = directory_new(&host as *const Ffi__Host, path);
        directory_set_mode(&directory as *const Ffi__Directory, &host as *const Ffi__Host, 644);

        Host::from(host);
        agent_mock.join().unwrap();
    }
}
