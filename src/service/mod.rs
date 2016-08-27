// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The primitive for controlling services on a managed host.
//!
//! # Basic Usage
//!
//! Initialise a new Host using your managed host's IP address and
//! port number:
//!
//! ```no_run
//! # use inapi::Host;
//! let mut host = Host::new();
#![cfg_attr(feature = "remote-run", doc = "host.connect(\"myhost.example.com\", 7101, 7102, \"auth.example.com:7101\").unwrap();")]
//! ```
//!
//! Create a new Service to manage your daemon:
//!
//! ```no_run
//! # use inapi::{Host, Service, ServiceRunnable};
//! # let mut host = Host::new();
//! let service = Service::new_service(ServiceRunnable::Service("nginx"), None);
//! ```
//!
//! If your daemon uses a script instead of the service manager, just
//! change the ServiceRunnable type to Command:
//!
//! ```no_run
//! # use inapi::{Host, Service, ServiceRunnable};
//! # let mut host = Host::new();
//! let service = Service::new_service(ServiceRunnable::Command("/usr/bin/apachectl"), None);
//! ```
//!
//! Now you can run an action against the Service. This will
//! optionally return a CommandResult if the action was run, or None
//! if the service was already in the desired state.
//!
//! ```no_run
//! # use inapi::{Host, Service, ServiceRunnable};
//! # let mut host = Host::new();
//! # let service = Service::new_service(ServiceRunnable::Service(""), None);
//! let result = service.action(&mut host, "start").unwrap();
//! if let Some(r) = result {
//!     assert_eq!(r.exit_code, 0);
//! }
//! ```
//!
//! # Runnables
//!
//! Runnables are the executable items that a Service calls actions
//! on.
//!
//! The "Command" Runnable represents a script that is executed by
//! the shell. Wherever possible, use the "Service" Runnable. However
//! if you are managing a service without Systemd, Upstart, Init etc.
//! integration then you would use the Command Runnable.
//!
//! The "Service" Runnable represents a daemon managed by the
//! default system service manager. For example, on a Linux system
//! using Init, the "Service" Runnable is executed as:
//! "service <ServiceRunnable::Service> <action>"
//!
//! # Mapping Actions to Runnables
//!
//! The other way of initialising a Service is with the new_map()
//! function. This allows you to map individual actions to separate
//! ServiceRunnable types.
//!
//! ```no_run
//! # use inapi::{Host, Service, ServiceRunnable};
//! # use std::collections::HashMap;
//! # let mut host = Host::new();
//! let mut map = HashMap::new();
//! map.insert("start", ServiceRunnable::Command("/usr/local/bin/svc_start"));
//! map.insert("stop", ServiceRunnable::Command("/usr/local/bin/svc_stop"));
//! map.insert("restart", ServiceRunnable::Command("/usr/local/bin/svc_stop && /usr/local/bin/svc_start"));
//! let service = Service::new_map(map, None);
//! ```
//!
//! Sometimes you'll want to set a default Runnable as well as one or
//! more per-action Runnables. For example, you could map the
//! 'status' action to a Command Runnable while defaulting to the
//! Service Runnable for all other actions:
//!
//! ```no_run
//! # use inapi::{Host, Service, ServiceRunnable};
//! # use std::collections::HashMap;
//! # let mut host = Host::new();
//! let mut map = HashMap::new();
//! map.insert("_", ServiceRunnable::Service("my_svc")); // <-- Note that "_" (underscore) is the default key used by the Service map
//! map.insert("status", ServiceRunnable::Command("/usr/bin/my_svc_status"));
//! let service = Service::new_map(map, None);
//! ```
//!
//! Note that when mapping actions to Command Runnables, the action
//! name is not appended to the command, unless it is the default
//! Runnable:
//!
//! ```no_run
//! # use inapi::{Host, Service, ServiceRunnable};
//! # use std::collections::HashMap;
//! # let mut host = Host::new();
//! let mut map = HashMap::new();
//! map.insert("start", ServiceRunnable::Command("/usr/bin/start_svc"));
//! map.insert("kill", ServiceRunnable::Command("killall my_svc"));
//! map.insert("_", ServiceRunnable::Command("/usr/bin/svc_ctl"));
//! let service = Service::new_map(map, None);
//! service.action(&mut host, "start").unwrap(); // <-- Calls "/usr/bin/start_svc"
//! service.action(&mut host, "status").unwrap(); // <-- Calls "/usr/bin/svc_ctl status"
//! ```
//!
//! # Mapping Actions to Other Actions
//!
//! In an effort to standardise actions across platforms and daemons,
//! it is sometimes necessary to map an action to another action. For
//! example, you could use action mapping to refer to the command
//! flags "-s" and "-t" as "start" and "stop" respectively. This
//! makes your code more readable and helps you create cross-platform
//! code in the event that action names are different across
//! platforms:
//!
//! ```no_run
//! # use inapi::{Host, Service, ServiceRunnable};
//! # use std::collections::HashMap;
//! # let mut host = Host::new();
//! let mut map = HashMap::new();
//! map.insert("start", "-s"); // <-- Map action "start" to "-s"
//! map.insert("stop", "-t");
//! let service = Service::new_service(ServiceRunnable::Command("/usr/local/bin/my_svc"), Some(map));
//! service.action(&mut host, "start").unwrap(); // <-- Calls "/usr/local/bin/my_svc -s"
//! ```
//!
//! You could use the same technique to load a configuration file on
//! start, while leaving the other actions untouched:
//!
//! ```no_run
//! # use inapi::{Host, Service, ServiceRunnable};
//! # use std::collections::HashMap;
//! # let mut host = Host::new();
//! let mut map = HashMap::new();
//! map.insert("start", "-c /usr/local/etc/my_svc.conf");
//! let service = Service::new_service(ServiceRunnable::Command("/usr/local/bin/my_svc"), Some(map));
//! service.action(&mut host, "start").unwrap(); // <-- Calls "/usr/local/bin/my_svc -c /usr/local/etc/my_svc.conf"
//! service.action(&mut host, "stop").unwrap(); // <-- Calls "/usr/local/bin/my_svc stop"
//! ```

