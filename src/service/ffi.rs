// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Service

use command::ffi::Ffi__CommandResult;
use ffi_helpers::Ffi__Array;
use host::Host;
use host::ffi::Ffi__Host;
use libc::{c_char, size_t};
use std::{convert, ptr};
use std::collections::HashMap;
use std::ffi::CString;
use std::panic::catch_unwind;
use super::*;

#[repr(C)]
pub struct Ffi__ServiceRunnable {
    command: *mut c_char,
    service: *mut c_char,
}

impl <'a>convert::From<ServiceRunnable<'a>> for Ffi__ServiceRunnable {
    fn from(runnable: ServiceRunnable) -> Ffi__ServiceRunnable {
        match runnable {
            ServiceRunnable::Command(cmd) => Ffi__ServiceRunnable {
                command: CString::new(cmd).unwrap().into_raw(),
                service: ptr::null_mut(),
            },
            ServiceRunnable::Service(svc) => Ffi__ServiceRunnable {
                command: ptr::null_mut(),
                service: CString::new(svc).unwrap().into_raw(),
            },
        }
    }
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

impl <'a>convert::From<HashMap<&'a str, ServiceRunnable<'a>>> for Ffi__Array<Ffi__ServiceAction> {
    fn from(map: HashMap<&'a str, ServiceRunnable<'a>>) -> Ffi__Array<Ffi__ServiceAction> {
        let mut arr = Vec::new();
        let mut map = map;

        for (action, runnable) in map.drain() {
            arr.push(Ffi__ServiceAction {
                action: CString::new(action).unwrap().into_raw(),
                runnable: Ffi__ServiceRunnable::from(runnable),
            });
        }

        Ffi__Array::from(arr)
    }
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

impl <'a>convert::From<HashMap<&'a str, &'a str>> for Ffi__Array<Ffi__ServiceMappedAction> {
    fn from(map: HashMap<&'a str, &'a str>) -> Ffi__Array<Ffi__ServiceMappedAction> {
        let mut arr = Vec::new();
        let mut map = map;

        for (action, mapped_action) in map.drain() {
            arr.push(Ffi__ServiceMappedAction {
                action: CString::new(action).unwrap().into_raw(),
                mapped_action: CString::new(mapped_action).unwrap().into_raw(),
            });
        }

        Ffi__Array::from(arr)
    }
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

#[repr(C)]
pub struct Ffi__Service {
    actions: Ffi__Array<Ffi__ServiceAction>,
    mapped_actions: *const Ffi__Array<Ffi__ServiceMappedAction>,
}

impl <'a>convert::From<Service<'a>> for Ffi__Service {
    fn from(service: Service) -> Ffi__Service {
        Ffi__Service {
            actions: Ffi__Array::from(service.actions),
            mapped_actions: if let Some(mapped) = service.mapped_actions {
                Box::into_raw(Box::new(Ffi__Array::from(mapped)))
            } else {
                ptr::null()
            },
        }
    }
}

impl <'a>convert::Into<Service<'a>> for Ffi__Service {
    fn into(self) -> Service<'a> {
        Service {
            actions: self.actions.into(),
            mapped_actions: if self.mapped_actions.is_null() {
                None
            } else {
                Some(trypanic!(readptr!(self.mapped_actions, "mapped_actions array")))
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn service_new_service(ffi_runnable: Ffi__ServiceRunnable,
                                      mapped_actions: *mut Ffi__ServiceMappedAction,
                                      mapped_actions_len: size_t) -> *mut Ffi__Service {
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

    let ffi_svc = trynull!(catch_unwind(|| Service::new_service(runnable, mapped_actions).into()));
    Box::into_raw(Box::new(ffi_svc))
}

#[no_mangle]
pub extern "C" fn service_new_map(actions_ptr: *mut Ffi__ServiceAction,
                                  actions_len: size_t,
                                  mapped_actions_ptr: *mut Ffi__ServiceMappedAction,
                                  mapped_actions_len: size_t) -> *mut Ffi__Service {
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

    let ffi_svc = trynull!(catch_unwind(|| Service::new_map(actions, mapped_actions).into()));
    Box::into_raw(Box::new(ffi_svc))
}

#[no_mangle]
pub extern "C" fn service_action(service_ptr: *mut Ffi__Service, host_ptr: *const Ffi__Host, action_ptr: *const c_char) -> *const Ffi__CommandResult {
    let service: Service = trynull!(readptr!(service_ptr, "Service struct"));
    let mut host: Host = trynull!(readptr!(host_ptr, "Host struct"));
    let action = trynull!(ptrtostr!(action_ptr, "action string"));

    let result = match trynull!(service.action(&mut host, action)) {
        Some(result) => {
            let ffi_r = trynull!(catch_unwind(|| result.into()));
            Box::into_raw(Box::new(ffi_r))
        },
        None => ptr::null(),
    };

    // When we convert the FFI service pointer into a Service, we
    // convert the C string pointers into &str's, which have the same
    // lifetime as the service binding. To avoid freeing this memory
    // when the binding goes out of scope, we convert the Service
    // back to an FFI Service and write it to the pointer.
    let ffi_svc = trynull!(catch_unwind(|| service.into()));
    unsafe { ptr::write(&mut *service_ptr, ffi_svc); }

    result
}

#[cfg(test)]
mod tests {
    use ffi_helpers::Ffi__Array;
    #[cfg(feature = "remote-run")]
    use Host;
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    #[cfg(feature = "remote-run")]
    use host::ffi::Ffi__Host;
    #[cfg(feature = "remote-run")]
    use host::ffi::host_close;
    use {Service, ServiceRunnable};
    use std::collections::HashMap;
    use std::ffi::{CStr, CString};
    use std::{ptr, str};
    #[cfg(feature = "remote-run")]
    use std::thread;
    use super::*;

    #[test]
    fn test_convert_service_runnable_cmd() {
        let ffi_runnable = Ffi__ServiceRunnable::from(ServiceRunnable::Command("test"));
        assert_eq!(str::from_utf8(unsafe { CStr::from_ptr(ffi_runnable.command) }.to_bytes()).unwrap(), "test");
        assert_eq!(ffi_runnable.service, ptr::null_mut());
    }

    #[test]
    fn test_convert_service_runnable_svc() {
        let ffi_runnable = Ffi__ServiceRunnable::from(ServiceRunnable::Service("test"));
        assert_eq!(str::from_utf8(unsafe { CStr::from_ptr(ffi_runnable.service) }.to_bytes()).unwrap(), "test");
        assert_eq!(ffi_runnable.command, ptr::null_mut());
    }

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
        let mut actions = HashMap::new();
        actions.insert("test", ServiceRunnable::Service("test"));

        let ffi_actions = Ffi__Array::from(actions);
        let actions: HashMap<_, _> = ffi_actions.into();

        match actions.get("test").unwrap() {
            &ServiceRunnable::Service(svc) => assert_eq!(svc, "test"),
            _ => panic!("Unexpected Runnable variant"),
        }
    }

    #[test]
    fn test_convert_service_mapped_actions() {
        let mut actions = HashMap::new();
        actions.insert("test", "mapped_test");

        let ffi_actions = Ffi__Array::from(actions);
        let actions: HashMap<_, _> = ffi_actions.into();

        assert_eq!(actions.get("test").unwrap(), &"mapped_test");
    }

    #[test]
    fn test_convert_service() {
        let mut actions = HashMap::new();
        actions.insert("test", ServiceRunnable::Service("test"));

        let mut mapped = HashMap::new();
        mapped.insert("test", "mapped_test");

        let ffi_service: Ffi__Service = Service {
            actions: actions,
            mapped_actions: Some(mapped),
        }.into();
        let service: Service = ffi_service.into();

        match service.actions.get("test").unwrap() {
            &ServiceRunnable::Service(svc) => assert_eq!(svc, "test"),
            _ => panic!("Unexpected Runnable variant"),
        }

        assert_eq!(service.mapped_actions.unwrap().get("test").unwrap(), &"mapped_test");
    }

    #[test]
    fn test_service_new_service() {
        let ffi_runnable = Ffi__ServiceRunnable::from(ServiceRunnable::Service("test"));

        let mut mapped = HashMap::new();
        mapped.insert("test", "mapped_test");
        let ffi_mapped = Ffi__Array::from(mapped);

        let service: Service = readptr!(service_new_service(ffi_runnable, ffi_mapped.ptr, ffi_mapped.length), "Service struct").unwrap();

        match service.actions.get("_").unwrap() {
            &ServiceRunnable::Service(svc) => assert_eq!(svc, "test"),
            _ => panic!("Unexpected Runnable variant"),
        }

        assert_eq!(service.mapped_actions.unwrap().get("test").unwrap(), &"mapped_test");
    }

    #[test]
    fn test_service_new_map() {
        let mut map = HashMap::new();
        map.insert("test", ServiceRunnable::Command("test"));
        let ffi_map = Ffi__Array::from(map);

        let service: Service = readptr!(service_new_map(ffi_map.ptr, ffi_map.length, ptr::null_mut(), 0), "Service struct").unwrap();

        match service.actions.get("test").unwrap() {
            &ServiceRunnable::Command(cmd) => assert_eq!(cmd, "test"),
            _ => panic!("Unexpected Runnable variant"),
        }

        assert!(service.mapped_actions.is_none());
    }

    // XXX This requires mocking the shell or Command struct
    // #[cfg(feature = "local-run")]
    // #[test]
    // fn test_service_action() {
    // }

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

        let mut ffi_host = Ffi__Host::from(Host::test_new(None, Some(client), None, None));
        let ffi_service = service_new_service(Ffi__ServiceRunnable::from(ServiceRunnable::Service("nginx")), ptr::null_mut(), 0);
        assert!(!ffi_service.is_null());
        let action_ptr = CString::new("start").unwrap().into_raw();

        let result_ptr = service_action(ffi_service, &ffi_host, action_ptr);
        assert!(!result_ptr.is_null());
        let result = unsafe { ptr::read(result_ptr) };
        assert_eq!(result.exit_code, 0);
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(result.stdout).to_bytes()).unwrap() }, "Service started...");
        assert_eq!(unsafe { str::from_utf8(CStr::from_ptr(result.stderr).to_bytes()).unwrap() }, "");

        let result_ptr = service_action(ffi_service, &ffi_host, action_ptr);
        assert!(result_ptr.is_null());

        assert_eq!(host_close(&mut ffi_host), 0);
        agent_mock.join().unwrap();
    }
}
