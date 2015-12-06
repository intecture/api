// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The host wrapper for communicating with a managed host.
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
//! Now you can pass it into other structures to enable them to
//! communicate with your managed host:
//!
//! ```no_run
//! # use inapi::Host;
//! # use inapi::Command;
//! # let mut host = Host::new("127.0.0.1", 7101).unwrap();
//! let cmd = Command::new("whoami");
//! let result = cmd.exec(&mut host).unwrap();
//! ```

pub mod ffi;

use error::{Error, MissingFrame, Result};
use std::sync::Mutex;
use zmq;

lazy_static! {
    static ref ZMQCTX: Mutex<zmq::Context> = Mutex::new(zmq::Context::new());
}

/// Representation of a managed host.
pub struct Host {
    /// ZMQ socket connection to host
    zmq_sock: zmq::Socket,
}

impl Host {
    /// Create a new Host to represent your managed host.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::Host;
    /// let host = Host::new("127.0.0.1", 7101);
    /// ```
    pub fn new(ip: &str, port: u32) -> Result<Host> {
        let mut sock = ZMQCTX.lock().unwrap().socket(zmq::REQ).unwrap();
        try!(sock.set_linger(5000));
        try!(sock.connect(&format!("tcp://{}:{}", ip, port)));

        Ok(Host {
            zmq_sock: sock,
        })
    }

    pub fn close(&mut self) -> Result<()> {
        try!(self.zmq_sock.close());
        Ok(())
    }

    pub fn send(&mut self, msg: &str, flags: i32) -> Result<()> {
        try!(self.zmq_sock.send_str(msg, flags));
        Ok(())
    }

    pub fn recv_header(&mut self) -> Result<()> {
        match try!(self.zmq_sock.recv_string(0)).unwrap().as_ref() {
            "Err" => Err(Error::Agent(try!(self.expect_recv("err_msg", 1)))),
            "Ok" => Ok(()),
            _ => unreachable!(),
        }
    }

    pub fn expect_recv(&mut self, name: &str, order: u8) -> Result<String> {
        if self.zmq_sock.get_rcvmore().unwrap() == false {
            return Err(Error::Frame(MissingFrame::new(name, order)));
        }

        Ok(try!(self.zmq_sock.recv_string(0)).unwrap())
    }

    pub fn expect_recvmsg(&mut self, name: &str, order: u8) -> Result<zmq::Message> {
        if self.zmq_sock.get_rcvmore().unwrap() == false {
            return Err(Error::Frame(MissingFrame::new(name, order)));
        }

        Ok(try!(self.zmq_sock.recv_msg(0)))
    }

    #[cfg(test)]
    pub fn test_new(sock: zmq::Socket) -> Host {
        let host = Host {
            zmq_sock: sock,
        };

        host
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zmq;

    #[test]
    fn test_host_new() {
        let host = Host::new("localhost", 7101);
        assert!(host.is_ok());
    }

    #[test]
    fn test_host_send() {
        let mut ctx = zmq::Context::new();

        let mut server = ctx.socket(zmq::REP).unwrap();
        server.bind("inproc://test_host_send").unwrap();

        let mut client = ctx.socket(zmq::REQ).unwrap();
        client.connect("inproc://test_host_send").unwrap();

        let mut host = Host::test_new(client);
        host.send("moo", zmq::SNDMORE).unwrap();
        host.send("cow", 0).unwrap();

        assert_eq!(server.recv_string(0).unwrap().unwrap(), "moo");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "cow");
    }

    #[test]
    fn test_host_recv_header_ok() {
        let mut ctx = zmq::Context::new();

        let mut req = ctx.socket(zmq::REQ).unwrap();
        req.connect("inproc://test_host_recv_header_ok").unwrap();
        req.send_str("Ok", 0).unwrap();

        let mut rep = ctx.socket(zmq::REP).unwrap();
        rep.bind("inproc://test_host_recv_header_ok").unwrap();

        let mut host = Host::test_new(rep);
        assert!(host.recv_header().is_ok());
    }

    #[test]
    fn test_host_recv_header_err() {
        let mut ctx = zmq::Context::new();

        let mut req = ctx.socket(zmq::REQ).unwrap();
        req.connect("inproc://test_host_recv_header_err").unwrap();
        req.send_str("Err", 0).unwrap();

        let mut rep = ctx.socket(zmq::REP).unwrap();
        rep.bind("inproc://test_host_recv_header_err").unwrap();

        let mut host = Host::test_new(rep);
        assert!(host.recv_header().is_err());
    }

    #[test]
    fn test_host_expect_recv_ok() {
        let mut ctx = zmq::Context::new();

        let mut req = ctx.socket(zmq::REQ).unwrap();
        req.connect("inproc://test_host_expect_recv_ok").unwrap();
        req.send_str("Ok", zmq::SNDMORE).unwrap();
        req.send_str("Frame 0", zmq::SNDMORE).unwrap();
        req.send_str("Frame 1", 0).unwrap();

        let mut rep = ctx.socket(zmq::REP).unwrap();
        rep.bind("inproc://test_host_expect_recv_ok").unwrap();
        rep.recv_string(0).unwrap().unwrap();

        let mut host = Host::test_new(rep);
        assert_eq!(host.expect_recv("Frame 0", 0).unwrap(), "Frame 0");
        assert_eq!(host.expect_recv("Frame 1", 1).unwrap(), "Frame 1");
    }

    #[test]
    fn test_host_expect_recv_err() {
        let mut ctx = zmq::Context::new();

        let mut req = ctx.socket(zmq::REQ).unwrap();
        req.connect("inproc://test_host_expect_recv_ok").unwrap();
        req.send_str("Ok", 0).unwrap();

        let mut rep = ctx.socket(zmq::REP).unwrap();
        rep.bind("inproc://test_host_expect_recv_ok").unwrap();
        rep.recv_string(0).unwrap().unwrap();

        let mut host = Host::test_new(rep);
        assert!(host.expect_recv("Frame 0", 0).is_err());
    }

    #[test]
    fn test_host_expect_recvmsg_ok() {
        let mut ctx = zmq::Context::new();

        let mut req = ctx.socket(zmq::REQ).unwrap();
        req.connect("inproc://test_host_expect_recvmsg_ok").unwrap();
        req.send_str("Ok", zmq::SNDMORE).unwrap();
        req.send_str("Frame 0", zmq::SNDMORE).unwrap();
        req.send_str("Frame 1", 0).unwrap();

        let mut rep = ctx.socket(zmq::REP).unwrap();
        rep.bind("inproc://test_host_expect_recvmsg_ok").unwrap();
        rep.recv_string(0).unwrap().unwrap();

        let mut host = Host::test_new(rep);
        assert_eq!(host.expect_recvmsg("Frame 0", 0).unwrap().as_str().unwrap(), "Frame 0");
        assert_eq!(host.expect_recvmsg("Frame 1", 1).unwrap().as_str().unwrap(), "Frame 1");
    }

    #[test]
    fn test_host_expect_recvmsg_err() {
        let mut ctx = zmq::Context::new();

        let mut req = ctx.socket(zmq::REQ).unwrap();
        req.connect("inproc://test_host_expect_recvmsg_ok").unwrap();
        req.send_str("Ok", 0).unwrap();

        let mut rep = ctx.socket(zmq::REP).unwrap();
        rep.bind("inproc://test_host_expect_recvmsg_ok").unwrap();
        rep.recv_string(0).unwrap().unwrap();

        let mut host = Host::test_new(rep);
        assert!(host.expect_recvmsg("Frame 0", 0).is_err());
    }
}