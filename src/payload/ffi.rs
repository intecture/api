// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use ffi_helpers::{Ffi__Array, Leaky};
use host::Host;
use libc::{c_char, size_t, uint8_t};
use project::Language;
use std::{convert, ptr};
use std::ffi::CString;
use std::panic::catch_unwind;
use std::path::PathBuf;
use super::*;

#[repr(C)]
pub struct Ffi__Payload {
    path: *const c_char,
    artifact: *const c_char,
    language: Language,
}

impl convert::From<Payload> for Ffi__Payload {
    fn from(payload: Payload) -> Ffi__Payload {
        Ffi__Payload {
            path: CString::new(payload.path.to_str().unwrap()).unwrap().into_raw(),
            artifact: if let Some(a) = payload.artifact {
                CString::new(a).unwrap().into_raw()
            } else {
                ptr::null()
            },
            language: payload.language,
        }
    }
}

impl convert::Into<Payload> for Ffi__Payload {
    fn into(self) -> Payload {
        let path: String = trypanic!(ptrtostr!(self.path, "path string")).into();

        Payload {
            path: PathBuf::from(&path),
            artifact: if self.artifact.is_null() {
                None
            } else {
                Some(trypanic!(ptrtostr!(self.artifact, "artifact string")).into())
            },
            language: self.language,
        }
    }
}

#[no_mangle]
pub extern "C" fn payload_new(payload_artifact_ptr: *const c_char) -> *mut Ffi__Payload {
    let payload_artifact = trynull!(ptrtostr!(payload_artifact_ptr, "payload::artifact string"));
    let payload = trynull!(Payload::new(&payload_artifact));
    let ffi_payload: Ffi__Payload = trynull!(catch_unwind(|| payload.into()));
    Box::into_raw(Box::new(ffi_payload))
}

#[no_mangle]
pub extern "C" fn payload_build(ffi_payload_ptr: *mut Ffi__Payload) -> uint8_t {
    let payload = tryrc!(readptr!(ffi_payload_ptr; Payload, "Payload struct"));
    tryrc!(payload.build());
    0
}

#[no_mangle]
pub extern "C" fn payload_run(ffi_payload_ptr: *mut Ffi__Payload,
                              host_ptr: *mut Host,
                              ffi_user_args: *mut *const c_char,
                              ffi_user_args_len: size_t) -> uint8_t {
    let payload = tryrc!(readptr!(ffi_payload_ptr; Payload, "Payload struct"));
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

#[cfg(test)]
mod tests {
    use czmq::{ZSock, ZSockType};
    use host::Host;
    use payload::config::Config;
    use project::Language;
    use std::ffi::CString;
    use std::{fs, ptr, thread};
    use std::path::PathBuf;
    use std::process::Command;
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
        assert!(!payload_new(payload_artifact.as_ptr()).is_null());
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
    }

    #[test]
    fn test_run() {
        let _ = ::_MOCK_ENV.init();

        let tempdir = TempDir::new("test_payload_run").unwrap();
        let mut buf = tempdir.path().to_owned();

        create_cargo_proj(&mut buf);

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
            let s = ZSock::new(ZSockType::DEALER);
            s.connect(&format!("ipc://{}/main_api.ipc", payload_name_clone)).unwrap();
            s.recv_str().unwrap().unwrap();
        });

        let payload_artifact = CString::new(payload_name.as_bytes()).unwrap();
        let payload_ptr = payload_new(payload_artifact.as_ptr());
        assert!(!payload_ptr.is_null());

        let mut host = Host::test_new(None, None, None, None);
        assert_eq!(payload_run(payload_ptr, &mut host, &mut ptr::null(), 0), 0);

        handle.join().unwrap();
    }

    fn create_cargo_proj(buf: &mut PathBuf) {
        let status = Command::new("cargo")
                             .args(&["init", buf.to_str().unwrap(), "--bin", "--name", "payload"])
                             .status()
                             .expect("Failed to execute process");

        assert!(status.success());
    }
}
