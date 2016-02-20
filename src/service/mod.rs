// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The primitive for controlling services on a managed host.
//!
//! # Examples
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
//! Now you can run a service action on your managed host.
//!
//! ```no_run
//! # use inapi::{Host, Service};
//! # let mut host = Host::new();
//! let service = Service::new("nginx");
//! let result = service.action(&mut host, "start").unwrap();
//! assert_eq!(result.exit_code, 0);
//! ```

pub mod ffi;

use {CommandResult, Host, Result};
use target::Target;

/// Container for managing a service.
pub struct Service {
    /// Name of the service
    name: String,
}

impl Service {
    /// Create a new Service struct.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::Service;
    /// let service = Service::new("myservice");
    /// ```
    pub fn new(name: &str) -> Service {
        Service {
            name: name.to_string(),
        }
    }

    /// Run an action on the service.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::{Host, Service};
    /// let mut host = Host::new();
    /// let service = Service::new("nginx");
    /// service.action(&mut host, "restart");
    /// ```
    pub fn action(&self, host: &mut Host, action: &str) -> Result<CommandResult> {
        Target::service_action(host, &self.name, action)
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
    fn test_action() {
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
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_action").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let service = Service::new("nginx");
        let result = service.action(&mut host, "start").unwrap();

        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "Service started...");
        assert_eq!(result.stderr, "");

        agent_mock.join().unwrap();
    }
}
