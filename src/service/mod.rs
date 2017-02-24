// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Service primitive.

pub mod ffi;

use command::{CommandResult, CommandTarget};
use error::{Error, Result};
use host::Host;
use std::collections::HashMap;
use std::convert::Into;
use target::Target;

/// Runnables are the executable items that a Service calls actions
/// on.
pub enum ServiceRunnable<'a> {
    /// A script that is executed by the shell
    Command(&'a str),
    /// A daemon managed by the default system service manager
    Service(&'a str),
}

enum ServiceRunnableOwned {
    Command(String),
    Service(String),
}

impl<'a> From<ServiceRunnable<'a>> for ServiceRunnableOwned {
    fn from(runnable: ServiceRunnable<'a>) -> ServiceRunnableOwned {
        match runnable {
            ServiceRunnable::Command(c) => ServiceRunnableOwned::Command(c.into()),
            ServiceRunnable::Service(s) => ServiceRunnableOwned::Service(s.into()),
        }
    }
}

/// Primitive for controlling service daemons.
///
///# Basic Usage
///
/// Initialise a new `Host`:
///
/// ```no_run
/// # use inapi::Host;
#[cfg_attr(feature = "local-run", doc = "let path: Option<String> = None;")]
#[cfg_attr(feature = "local-run", doc = "let mut host = Host::local(path).unwrap();")]
#[cfg_attr(feature = "remote-run", doc = "let mut host = Host::connect(\"hosts/myhost.json\").unwrap();")]
/// ```
///
/// Create a new `Service` to manage a daemon (for example, Nginx):
///
/// ```no_run
/// # use inapi::{Host, Service, ServiceRunnable};
#[cfg_attr(feature = "local-run", doc = "# let path: Option<String> = None;")]
#[cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(path).unwrap();")]
#[cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"hosts/myhost.json\").unwrap();")]
///let service = Service::new_service(ServiceRunnable::Service("nginx"), None);
/// ```
///
/// If your daemon uses a script instead of an init system, just
/// change the `ServiceRunnable` type to `Command`:
///
/// ```no_run
/// # use inapi::{Host, Service, ServiceRunnable};
#[cfg_attr(feature = "local-run", doc = "# let path: Option<String> = None;")]
#[cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(path).unwrap();")]
#[cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"hosts/myhost.json\").unwrap();")]
///let service = Service::new_service(ServiceRunnable::Command("/usr/bin/apachectl"), None);
/// ```
///
/// Now you can run an action against the `Service`. This will
/// optionally return a `CommandResult` if the action was run, or
/// `None` if the service was already in the desired state.
///
/// ```no_run
/// # use inapi::{Host, Service, ServiceRunnable};
#[cfg_attr(feature = "local-run", doc = "# let path: Option<String> = None;")]
#[cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(path).unwrap();")]
#[cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"hosts/myhost.json\").unwrap();")]
/// # let service = Service::new_service(ServiceRunnable::Service(""), None);
///let result = service.action(&mut host, "start").unwrap();
///if let Some(r) = result {
///    assert_eq!(r.exit_code, 0);
///}
/// ```
///
///# Runnables
///
/// Runnables are the executable items that a `Service` runs.
///
/// `ServiceRunnable::Service` represents a daemon managed by the
/// init system.
///
/// For example, on a Linux system using Init,
/// `ServiceRunnable::Service("nginx")` is executed as:
///>`service nginx <action>`
///
/// If your daemon does not have an init script, use
/// `ServiceRunnable::Command` to reference an executable directly.
///
/// For example, `ServiceRunnable::Command("/usr/bin/nginx")` will
/// simply run the executable path `/usr/bin/nginx`.
///
///# Mapping Actions to Runnables
///
/// The other way of initialising a `Service` is with the `new_map()`
/// function. This allows you to map individual actions to separate
/// `ServiceRunnable`s.
///
/// ```no_run
/// # use inapi::{Host, Service, ServiceRunnable};
/// # use std::collections::HashMap;
#[cfg_attr(feature = "local-run", doc = "# let path: Option<String> = None;")]
#[cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(path).unwrap();")]
#[cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"hosts/myhost.json\").unwrap();")]
/// let mut map = HashMap::new();
/// map.insert("start", ServiceRunnable::Command("/usr/local/bin/svc_start"));
/// map.insert("stop", ServiceRunnable::Command("/usr/local/bin/svc_stop"));
/// map.insert("restart", ServiceRunnable::Command("/usr/local/bin/svc_stop && /usr/local/bin/svc_start"));
/// let service = Service::new_map(map, None);
/// ```
///
/// You can also set a default `ServiceRunnable`. For example, you
/// could map the "status" action to a `ServiceRunnable::Command` while
/// defaulting to a `ServiceRunnable::Service` for all other actions:
///
/// ```no_run
/// # use inapi::{Host, Service, ServiceRunnable};
/// # use std::collections::HashMap;
#[cfg_attr(feature = "local-run", doc = "# let path: Option<String> = None;")]
#[cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(path).unwrap();")]
#[cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"hosts/myhost.json\").unwrap();")]
///let mut map = HashMap::new();
///map.insert("_", ServiceRunnable::Service("nginx")); // <-- "_" is the default key
///map.insert("status", ServiceRunnable::Command("/usr/bin/zabbix_get -k 'proc.num[nginx,nginx]'"));
///let service = Service::new_map(map, None);
/// ```
///
/// Note that when mapping actions to `ServiceRunnable::Command`s, the
/// action name is not appended to the command, unless it is the default
/// `ServiceRunnable`:
///
/// ```no_run
/// # use inapi::{Host, Service, ServiceRunnable};
/// # use std::collections::HashMap;
#[cfg_attr(feature = "local-run", doc = "# let path: Option<String> = None;")]
#[cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(path).unwrap();")]
#[cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"hosts/myhost.json\").unwrap();")]
///let mut map = HashMap::new();
///map.insert("start", ServiceRunnable::Command("/usr/bin/start_svc"));
///map.insert("kill", ServiceRunnable::Command("killall my_svc"));
///map.insert("_", ServiceRunnable::Command("/usr/bin/svc_ctl"));
///let service = Service::new_map(map, None);
///service.action(&mut host, "start").unwrap(); // <-- Calls "/usr/bin/start_svc"
///service.action(&mut host, "status").unwrap(); // <-- Calls "/usr/bin/svc_ctl status"
/// ```
///
///# Mapping Actions to Other Actions
///
/// In an effort to standardise actions across platforms and daemons,
/// it is sometimes advantageous to map one action to another. For
/// example, you can use action maps to abstract complex flags, which
/// makes your code more readable:
///
/// ```no_run
/// # use inapi::{Host, Service, ServiceRunnable};
/// # use std::collections::HashMap;
#[cfg_attr(feature = "local-run", doc = "# let path: Option<String> = None;")]
#[cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(path).unwrap();")]
#[cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"hosts/myhost.json\").unwrap();")]
///let mut map = HashMap::new();
///map.insert("start", "-c /etc/my_svc.conf");
///map.insert("stop", "-t");
///let service = Service::new_service(ServiceRunnable::Command("/usr/local/bin/my_svc"), Some(map));
///service.action(&mut host, "start").unwrap(); // <-- Calls "/usr/local/bin/my_svc -c /etc/my_svc.conf"
/// ```
///
/// You can also use action maps to assist with cross-platform compatibility:
///
/// ```no_run
/// # use inapi::{Host, Service, ServiceRunnable};
/// # use std::collections::HashMap;
#[cfg_attr(feature = "local-run", doc = "# let path: Option<String> = None;")]
#[cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(path).unwrap();")]
#[cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"hosts/myhost.json\").unwrap();")]
///let mut map = HashMap::new();
///if needstr!(host.data_owned() => "/_telemetry/os/platform").unwrap() == "centos" {
///    map.insert("reload", "restart");
///}
///let service = Service::new_service(ServiceRunnable::Command("/usr/bin/my_svc"), Some(map));
///service.action(&mut host, "reload").unwrap(); // <-- Calls "/usr/bin/my_svc restart" on CentOS
/// ```
pub struct Service {
    /// Actions map for Runnables
    actions: HashMap<String, ServiceRunnableOwned>,
    /// Action aliases map
    mapped_actions: Option<HashMap<String, String>>,
}

impl Service {
    /// Create a new `Service` with a single `ServiceRunnable`.
    ///
    /// ```no_run
    /// # use inapi::{Service, ServiceRunnable};
    /// let service = Service::new_service(ServiceRunnable::Service("service_name"), None);
    /// ```
    pub fn new_service<'a>(runnable: ServiceRunnable<'a>, mapped_actions: Option<HashMap<&'a str, &'a str>>) -> Service {
        let mut actions = HashMap::new();
        actions.insert("_", runnable);
        Self::new_map(actions, mapped_actions)
    }

    /// Create a new `Service` with multiple `ServiceRunnable`s.
    ///
    /// ```no_run
    /// # use inapi::{Service, ServiceRunnable};
    /// # use std::collections::HashMap;
    /// let mut map = HashMap::new();
    /// map.insert("start", ServiceRunnable::Command("/usr/local/bin/nginx"));
    /// map.insert("stop", ServiceRunnable::Command("killall \"nginx: master process nginx\""));
    /// let service = Service::new_map(map, None);
    /// ```
    pub fn new_map<'a>(actions: HashMap<&'a str, ServiceRunnable<'a>>, mapped_actions: Option<HashMap<&'a str, &'a str>>) -> Service {
        let mut actions_owned = HashMap::new();
        for (k, v) in actions {
            actions_owned.insert(k.to_owned(), v.into());
        }

        let mapped_actions_owned = match mapped_actions {
            Some(mapped) => {
                let mut owned = HashMap::new();
                for (k, v) in mapped {
                    owned.insert(k.to_owned(), v.to_owned());
                }
                Some(owned)
            },
            None => None,
        };

        Service {
            actions: actions_owned,
            mapped_actions: mapped_actions_owned,
        }
    }

    /// Run a service action, e.g. "start" or "stop".
    ///
    /// If the function returns `Some`, the action was required to
    /// run in order to get the host into the required state. If the
    /// function returns `None`, the host is already in the required
    /// state.
    ///
    /// ```no_run
    /// # use inapi::{Host, Service, ServiceRunnable};
    #[cfg_attr(feature = "local-run", doc = "# let path: Option<String> = None;")]
    #[cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(path).unwrap();")]
    #[cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"hosts/myhost.json\").unwrap();")]
    /// let service = Service::new_service(ServiceRunnable::Command("/usr/bin/nginx"), None);
    /// service.action(&mut host, "start").unwrap();
    /// ```
    pub fn action(&self, host: &mut Host, action: &str) -> Result<Option<CommandResult>> {
        let mut action = action;

        // Exchange this action with a mapped action if possible
        if let Some(ref mapped) = self.mapped_actions {
            if mapped.contains_key(action) {
                action = mapped.get(action).unwrap();
            }
        }

        if self.actions.contains_key(action) {
            self.run(host, action, self.actions.get(action).unwrap(), false)
        } else if self.actions.contains_key("_") {
            self.run(host, action, self.actions.get("_").unwrap(), true)
        } else {
            Err(Error::Generic(format!("Unrecognised action {}", action)))
        }
    }

    fn run(&self, host: &mut Host, action: &str, runnable: &ServiceRunnableOwned, default: bool) -> Result<Option<CommandResult>> {
        match *runnable {
            ServiceRunnableOwned::Service(ref name) => Target::service_action(host, name, action),
            ServiceRunnableOwned::Command(ref cmd) => if default {
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

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("service::action", req.popstr().unwrap().unwrap());
            assert_eq!("nginx", req.popstr().unwrap().unwrap());
            assert_eq!("start", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("Service started...").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("service::action", req.popstr().unwrap().unwrap());
            assert_eq!("nginx", req.popstr().unwrap().unwrap());
            assert_eq!("start", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

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

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("service::action", msg.popstr().unwrap().unwrap());
            assert_eq!("nginx", msg.popstr().unwrap().unwrap());
            assert_eq!("start", msg.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("Service started...").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

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

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("service::action", msg.popstr().unwrap().unwrap());
            assert_eq!("nginx", msg.popstr().unwrap().unwrap());
            assert_eq!("load", msg.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("Service started...").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

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
        let mut host = Host::test_new(None, None, None, None);

        let mut map = HashMap::new();
        map.insert("start", ServiceRunnable::Service("nginx"));
        let service = Service::new_map(map, None);
        assert!(service.action(&mut host, "nonexistent").is_err());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_action_command() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("command::exec", msg.popstr().unwrap().unwrap());
            assert_eq!("/usr/local/bin/nginx", msg.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("Service started...").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

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

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("command::exec", msg.popstr().unwrap().unwrap());
            assert_eq!("/usr/local/bin/nginx -s", msg.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.addstr("Service started...").unwrap();
            rep.addstr("").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

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
