// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The host wrapper for communicating with a remote host.

use error::{Error, MissingFrame};
use Result;
use std::sync::Mutex;
use super::*;
use zmq;

lazy_static! {
    static ref ZMQCTX: Mutex<zmq::Context> = Mutex::new(zmq::Context::new());
}

impl Host {
    /// Create a new Host to represent your managed host.
    pub fn new() -> Host {
        Host {
            zmq_sock: None,
        }
    }

    #[cfg(test)]
    pub fn test_new(sock: zmq::Socket) -> Host {
        let host = Host {
            zmq_sock: Some(sock),
        };

        host
    }

    pub fn connect(&mut self, ip: &str, port: u32) -> Result<()> {
        let mut sock = ZMQCTX.lock().unwrap().socket(zmq::REQ).unwrap();
        try!(sock.set_linger(5000));
        try!(sock.connect(&format!("tcp://{}:{}", ip, port)));

        self.zmq_sock = Some(sock);
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if self.zmq_sock.is_none() {
            return Err(Error::Generic("Host is not connected".to_string()));
        }

        try!(self.zmq_sock.as_mut().unwrap().close());
        self.zmq_sock = None;
        Ok(())
    }

    #[doc(hidden)]
    pub fn send(&mut self, msg: &str, flags: i32) -> Result<()> {
        if self.zmq_sock.is_none() {
            return Err(Error::Generic("Host is not connected".to_string()));
        }

        try!(self.zmq_sock.as_mut().unwrap().send_str(msg, flags));
        Ok(())
    }

    #[doc(hidden)]
    pub fn recv_header(&mut self) -> Result<()> {
        if self.zmq_sock.is_none() {
            return Err(Error::Generic("Host is not connected".to_string()));
        }

        match try!(self.zmq_sock.as_mut().unwrap().recv_string(0)).unwrap().as_ref() {
            "Err" => Err(Error::Agent(try!(self.expect_recv("err_msg", 1)))),
            "Ok" => Ok(()),
            _ => unreachable!(),
        }
    }

    #[doc(hidden)]
    pub fn expect_recv(&mut self, name: &str, order: u8) -> Result<String> {
        if self.zmq_sock.is_none() {
            return Err(Error::Generic("Host is not connected".to_string()));
        }

        if self.zmq_sock.as_mut().unwrap().get_rcvmore().unwrap() == false {
            return Err(Error::Frame(MissingFrame::new(name, order)));
        }

        Ok(try!(self.zmq_sock.as_mut().unwrap().recv_string(0)).unwrap())
    }

    #[doc(hidden)]
    pub fn expect_recvmsg(&mut self, name: &str, order: u8) -> Result<zmq::Message> {
        if self.zmq_sock.is_none() {
            return Err(Error::Generic("Host is not connected".to_string()));
        }

        if self.zmq_sock.as_mut().unwrap().get_rcvmore().unwrap() == false {
            return Err(Error::Frame(MissingFrame::new(name, order)));
        }

        Ok(try!(self.zmq_sock.as_mut().unwrap().recv_msg(0)))
    }
}

#[cfg(test)]
mod tests {
    use {Host, zmq};

    #[test]
    fn test_host_connect() {
        let mut host = Host::new();
        assert!(host.connect("127.0.0.1", 7101).is_ok());
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
