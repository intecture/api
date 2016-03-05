// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
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
#![cfg_attr(feature = "remote-run", doc = " host.connect(\"127.0.0.1\", 7101, 7102, 7103).unwrap();")]
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
//! Now you can run an action against the Service:
//!
//! ```no_run
//! # use inapi::{Host, Service, ServiceRunnable};
//! # let mut host = Host::new();
//! # let service = Service::new_service(ServiceRunnable::Service(""), None);
//! let result = service.action(&mut host, "start").unwrap();
//! assert_eq!(result.exit_code, 0);
//! ```
//!
//! # Runnables
//!
//! Runnables are the executable items that a Service calls actions
//! on.
//!
//! The "Service" Runnable represents a daemon managed by the
//! default system service manager. For example, on a Linux system
//! using Init, the "Service" Runnable is executed as:
//! "service <ServiceRunnable::Service> <action>"
//!
//! The "Command" Runnable represents a script that is executed by
//! the shell. Wherever possible, use the "Service" Runnable. However
//! if you are managing a service without Systemd, Upstart, Init etc.
//! integration then you would use the Command Runnable.
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

// pub mod ffi;

use {CommandResult, Error, Host, Result};
use command::CommandTarget;
use std::collections::HashMap;
use target::Target;

pub enum ServiceRunnable<'a> {
    Service(&'a str),
    Command(&'a str),
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
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::{Host, Service, ServiceRunnable};
    /// # let mut host = Host::new();
    /// let service = Service::new_service(ServiceRunnable::Command("/usr/bin/nginx"), None);
    /// service.action(&mut host, "start").unwrap();
    /// ```
    pub fn action(&self, host: &mut Host, action: &str) -> Result<CommandResult> {
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

    fn run(&self, host: &mut Host, action: &str, runnable: &ServiceRunnable<'a>, default: bool) -> Result<CommandResult> {
        match runnable {
            &ServiceRunnable::Service(name) => Target::service_action(host, name, action),
            &ServiceRunnable::Command(cmd) => if default {
                Target::exec(host, &format!("{} {}", cmd, action))
            } else {
                Target::exec(host, cmd)
            },
        }
    }
}

pub trait ServiceTarget {
    fn service_action(host: &mut Host, name: &str, action: &str) -> Result<CommandResult>;
}

#[cfg(test)]
mod tests {
    use Host;
    use super::*;
    #[cfg(feature = "remote-run")]
    use std::collections::HashMap;
    #[cfg(feature = "remote-run")]
    use std::thread;
    #[cfg(feature = "remote-run")]
    use zmq;

    // XXX This requires mocking the shell or Command struct
    // #[cfg(feature = "local-run")]
    // #[test]
    // fn test_action() {
    // }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_default() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

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
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let service = Service::new_service(ServiceRunnable::Service("nginx"), None);
        let result = service.action(&mut host, "start").unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Service started...");
        assert_eq!(result.stderr, "");

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_map() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

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
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let mut map = HashMap::new();
        map.insert("start", ServiceRunnable::Service("nginx"));
        let service = Service::new_map(map, None);
        let result = service.action(&mut host, "start").unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Service started...");
        assert_eq!(result.stderr, "");

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_mapped() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("service::action", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("nginx", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("load", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Service started...", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let mut map = HashMap::new();
        map.insert("start", "load");
        let service = Service::new_service(ServiceRunnable::Service("nginx"), Some(map));
        let result = service.action(&mut host, "start").unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Service started...");
        assert_eq!(result.stderr, "");

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_error() {
        let mut host = Host::test_new(None, None, None, None);

        let mut map = HashMap::new();
        map.insert("start", ServiceRunnable::Service("nginx"));
        let service = Service::new_map(map, None);
        assert!(service.action(&mut host, "nonexistent").is_err());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_command() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/usr/local/bin/nginx", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Service started...", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let mut map = HashMap::new();
        map.insert("start", ServiceRunnable::Command("/usr/local/bin/nginx"));
        let service = Service::new_map(map, None);
        let result = service.action(&mut host, "start").unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Service started...");
        assert_eq!(result.stderr, "");

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_command_mapped() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("command::exec", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/usr/local/bin/nginx -s", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Service started...", zmq::SNDMORE).unwrap();
            agent_sock.send_str("", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let mut map = HashMap::new();
        map.insert("start", "-s");
        let service = Service::new_service(ServiceRunnable::Command("/usr/local/bin/nginx"), Some(map));
        let result = service.action(&mut host, "start").unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Service started...");
        assert_eq!(result.stderr, "");

        agent_mock.join().unwrap();
    }
}
