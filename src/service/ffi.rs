// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Service

use command::ffi::Ffi__CommandResult;
use ffi_helpers::{Ffi__Array, Leaky};
use host::Host;
use libc::{c_char, size_t, uint8_t};
use std::{convert, ptr};
use std::collections::HashMap;
use std::panic::catch_unwind;
use super::*;

#[repr(C)]
pub struct Ffi__ServiceRunnable {
    command: *mut c_char,
    service: *mut c_char,
}

impl <'a>convert::Into<ServiceRunnable<'a>> for Ffi__ServiceRunnable {
    fn into(self) -> ServiceRunnable<'a> {
        if !self.command.is_null() {
            ServiceRunnable::Command(trypanic!(ptrtostr!(self.command, "command string")))
        } else {
            ServiceRunnable::Service(trypanic!(ptrtostr!(self.service, "service string")))
        }
    }
}

#[repr(C)]
pub struct Ffi__ServiceAction {
    action: *mut c_char,
    runnable: Ffi__ServiceRunnable,
}

impl <'a>convert::Into<HashMap<&'a str, ServiceRunnable<'a>>> for Ffi__Array<Ffi__ServiceAction> {
    fn into(self) -> HashMap<&'a str, ServiceRunnable<'a>> {
        let actions_vec: Vec<_> = self.into();

        let mut actions = HashMap::new();
        for action in actions_vec {
            actions.insert(
                trypanic!(ptrtostr!(action.action, "action string")),
                action.runnable.into()
            );
        }

        actions
    }
}

#[repr(C)]
pub struct Ffi__ServiceMappedAction {
    action: *mut c_char,
    mapped_action: *mut c_char,
}

impl <'a>convert::Into<HashMap<&'a str, &'a str>> for Ffi__Array<Ffi__ServiceMappedAction> {
    fn into(self) -> HashMap<&'a str, &'a str> {
        let actions_vec: Vec<_> = self.into();

        let mut actions = HashMap::new();
        for action in actions_vec {
            actions.insert(
                trypanic!(ptrtostr!(action.action, "action string")),
                trypanic!(ptrtostr!(action.mapped_action, "mapped_action string")),
            );
        }

        actions
    }
}

#[no_mangle]
pub extern "C" fn service_new_service(ffi_runnable: Ffi__ServiceRunnable,
                                      mapped_actions: *mut Ffi__ServiceMappedAction,
                                      mapped_actions_len: size_t) -> *mut Service {
    let runnable: ServiceRunnable = trynull!(catch_unwind(|| ffi_runnable.into()));
    let mapped_actions = if !mapped_actions.is_null() {
        Some(trynull!(catch_unwind(|| Ffi__Array {
            ptr: mapped_actions,
            length: mapped_actions_len,
            capacity: mapped_actions_len,
        }.into())))
    } else {
        None
    };

    let svc = Service::new_service(runnable, mapped_actions);
    Box::into_raw(Box::new(svc))
}

#[no_mangle]
pub extern "C" fn service_new_map(actions_ptr: *mut Ffi__ServiceAction,
                                  actions_len: size_t,
                                  mapped_actions_ptr: *mut Ffi__ServiceMappedAction,
                                  mapped_actions_len: size_t) -> *mut Service {
    let actions: HashMap<_, _> = trynull!(catch_unwind(|| Ffi__Array {
        ptr: actions_ptr,
        length: actions_len,
        capacity: actions_len,
    }.into()));
    let mapped_actions = if !mapped_actions_ptr.is_null() {
        Some(trynull!(catch_unwind(|| Ffi__Array {
            ptr: mapped_actions_ptr,
            length: mapped_actions_len,
            capacity: mapped_actions_len,
        }.into())))
    } else {
        None
    };

    let svc = Service::new_map(actions, mapped_actions);
    Box::into_raw(Box::new(svc))
}

#[no_mangle]
pub extern "C" fn service_action(service_ptr: *mut Service, host_ptr: *const Host, action_ptr: *const c_char) -> *const Ffi__CommandResult {
    let service = Leaky::new(trynull!(readptr!(service_ptr, "Service pointer")));
    let mut host = Leaky::new(trynull!(readptr!(host_ptr, "Host pointer")));
    let action = trynull!(ptrtostr!(action_ptr, "action string"));

    match trynull!(service.action(&mut host, action)) {
        Some(result) => {
            let ffi_r = trynull!(catch_unwind(|| result.into()));
            Box::into_raw(Box::new(ffi_r))
        },
        None => ptr::null(),
    }
}

#[no_mangle]
pub extern "C" fn service_free(service_ptr: *mut Service) -> uint8_t {
    tryrc!(boxptr!(service_ptr, "Service pointer"));
    0
}

#[cfg(test)]
mod tests {
    use ffi_helpers::Ffi__Array;
    #[cfg(feature = "remote-run")]
    use Host;
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    #[cfg(feature = "remote-run")]
    use host::ffi::host_close;
    use service::{ServiceRunnable, ServiceRunnableOwned};
    use std::collections::HashMap;
    #[cfg(feature = "remote-run")]
    use std::ffi::CStr;
    use std::ffi::CString;
    use std::{ptr, str};
    #[cfg(feature = "remote-run")]
    use std::thread;
    use super::*;

