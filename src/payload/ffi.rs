// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Payload

use ffi_helpers::{Ffi__Array, Leaky};
use host::Host;
use libc::{c_char, size_t, uint8_t};
use std::panic::catch_unwind;
use super::*;

#[no_mangle]
pub extern "C" fn payload_new(payload_artifact_ptr: *const c_char) -> *mut Payload {
    let payload_artifact = trynull!(ptrtostr!(payload_artifact_ptr, "payload::artifact string"));
    let payload = trynull!(Payload::new(&payload_artifact));
    Box::into_raw(Box::new(payload))
}

#[no_mangle]
pub extern "C" fn payload_build(payload_ptr: *mut Payload) -> uint8_t {
    let payload = Leaky::new(tryrc!(readptr!(payload_ptr, "Payload pointer")));

    tryrc!(payload.build());

    0
}

#[no_mangle]
pub extern "C" fn payload_run(payload_ptr: *mut Payload,
                              host_ptr: *mut Host,
                              ffi_user_args: *mut *const c_char,
                              ffi_user_args_len: size_t) -> uint8_t {
    let payload = Leaky::new(tryrc!(readptr!(payload_ptr, "Payload pointer")));
    let mut host = Leaky::new(tryrc!(readptr!(host_ptr, "Host pointer")));

    let user_args = if ffi_user_args.is_null() {
        None
    } else {
        let a: Vec<_> = tryrc!(catch_unwind(|| Ffi__Array {
            ptr: ffi_user_args,
            length: ffi_user_args_len,
            capacity: ffi_user_args_len,
        }.into()));
        let mut b = Vec::new();
        for ptr in a {
            b.push(tryrc!(ptrtostr!(ptr, "User arg string")));
        }
        Some(b)
    };

    tryrc!(payload.run(&mut host, user_args));

    0
}

#[no_mangle]
pub extern "C" fn payload_free(payload_ptr: *mut Payload) -> uint8_t {
    tryrc!(boxptr!(payload_ptr, "Payload pointer"));
    0
}

#[cfg(test)]
mod tests {
    use czmq::{ZSock, SocketType};
    use host::Host;
    use host::ffi::host_close;
    use payload::config::Config;
    use project::Language;
    use std::ffi::CString;
    use std::{fs, ptr, thread};
    use super::*;
    use tempdir::TempDir;
    use zdaemon::ConfigFile;

    #[test]
    fn test_new() {
        let _ = ::_MOCK_ENV.init();

        let tempdir = TempDir::new("test_payload_ffi_new").unwrap();
        let mut buf = tempdir.path().to_owned();

        buf.push("main.php");
        fs::File::create(&buf).expect("Failed to create main.php");
        buf.pop();

        let conf = Config {
            author: "Dr. Hibbert".into(),
            repository: "https://github.com/dhibbz/hehehe.git".into(),
            language: Language::Php,
            dependencies: None,
        };

        buf.push("payload.json");
        conf.save(&buf).unwrap();
        buf.pop();

        let payload_artifact = CString::new(buf.to_str().unwrap()).unwrap();
        let payload = payload_new(payload_artifact.as_ptr());
        assert!(!payload.is_null());
        assert_eq!(payload_free(payload), 0);
    }

    #[test]
    fn test_build() {
        let _ = ::_MOCK_ENV.init();

        let tempdir = TempDir::new("test_payload_ffi_build").unwrap();

        let conf = Config {
            author: "Dr. Hibbert".into(),
            repository: "https://github.com/dhibbz/hehehe.git".into(),
            language: Language::Php,
            dependencies: None,
        };

        let mut buf = tempdir.path().to_owned();
        buf.push("payload.json");
        conf.save(&buf).unwrap();
        buf.pop();

        let payload_artifact = CString::new(buf.to_str().unwrap()).unwrap();
        let payload_ptr = payload_new(payload_artifact.as_ptr());
        assert!(!payload_ptr.is_null());
        assert_eq!(payload_build(payload_ptr), 0);
        assert_eq!(payload_free(payload_ptr), 0);
    }

    #[test]
    fn test_build_run() {
        let _ = ::_MOCK_ENV.init();

        // These need to be run sequentially as env::set_current_dir
        // is not thread-safe.
        super::super::tests::test_build_c();
        super::super::tests::test_run();
        test_run();
    }

    // Don't run this test directly as there's a race condition
    // between mod::test_run and ffi::test_run.
    fn test_run() {
        let _ = ::_MOCK_ENV.init();

        let tempdir = TempDir::new("test_payload_run").unwrap();
        let mut buf = tempdir.path().to_owned();

        super::super::tests::create_cargo_proj(&mut buf);

        let conf = Config {
            author: "Dr. Hibbert".into(),
            repository: "https://github.com/dhibbz/hehehe.git".into(),
            language: Language::Rust,
            dependencies: None,
        };

        buf.push("payload.json");
        conf.save(&buf).unwrap();
        buf.pop();

        let payload_name = buf.into_os_string().into_string().unwrap();
        let payload_name_clone = payload_name.clone();

        let handle = thread::spawn(move || {
            let s = ZSock::new(SocketType::DEALER);
            s.connect(&format!("ipc://{}/main_api.ipc", payload_name_clone)).unwrap();
            s.recv_str().unwrap().unwrap();
        });

        let payload_artifact = CString::new(payload_name.as_bytes()).unwrap();
        let payload_ptr = payload_new(payload_artifact.as_ptr());
        assert!(!payload_ptr.is_null());

        let host = Box::into_raw(Box::new(Host::test_new(None, None, None, None)));
        assert_eq!(payload_run(payload_ptr, host, &mut ptr::null(), 0), 0);

        assert_eq!(payload_free(payload_ptr), 0);
        assert_eq!(host_close(host), 0);
        handle.join().unwrap();
    }
}
