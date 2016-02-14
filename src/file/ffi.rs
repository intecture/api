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
    let user = unsafe { str::from_utf8(CStr::from_ptr(user_ptr).to_bytes()).unwrap() };;
    let group = unsafe { str::from_utf8(CStr::from_ptr(group_ptr).to_bytes()).unwrap() };;

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

// #[cfg(test)]
// mod tests {
//     use Host;
//     use host::ffi::Ffi__Host;
//     use super::*;
//
//     #[test]
//     fn test_() {
//
//     }
// }
