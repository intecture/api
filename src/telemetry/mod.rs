// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Data structures containing information about your managed host.
//!
//! The Telemetry struct stores metadata about a host, such as its
//! network interfaces, disk mounts, CPU stats and hostname.
//!
//! # Examples
//!
//! Initialise a new Host using your managed host's IP address and
//! port number:
//!
//! ```no_run
//! # use inapi::Host;
//! let host = Host::new("127.0.0.1", 7101);
//! ```
//!
//! Now run your command and get the result:
//!
//! ```no_run
//! # use inapi::{Host, Telemetry, TelemetryInit};
//! # let mut host = Host::new("127.0.0.1", 7101).unwrap();
//! let telemetry = Telemetry::init(&mut host);
//! ```

pub mod ffi;

use error::Result;
use host::Host;
pub use inprimitives::telemetry::{Cpu, FsMount, Netif, NetifStatus, NetifIPv4, NetifIPv6, Os, Telemetry};
use rustc_serialize::json;

/// The TelemetryInit trait is used to initialise new Telemetry
/// structs.
pub trait TelemetryInit {
    /// Initialise a new Telemetry struct for the given Host.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::Host;
    /// use inapi::{Telemetry, TelemetryInit};
    /// # let mut host = Host::new("127.0.0.1", 7101).unwrap();
    /// let telemetry = Telemetry::init(&mut host);
    fn init(host: &mut Host) -> Result<Telemetry>;
}

impl TelemetryInit for Telemetry {
    fn init(host: &mut Host) -> Result<Telemetry> {
        try!(host.send("telemetry", 0));

        try!(host.recv_header());

        let telemetry = try!(host.expect_recv("telemetry", 1));

        Ok(try!(json::decode(&telemetry)))
    }
}

#[cfg(test)]
mod tests {
    use host::Host;
    use rustc_serialize::json;
    use super::*;
    use std::thread;
    use zmq;

    #[test]
    fn test_telemetry_init() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_init").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("telemetry", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            let telemetry = Telemetry::new(
                Cpu::new(
                    "moo".to_string(),
                    "Moo Cow Super Fun Happy CPU".to_string(),
                    100,
                ),
                vec![FsMount::new(
                    "/dev/disk0".to_string(),
                    "/".to_string(),
                    10000,
                    5000,
                    5000,
                    0.5,
                    20,
                    0,
                    1.0,
                )],
                "localhost".to_string(),
                2048,
                vec![Netif::new(
                    "em0".to_string(),
                    Some("01:23:45:67:89:ab".to_string()),
                    Some(NetifIPv4::new(
                        "127.0.0.1".to_string(),
                        "255.255.255.255".to_string(),
                    )),
                    Some(NetifIPv6::new(
                        "::1".to_string(),
                        8,
                        Some("0x4".to_string()),
                    )),
                    Some(NetifStatus::Active),
                )],
                Os::new(
                    "doctor string".to_string(),
                    "moo".to_string(),
                    "cow".to_string(),
                    "1.0".to_string(),
                ),
            );

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str(&json::encode(&telemetry).unwrap(), 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_init").unwrap();

        let mut host = Host::test_new(sock);

        let telemetry = Telemetry::init(&mut host).unwrap();

        assert_eq!(telemetry.memory, 2048);
        assert_eq!(telemetry.os.arch, "doctor string".to_string());

        agent_mock.join().unwrap();
    }
}