    #[test]
    fn test_convert_ffi_service_runnable_cmd() {
        let runnable: ServiceRunnable = Ffi__ServiceRunnable {
            command: CString::new("test").unwrap().into_raw(),
            service: ptr::null_mut(),
        }.into();

        match runnable {
            ServiceRunnable::Command(cmd) => assert_eq!(cmd, "test"),
            _ => panic!("Unexpected Runnable variant"),
        }
    }

    #[test]
    fn test_convert_ffi_service_runnable_svc() {
        let runnable: ServiceRunnable = Ffi__ServiceRunnable {
            command: ptr::null_mut(),
            service: CString::new("test").unwrap().into_raw(),
        }.into();

        match runnable {
            ServiceRunnable::Service(svc) => assert_eq!(svc, "test"),
            _ => panic!("Unexpected Runnable variant"),
        }
    }

    #[test]
    fn test_convert_service_actions() {
        let mut arr = Vec::new();
        arr.push(Ffi__ServiceAction {
            action: CString::new("test").unwrap().into_raw(),
            runnable: Ffi__ServiceRunnable {
                command: ptr::null_mut(),
                service: CString::new("test").unwrap().into_raw(),
            },
        });
        let ffi_actions = Ffi__Array::from(arr);
        let actions: HashMap<_, _> = ffi_actions.into();

        match *actions.get("test").unwrap() {
            ServiceRunnable::Service(svc) => assert_eq!(svc, "test"),
            _ => panic!("Unexpected Runnable variant"),
        }
    }

    #[test]
    fn test_convert_service_mapped_actions() {
        let mut arr = Vec::new();
        arr.push(Ffi__ServiceMappedAction {
            action: CString::new("test").unwrap().into_raw(),
            mapped_action: CString::new("mapped_test").unwrap().into_raw(),
        });
        let ffi_actions = Ffi__Array::from(arr);
        let actions: HashMap<_, _> = ffi_actions.into();

        assert_eq!(actions.get("test").unwrap(), &"mapped_test");
    }

    #[test]
    fn test_service_new_service() {
        let runnable = Ffi__ServiceRunnable {
            command: ptr::null_mut(),
            service: CString::new("test").unwrap().into_raw(),
        };

        let mut arr = Vec::new();
        arr.push(Ffi__ServiceMappedAction {
            action: CString::new("test").unwrap().into_raw(),
            mapped_action: CString::new("mapped_test").unwrap().into_raw(),
        });
        let ffi_mapped = Ffi__Array::from(arr);

        let service = readptr!(service_new_service(runnable, ffi_mapped.ptr, ffi_mapped.length), "Service pointer").unwrap();

        match *service.actions.get("_").unwrap() {
            ServiceRunnableOwned::Service(ref svc) => assert_eq!(svc, "test"),
            _ => panic!("Unexpected Runnable variant"),
        }

        assert_eq!(service.mapped_actions.unwrap().get("test").unwrap(), &"mapped_test");
    }

    #[test]
    fn test_service_new_map() {
        let mut arr = Vec::new();
        arr.push(Ffi__ServiceAction {
            action: CString::new("test").unwrap().into_raw(),
            runnable: Ffi__ServiceRunnable {
                command: CString::new("test").unwrap().into_raw(),
                service: ptr::null_mut(),
            },
        });
        let ffi_map = Ffi__Array::from(arr);

        let service = readptr!(service_new_map(ffi_map.ptr, ffi_map.length, ptr::null_mut(), 0), "Service pointer").unwrap();

        match *service.actions.get("test").unwrap() {
            ServiceRunnableOwned::Command(ref cmd) => assert_eq!(cmd, "test"),
            _ => panic!("Unexpected Runnable variant"),
        }

        assert!(service.mapped_actions.is_none());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_service_action() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("Service started...").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.send(&mut server).unwrap();
        });

        let host = Box::into_raw(Box::new(Host::test_new(None, Some(client), None, None)));

        let runnable = Ffi__ServiceRunnable {
            command: ptr::null_mut(),
            service: CString::new("nginx").unwrap().into_raw(),
        };

        let service = service_new_service(runnable, ptr::null_mut(), 0);
        assert!(!service.is_null());

        let action_ptr = CString::new("start").unwrap().into_raw();
        let result = readptr!(service_action(service, host, action_ptr), "CommandResult pointer").unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(unsafe { CStr::from_ptr(result.stdout).to_str().unwrap() }, "Service started...");
        assert_eq!(unsafe { CStr::from_ptr(result.stderr).to_str().unwrap() }, "");

        assert!(service_action(service, host, action_ptr).is_null());

        assert_eq!(service_free(service), 0);
        assert_eq!(host_close(host), 0);
        agent_mock.join().unwrap();
    }
}
