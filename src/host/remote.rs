// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The host wrapper for communicating with a remote host.

use czmq::ZCert;
use error::{Error, MissingFrame};
use file::FileOpts;
use Result;
use runtime_helpers::RUNTIME_ARGS;
use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;
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
    pub fn test_new(hostname: Option<String>, api_sock: Option<zmq::Socket>, upload_sock: Option<zmq::Socket>, download_port: Option<u32>) -> Host {
        let host = Host {
            hostname: hostname,
            api_sock: api_sock,
            upload_sock: upload_sock,
            download_port: download_port,
        };

        host
    }

    pub fn connect(&mut self, hostname: &str, api_port: u32, upload_port: u32, download_port: u32) -> Result<()> {
        self.hostname = Some(hostname.to_string());

        let server_cert = try!(ZCert::load(&format!("{}/{}.crt", RUNTIME_ARGS.server_cert_path, hostname)));

        self.api_sock = Some(ZMQCTX.lock().unwrap().socket(zmq::REQ).unwrap());
        RUNTIME_ARGS.user_cert.apply(self.api_sock.as_mut().unwrap());
        try!(self.api_sock.as_mut().unwrap().set_curve_serverkey(server_cert.public_txt()));
        try!(self.api_sock.as_mut().unwrap().set_linger(5000));
        try!(self.api_sock.as_mut().unwrap().connect(&format!("tcp://{}:{}", hostname, api_port)));

        self.upload_sock = Some(ZMQCTX.lock().unwrap().socket(zmq::PUB).unwrap());
        RUNTIME_ARGS.user_cert.apply(self.api_sock.as_mut().unwrap());
        try!(self.upload_sock.as_mut().unwrap().set_curve_serverkey(server_cert.public_txt()));
        try!(self.upload_sock.as_mut().unwrap().connect(&format!("tcp://{}:{}", hostname, upload_port)));

        self.download_port = Some(download_port);

        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if self.api_sock.is_some() {
            try!(self.api_sock.as_mut().unwrap().close());
            self.api_sock = None;
        }

        if self.upload_sock.is_some() {
            try!(self.upload_sock.as_mut().unwrap().close());
            self.upload_sock = None;
        }

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
    pub fn send_file(&mut self, endpoint: &str, path: &str, hash: u64, size: u64, total_chunks: u64, options: Option<&[FileOpts]>) -> Result<zmq::Socket> {
        let mut download_sock = ZMQCTX.lock().unwrap().socket(zmq::SUB).unwrap();
        RUNTIME_ARGS.user_cert.apply(&mut download_sock);
        let server_cert = try!(ZCert::load(&format!("{}/{}.crt", RUNTIME_ARGS.server_cert_path, self.hostname.as_mut().unwrap())));
        try!(download_sock.set_curve_serverkey(server_cert.public_txt()));
        try!(download_sock.connect(&format!("tcp://{}:{}", self.hostname.as_mut().unwrap(), self.download_port.unwrap())));
        try!(download_sock.set_subscribe(path.as_bytes()));

        // Try to mitigate late joiner syndrome
        sleep(Duration::from_millis(100));

        try!(self.send(endpoint, zmq::SNDMORE));
        try!(self.send(path, zmq::SNDMORE));
        try!(self.send(&hash.to_string(), zmq::SNDMORE));
        try!(self.send(&size.to_string(), zmq::SNDMORE));
        try!(self.send(&total_chunks.to_string(), if options.is_some() { zmq::SNDMORE } else { 0 }));

        if let Some(opts) = options {
            let mut cnt = 1;

            for opt in opts {
                let send_more = if cnt < opts.len() { zmq::SNDMORE } else { 0 };

                match opt {
                    &FileOpts::BackupExistingFile(ref suffix) => {
                        try!(self.send("OPT_BackupExistingFile", zmq::SNDMORE));
                        try!(self.send(suffix, send_more));
                    },
                }

                cnt += 1;
            }
        }

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
    use file::FileOpts;

    #[test]
    fn test_host_connect() {
        let mut host = Host::new();
        assert!(host.connect("localhost", 7101, 7102, 7103).is_ok());
    }

    #[test]
    fn test_host_send() {
        let mut ctx = zmq::Context::new();

        let mut server = ctx.socket(zmq::REP).unwrap();
        server.bind("inproc://test_host_send").unwrap();

        let mut client = ctx.socket(zmq::REQ).unwrap();
        client.connect("inproc://test_host_send").unwrap();

        let mut host = Host::test_new(None, Some(client), None, None);
        host.send("moo", zmq::SNDMORE).unwrap();
        host.send("cow", 0).unwrap();

        assert_eq!(server.recv_string(0).unwrap().unwrap(), "moo");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "cow");
    }

    #[test]
    fn test_send_file_noopts() {
        let mut ctx = zmq::Context::new();

        let mut server = ctx.socket(zmq::REP).unwrap();
        server.bind("inproc://test_host_send_file_noopts").unwrap();

        let mut client = ctx.socket(zmq::REQ).unwrap();
        client.connect("inproc://test_host_send_file_noopts").unwrap();

        let mut host = Host::test_new(Some("localhost".to_string()), Some(client), None, Some(7103));
        host.send_file("file::upload", "/tmp/moo", 123, 0, 0, None).unwrap();

        assert_eq!(server.recv_string(0).unwrap().unwrap(), "file::upload");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "/tmp/moo");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "123");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "0");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "0");
        assert!(!server.get_rcvmore().unwrap());
    }

    #[test]
    fn test_send_file_opts() {
        let mut ctx = zmq::Context::new();

        let mut server = ctx.socket(zmq::REP).unwrap();
        server.bind("inproc://test_host_send_file_opts").unwrap();

        let mut client = ctx.socket(zmq::REQ).unwrap();
        client.connect("inproc://test_host_send_file_opts").unwrap();

        let mut host = Host::test_new(Some("localhost".to_string()), Some(client), None, Some(7103));

        let opts = vec![FileOpts::BackupExistingFile("_moo".to_string())];

        host.send_file("file::upload", "/tmp/moo", 123, 0, 0, Some(&opts)).unwrap();

        assert_eq!(server.recv_string(0).unwrap().unwrap(), "file::upload");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "/tmp/moo");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "123");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "0");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "0");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "OPT_BackupExistingFile");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "_moo");
    }

    #[test]
    fn test_host_send_chunk() {
        let mut ctx = zmq::Context::new();

        let mut server = ctx.socket(zmq::REP).unwrap();
        server.bind("inproc://test_host_send_chunk").unwrap();

        let mut client = ctx.socket(zmq::REQ).unwrap();
        client.connect("inproc://test_host_send_chunk").unwrap();

        let mut host = Host::test_new(None, None, Some(client), None);

        let bytes = [1, 2, 3];

        host.send_chunk("/tmp/moo", 0, &bytes).unwrap();

        assert_eq!(server.recv_string(0).unwrap().unwrap(), "/tmp/moo");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_string(0).unwrap().unwrap(), "0");
        assert!(server.get_rcvmore().unwrap());
        assert_eq!(server.recv_bytes(0).unwrap(), &bytes);
    }

    #[test]
    fn test_host_recv_header_ok() {
        let mut ctx = zmq::Context::new();

        let mut req = ctx.socket(zmq::REQ).unwrap();
        req.connect("inproc://test_host_recv_header_ok").unwrap();
        req.send_str("Ok", 0).unwrap();

        let mut rep = ctx.socket(zmq::REP).unwrap();
        rep.bind("inproc://test_host_recv_header_ok").unwrap();

        let mut host = Host::test_new(None, Some(rep), None, None);
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

        let mut host = Host::test_new(None, Some(rep), None, None);
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

        let mut host = Host::test_new(None, Some(rep), None, None);
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

        let mut host = Host::test_new(None, Some(rep), None, None);
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

        let mut host = Host::test_new(None, Some(rep), None, None);
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

        let mut host = Host::test_new(None, Some(rep), None, None);
        assert!(host.expect_recvmsg("Frame 0", 0).is_err());
    }
}
