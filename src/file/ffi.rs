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
use super::*;

#[repr(C)]
pub struct Ffi__File {

}

impl convert::From<File> for Ffi__File {
    fn from(file: File) -> Ffi__File {
        Ffi__File {

        }
    }
}

impl convert::From<Ffi__File> for File {
    fn from(ffi_file: Ffi__File) -> File {
        File {

        }
    }
}

#[no_mangle]
pub extern "C" fn myfn(ffi_file_ptr: *mut Ffi__File, ffi_host_ptr: *const Ffi__Host) -> Ffi__File {
    let mut file = File::from(unsafe { ptr::read(ffi_file_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);
}

#[cfg(test)]
mod tests {
    use Host;
    use host::ffi::Ffi__Host;
    use super::*;

    #[test]
    fn test_() {

    }
}