pub mod ffi;

use command::{CommandResult, CommandTarget};
use error::{Error, Result};
use host::Host;
use std::collections::HashMap;
use target::Target;

/// Runnables are the executable items that a Service calls actions
/// on.
pub enum ServiceRunnable<'a> {
    /// A script that is executed by the shell
    Command(&'a str),
    /// A daemon managed by the default system service manager
    Service(&'a str),
}

/// Container for managing a service.
pub struct Service<'a> {
    /// Actions map for Runnables
    actions: HashMap<&'a str, ServiceRunnable<'a>>,
    /// Action aliases map
    mapped_actions: Option<HashMap<&'a str, &'a str>>,
}

impl <'a>Service<'a> {
    /// Create a new Service with a single Runnable.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::{Service, ServiceRunnable};
    /// let service = Service::new_service(ServiceRunnable::Service("service_name"), None);
    /// ```
    pub fn new_service(runnable: ServiceRunnable<'a>, mapped_actions: Option<HashMap<&'a str, &'a str>>) -> Service<'a> {
        let mut actions = HashMap::new();
        actions.insert("_", runnable);
        Self::new_map(actions, mapped_actions)
    }

    /// Create a new Service with multiple Runnables.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::{Service, ServiceRunnable};
    /// # use std::collections::HashMap;
    /// let mut map = HashMap::new();
    /// map.insert("start", ServiceRunnable::Command("/usr/local/bin/nginx"));
    /// map.insert("stop", ServiceRunnable::Command("killall \"nginx: master process nginx\""));
    /// let service = Service::new_map(map, None);
    /// ```
    pub fn new_map(actions: HashMap<&'a str, ServiceRunnable<'a>>, mapped_actions: Option<HashMap<&'a str, &'a str>>) -> Service<'a> {
        Service {
            actions: actions,
            mapped_actions: mapped_actions,
        }
    }

    /// Run a service action, e.g. "start" or "stop".
    ///
    /// If the function returns Some(), the action was required to
    /// run in order to get the host into the required state. If the
    /// function returns None, the host is already in the required
    /// state.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::{Host, Service, ServiceRunnable};
    /// # let mut host = Host::new();
    /// let service = Service::new_service(ServiceRunnable::Command("/usr/bin/nginx"), None);
    /// service.action(&mut host, "start").unwrap();
    /// ```
    pub fn action(&self, host: &mut Host, action: &str) -> Result<Option<CommandResult>> {
        let mut action = action;

        // Exchange this action with a mapped action if possible
        if let Some(ref mapped) = self.mapped_actions {
            if mapped.contains_key(&action) {
                action = mapped.get(&action).unwrap();
            }
        }

        if self.actions.contains_key(&action) {
            self.run(host, action, self.actions.get(&action).unwrap(), false)
        } else if self.actions.contains_key(&"_") {
            self.run(host, action, self.actions.get(&"_").unwrap(), true)
        } else {
            Err(Error::Generic(format!("Unrecognised action {}", action)))
        }
    }

