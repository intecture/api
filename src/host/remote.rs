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
            hostname: None,
            api_sock: None,
            upload_sock: None,
            download_port: None,
        }
    }

    #[cfg(test)]
    pub fn test_new(sock: zmq::Socket) -> Host {
        let host = Host {
            hostname: None,
            api_sock: Some(sock),
            upload_sock: None,
            download_port: None,
        };

        host
    }

    pub fn connect(&mut self, hostname: &str, api_port: u32, upload_port: u32, download_port: u32) -> Result<()> {
        self.hostname = Some(hostname.to_string());

        self.api_sock = Some(ZMQCTX.lock().unwrap().socket(zmq::REQ).unwrap());
        try!(self.api_sock.as_mut().unwrap().set_linger(5000));
        try!(self.api_sock.as_mut().unwrap().connect(&format!("tcp://{}:{}", hostname, api_port)));

        self.upload_sock = Some(ZMQCTX.lock().unwrap().socket(zmq::PUB).unwrap());
        try!(self.upload_sock.as_mut().unwrap().connect(&format!("tcp://{}:{}", hostname, upload_port)));

        self.download_port = Some(download_port);

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if self.api_sock.is_none() {
            return Err(Error::Generic("Host is not connected".to_string()));
        }

        try!(self.api_sock.as_mut().unwrap().close());
        self.api_sock = None;

        try!(self.upload_sock.as_mut().unwrap().close());
        self.upload_sock = None;

        Ok(())
    }

    #[doc(hidden)]
    pub fn send(&mut self, msg: &str, flags: i32) -> Result<()> {
        if self.api_sock.is_none() {
            return Err(Error::Generic("Host is not connected".to_string()));
        }

        try!(self.api_sock.as_mut().unwrap().send_str(msg, flags));
        Ok(())
    }

    #[doc(hidden)]
    pub fn send_file(&mut self, endpoint: &str, path: &str, hash: u64, size: u64, total_chunks: u64) -> Result<zmq::Socket> {
        let mut download_sock = ZMQCTX.lock().unwrap().socket(zmq::SUB).unwrap();
        try!(download_sock.connect(&format!("tcp://{}:{}", self.hostname.as_mut().unwrap(), self.download_port.unwrap())));
        try!(download_sock.set_subscribe(path.as_bytes()));

        try!(self.api_sock.as_mut().unwrap().send_str(endpoint, zmq::SNDMORE));
        try!(self.api_sock.as_mut().unwrap().send_str(path, zmq::SNDMORE));
        try!(self.api_sock.as_mut().unwrap().send_str(&hash.to_string(), zmq::SNDMORE));
        try!(self.api_sock.as_mut().unwrap().send_str(&size.to_string(), zmq::SNDMORE));
        try!(self.api_sock.as_mut().unwrap().send_str(&total_chunks.to_string(), 0));

        Ok(download_sock)
    }

    #[doc(hidden)]
    pub fn send_chunk(&mut self, path: &str, index: u64, chunk: &[u8]) -> Result<()> {
        try!(self.upload_sock.as_mut().unwrap().send_str(path, zmq::SNDMORE));
        try!(self.upload_sock.as_mut().unwrap().send_str(&index.to_string(), zmq::SNDMORE));
        try!(self.upload_sock.as_mut().unwrap().send(chunk, 0));
        Ok(())
    }

    #[doc(hidden)]
    pub fn recv_header(&mut self) -> Result<()> {
        if self.api_sock.is_none() {
            return Err(Error::Generic("Host is not connected".to_string()));
        }

        match try!(self.api_sock.as_mut().unwrap().recv_string(0)).unwrap().as_ref() {
            "Ok" => Ok(()),
            "Err" => Err(Error::Agent(try!(self.expect_recv("err_msg", 1)))),
            _ => unreachable!(),
        }
    }

    #[doc(hidden)]
    pub fn recv_chunk(&mut self, download_sock: &mut zmq::Socket) -> Result<u64> {
        try!(download_sock.recv_string(0)).unwrap();

        if download_sock.get_rcvmore().unwrap() == false {
            return Err(Error::Frame(MissingFrame::new("chunk", 2)));
        }

        Ok(try!(download_sock.recv_string(0)).unwrap().parse::<u64>().unwrap())
    }

    #[doc(hidden)]
    pub fn expect_recv(&mut self, name: &str, order: u8) -> Result<String> {
        if self.api_sock.is_none() {
            return Err(Error::Generic("Host is not connected".to_string()));
        }

        if self.api_sock.as_mut().unwrap().get_rcvmore().unwrap() == false {
            return Err(Error::Frame(MissingFrame::new(name, order)));
        }

        Ok(try!(self.api_sock.as_mut().unwrap().recv_string(0)).unwrap())
    }

    #[doc(hidden)]
    pub fn expect_recvmsg(&mut self, name: &str, order: u8) -> Result<zmq::Message> {
        if self.api_sock.is_none() {
            return Err(Error::Generic("Host is not connected".to_string()));
        }

        if self.api_sock.as_mut().unwrap().get_rcvmore().unwrap() == false {
            return Err(Error::Frame(MissingFrame::new(name, order)));
        }

        Ok(try!(self.api_sock.as_mut().unwrap().recv_msg(0)))
    }
}

#[cfg(test)]
mod tests {
    use {Host, zmq};

    #[test]
    fn test_host_connect() {
        let mut host = Host::new();
        assert!(host.connect("127.0.0.1", 7101, 7102, 7103).is_ok());
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
