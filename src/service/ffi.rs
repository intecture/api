// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! FFI interface for Service

use command::ffi::Ffi__CommandResult;
use ffi_helpers::Ffi__Array;
use Host;
use host::ffi::Ffi__Host;
use libc::{c_char, size_t};
use std::{convert, ptr, str};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
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

impl <'a>convert::From<Ffi__ServiceRunnable> for ServiceRunnable<'a> {
    fn from(ffi_runnable: Ffi__ServiceRunnable) -> ServiceRunnable<'a> {
        if ffi_runnable.command != ptr::null_mut() {
            ServiceRunnable::Command(str::from_utf8(unsafe { CStr::from_ptr(ffi_runnable.command) }.to_bytes()).unwrap())
        } else {
            ServiceRunnable::Service(str::from_utf8(unsafe { CStr::from_ptr(ffi_runnable.service) }.to_bytes()).unwrap())
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

// Because we can't implement an un-owned trait on an un-owned
// struct, we have to roll our own fn.
fn convert_from_actions<'a>(ffi_actions: Ffi__Array<Ffi__ServiceAction>) -> HashMap<&'a str, ServiceRunnable<'a>> {
    let actions_vec = unsafe { Vec::from_raw_parts(ffi_actions.ptr, ffi_actions.length, ffi_actions.capacity) };
    let mut actions = HashMap::new();
    for action in actions_vec {
        actions.insert(
            str::from_utf8(unsafe { CStr::from_ptr(action.action) }.to_bytes()).unwrap(),
            ServiceRunnable::from(action.runnable)
        );
    }

    actions
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

// Because we can't implement an un-owned trait on an un-owned
// struct, we have to roll our own fn.
fn convert_from_mapped_actions<'a>(ffi_actions: Ffi__Array<Ffi__ServiceMappedAction>) -> HashMap<&'a str, &'a str> {
    let actions_vec = unsafe { Vec::from_raw_parts(ffi_actions.ptr, ffi_actions.length, ffi_actions.capacity) };
    let mut actions = HashMap::new();
    for action in actions_vec {
        actions.insert(
            str::from_utf8(unsafe { CStr::from_ptr(action.action) }.to_bytes()).unwrap(),
            str::from_utf8(unsafe { CStr::from_ptr(action.mapped_action) }.to_bytes()).unwrap(),
        );
    }

    actions
}

#[repr(C)]
pub struct Ffi__Service {
    actions: Ffi__Array<Ffi__ServiceAction>,
    mapped_actions: Option<Ffi__Array<Ffi__ServiceMappedAction>>,
}

impl <'a>convert::From<Service<'a>> for Ffi__Service {
    fn from(service: Service) -> Ffi__Service {
        Ffi__Service {
            actions: Ffi__Array::from(service.actions),
            mapped_actions: if let Some(mapped) = service.mapped_actions {
                Some(Ffi__Array::from(mapped))
            } else {
                None
            },
        }
    }
}

impl <'a>convert::From<Ffi__Service> for Service<'a> {
    fn from(ffi_service: Ffi__Service) -> Service<'a> {
        Service {
            actions: convert_from_actions(ffi_service.actions),
            mapped_actions: if let Some(mapped) = ffi_service.mapped_actions {
                Some(convert_from_mapped_actions(mapped))
            } else {
                None
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn service_new_service(ffi_runnable: Ffi__ServiceRunnable, ffi_mapped_actions: *mut Ffi__ServiceMappedAction, mapped_actions_len: size_t) -> Ffi__Service {
    let runnable = ServiceRunnable::from(ffi_runnable);
    let mapped_actions = if ffi_mapped_actions != ptr::null_mut() {
        Some(convert_from_mapped_actions(Ffi__Array {
            ptr: ffi_mapped_actions,
            length: mapped_actions_len,
            capacity: mapped_actions_len,
        }))
    } else {
        None
    };

    Ffi__Service::from(Service::new_service(runnable, mapped_actions))
}

#[no_mangle]
pub extern "C" fn service_new_map(ffi_actions: *mut Ffi__ServiceAction, actions_len: size_t, ffi_mapped_actions: *mut Ffi__ServiceMappedAction, mapped_actions_len: size_t) -> Ffi__Service {
    let actions = convert_from_actions(Ffi__Array {
        ptr: ffi_actions,
        length: actions_len,
        capacity: actions_len,
    });
    let mapped_actions = if ffi_mapped_actions != ptr::null_mut() {
        Some(convert_from_mapped_actions(Ffi__Array {
            ptr: ffi_mapped_actions,
            length: mapped_actions_len,
            capacity: mapped_actions_len,
        }))
    } else {
        None
    };

    Ffi__Service::from(Service::new_map(actions, mapped_actions))
}

#[no_mangle]
pub extern "C" fn service_action(ffi_service_ptr: *mut Ffi__Service, ffi_host_ptr: *const Ffi__Host, action_ptr: *const c_char) -> Ffi__CommandResult {
    let service = Service::from(unsafe { ptr::read(ffi_service_ptr) });
    let mut host = Host::from(unsafe { ptr::read(ffi_host_ptr) });
    let action = unsafe { str::from_utf8(CStr::from_ptr(action_ptr).to_bytes()).unwrap() };

    let result = Ffi__CommandResult::from(service.action(&mut host, action).unwrap());

    // When we convert the FFI service pointer into a Service, we
    // convert the C string pointers into &str's, which have the same
    // lifetime as the service binding. To avoid freeing this memory
    // when the binding goes out of scope, we convert the Service
    // back to an FFI Service and write it to the pointer.
    unsafe { ptr::write(&mut *ffi_service_ptr, Ffi__Service::from(service)); }

    // Convert ZMQ socket to raw to avoid destructor closing sock
    Ffi__Host::from(host);

    result
}

#[cfg(test)]
mod tests {
    use ffi_helpers::Ffi__Array;
    #[cfg(feature = "remote-run")]
    use Host;
    use host::ffi::Ffi__Host;
    use {Service, ServiceRunnable};
    use std::collections::HashMap;
    use std::ffi::{CStr, CString};
    use std::{ptr, str};
    #[cfg(feature = "remote-run")]
    use std::thread;
    use super::*;
    use super::{convert_from_actions, convert_from_mapped_actions};
    #[cfg(feature = "remote-run")]
    use zmq;

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
        let runnable = ServiceRunnable::from(Ffi__ServiceRunnable {
            command: CString::new("test").unwrap().into_raw(),
            service: ptr::null_mut(),
        });

        match runnable {
            ServiceRunnable::Command(cmd) => assert_eq!(cmd, "test"),
            _ => panic!("Unexpected Runnable variant"),
        }
    }

    #[test]
    fn test_convert_ffi_service_runnable_svc() {
        let runnable = ServiceRunnable::from(Ffi__ServiceRunnable {
            command: ptr::null_mut(),
            service: CString::new("test").unwrap().into_raw(),
        });

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
        actions = convert_from_actions(ffi_actions);

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
        actions = convert_from_mapped_actions(ffi_actions);

        assert_eq!(actions.get("test").unwrap(), &"mapped_test");
    }

    #[test]
    fn test_convert_service() {
        let mut actions = HashMap::new();
        actions.insert("test", ServiceRunnable::Service("test"));

        let mut mapped = HashMap::new();
        mapped.insert("test", "mapped_test");

        let mut service = Service {
            actions: actions,
            mapped_actions: Some(mapped),
        };

        let ffi_service = Ffi__Service::from(service);
        service = Service::from(ffi_service);

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

        let ffi_service = service_new_service(ffi_runnable, ffi_mapped.ptr, ffi_mapped.length);
        let service = Service::from(ffi_service);

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

        let ffi_service = service_new_map(ffi_map.ptr, ffi_map.length, ptr::null_mut(), 0);
        let service = Service::from(ffi_service);

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
        let mut ctx = zmq::Context::new();

        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_action").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("service::action", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("nginx", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("start", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Service started...", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.connect("inproc://test_action").unwrap();

        let ffi_host = Ffi__Host::from(Host::test_new(None, Some(sock), None, None));
        let mut ffi_service = service_new_service(Ffi__ServiceRunnable::from(ServiceRunnable::Service("nginx")), ptr::null_mut(), 0);
        let result = service_action(&mut ffi_service as *mut Ffi__Service, &ffi_host as *const Ffi__Host, CString::new("start").unwrap().into_raw());

        assert_eq!(result.exit_code, 0);

        let stdout = unsafe { str::from_utf8(CStr::from_ptr(result.stdout).to_bytes()).unwrap() };
        assert_eq!(stdout, "Service started...");

        let stderr = unsafe { str::from_utf8(CStr::from_ptr(result.stderr).to_bytes()).unwrap() };
        assert_eq!(stderr, "");

        Host::from(ffi_host);

        agent_mock.join().unwrap();
    }
}