    fn run(&self, host: &mut Host, action: &str, runnable: &ServiceRunnable<'a>, default: bool) -> Result<Option<CommandResult>> {
        match runnable {
            &ServiceRunnable::Service(name) => Target::service_action(host, name, action),
            &ServiceRunnable::Command(cmd) => if default {
                Ok(Some(try!(Target::exec(host, &format!("{} {}", cmd, action)))))
            } else {
                Ok(Some(try!(Target::exec(host, cmd))))
            },
        }
    }
}

pub trait ServiceTarget {
    fn service_action(host: &mut Host, name: &str, action: &str) -> Result<Option<CommandResult>>;
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "remote-run")]
    use Host;
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    #[cfg(feature = "remote-run")]
    use super::*;
    #[cfg(feature = "remote-run")]
    use std::collections::HashMap;
    #[cfg(feature = "remote-run")]
    use std::thread;

    // XXX This requires mocking the shell or Command struct
    // #[cfg(feature = "local-run")]
    // #[test]
    // fn test_action() {
    // }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_default() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&server).unwrap();
            assert_eq!("service::action", req.popstr().unwrap().unwrap());
            assert_eq!("nginx", req.popstr().unwrap().unwrap());
            assert_eq!("start", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("Service started...").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();

            let req = ZMsg::recv(&server).unwrap();
            assert_eq!("service::action", req.popstr().unwrap().unwrap());
            assert_eq!("nginx", req.popstr().unwrap().unwrap());
            assert_eq!("start", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.send(&server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None);

        let service = Service::new_service(ServiceRunnable::Service("nginx"), None);

        let result = service.action(&mut host, "start").unwrap().unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Service started...");
        assert_eq!(result.stderr, "");

        let result = service.action(&mut host, "start").unwrap();
        assert!(result.is_none());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_map() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&server).unwrap();
            assert_eq!("service::action", msg.popstr().unwrap().unwrap());
            assert_eq!("nginx", msg.popstr().unwrap().unwrap());
            assert_eq!("start", msg.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("Service started...").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None);

        let mut map = HashMap::new();
        map.insert("start", ServiceRunnable::Service("nginx"));
        let service = Service::new_map(map, None);
        let result = service.action(&mut host, "start").unwrap().unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Service started...");
        assert_eq!(result.stderr, "");

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_mapped() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&server).unwrap();
            assert_eq!("service::action", msg.popstr().unwrap().unwrap());
            assert_eq!("nginx", msg.popstr().unwrap().unwrap());
            assert_eq!("load", msg.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("Service started...").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None);

        let mut map = HashMap::new();
        map.insert("start", "load");
        let service = Service::new_service(ServiceRunnable::Service("nginx"), Some(map));
        let result = service.action(&mut host, "start").unwrap().unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Service started...");
        assert_eq!(result.stderr, "");

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_error() {
        let mut host = Host::test_new(None, None, None);

        let mut map = HashMap::new();
        map.insert("start", ServiceRunnable::Service("nginx"));
        let service = Service::new_map(map, None);
        assert!(service.action(&mut host, "nonexistent").is_err());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_command() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&server).unwrap();
            assert_eq!("command::exec", msg.popstr().unwrap().unwrap());
            assert_eq!("/usr/local/bin/nginx", msg.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("Service started...").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None);

        let mut map = HashMap::new();
        map.insert("start", ServiceRunnable::Command("/usr/local/bin/nginx"));
        let service = Service::new_map(map, None);
        let result = service.action(&mut host, "start").unwrap().unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Service started...");
        assert_eq!(result.stderr, "");

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_command_mapped() {
        ZSys::init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&server).unwrap();
            assert_eq!("command::exec", msg.popstr().unwrap().unwrap());
            assert_eq!("/usr/local/bin/nginx -s", msg.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("Service started...").unwrap();
            rep.addstr("").unwrap();
            rep.send(&server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None);

        let mut map = HashMap::new();
        map.insert("start", "-s");
        let service = Service::new_service(ServiceRunnable::Command("/usr/local/bin/nginx"), Some(map));
        let result = service.action(&mut host, "start").unwrap().unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Service started...");
        assert_eq!(result.stderr, "");

        agent_mock.join().unwrap();
    }
}
