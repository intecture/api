// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The host wrapper for communicating with a remote host.

use czmq::{ZCert, ZMsg, ZSock, ZSockType};
use error::{Error, Result};
use std::path::Path;
use super::*;
use zfilexfer;

impl Host {
    /// Create a new Host to represent your managed host.
    pub fn new() -> Host {
        Host {
            hostname: None,
            api_sock: None,
            file_sock: None,
        }
    }

    #[cfg(test)]
    pub fn test_new(hostname: Option<String>, api_sock: Option<ZSock>, file_sock: Option<ZSock>) -> Host {
        let host = Host {
            hostname: hostname,
            api_sock: api_sock,
            file_sock: file_sock,
        };

        host
    }

    pub fn connect(&mut self, hostname: &str, api_port: u32, file_port: u32) -> Result<()> {
        self.hostname = Some(hostname.to_string());

        let user_cert = try!(ZCert::load("user.crt"));
        let server_cert = try!(self.lookup_server_cert(hostname, &::PROJECT_CONFIG.auth_server, &user_cert));

        let api_sock = ZSock::new(ZSockType::REQ);
        user_cert.apply(&api_sock);
        api_sock.set_curve_serverkey(server_cert.public_txt());
        api_sock.set_sndtimeo(Some(1800000));
        api_sock.set_rcvtimeo(Some(1800000));
        try!(api_sock.connect(&format!("tcp://{}:{}", hostname, api_port)));
        self.api_sock = Some(api_sock);

        let file_sock = ZSock::new(ZSockType::DEALER);
        user_cert.apply(&file_sock);
        file_sock.set_curve_serverkey(server_cert.public_txt());
        file_sock.set_sndtimeo(Some(1800000));
        file_sock.set_rcvtimeo(Some(1800000));
        try!(file_sock.connect(&format!("tcp://{}:{}", hostname, file_port)));
        self.file_sock = Some(file_sock);

        Ok(())
    }

    fn lookup_server_cert(&self, hostname: &str, auth_server: &str, user_cert: &ZCert) -> Result<ZCert> {
        let auth_cert = try!(ZCert::load("auth.crt"));

        let auth_sock = ZSock::new(ZSockType::REQ);
        user_cert.apply(&auth_sock);
        auth_sock.set_curve_serverkey(auth_cert.public_txt());
        auth_sock.set_sndtimeo(Some(10000));
        auth_sock.set_rcvtimeo(Some(10000));
        try!(auth_sock.connect(&format!("tcp://{}", auth_server)));

        // Get server cert from Auth server
        let msg = ZMsg::new();
        try!(msg.addstr("cert::lookup"));
        try!(msg.addstr(hostname));
        try!(msg.send(&auth_sock));

        let reply = try!(ZMsg::recv(&auth_sock));

        if reply.size() != 2 {
            return Err(Error::HostResponse);
        }

        match try!(reply.popstr().unwrap().or(Err(Error::HostResponse))).as_ref() {
            "Ok" => {
                let pk = try!(reply.popstr().unwrap().or(Err(Error::HostResponse)));
                Ok(ZCert::from_txt(&pk, "0000000000000000000000000000000000000000"))
            },
            "Err" => Err(Error::Auth(try!(reply.popstr().unwrap().or(Err(Error::HostResponse))))),
            _ => unreachable!(),
        }
    }

    pub fn close(&mut self) -> Result<()> {
        if self.api_sock.is_some() {
            self.api_sock.take().unwrap();
        }

        if self.file_sock.is_some() {
            self.file_sock.take().unwrap();
        }

        Ok(())
    }

    #[doc(hidden)]
    pub fn send(&mut self, msg: ZMsg) -> Result<()> {
        if self.api_sock.is_none() {
            return Err(Error::HostDisconnected);
        }

        try!(msg.send(self.api_sock.as_mut().unwrap()));
        Ok(())
    }

    #[doc(hidden)]
    pub fn send_file<P: AsRef<Path>>(&mut self, file: &zfilexfer::File, remote_path: P) -> Result<()> {
        if self.file_sock.is_none() {
            return Err(Error::HostDisconnected);
        }

        try!(file.send(self.file_sock.as_ref().unwrap(), remote_path));
        Ok(())
    }

    #[doc(hidden)]
    pub fn recv(&self, min: usize, max: Option<usize>) -> Result<ZMsg> {
        let msg = try!(ZMsg::recv(self.api_sock.as_ref().unwrap()));
        try!(Self::extract_header(&msg));

        // Check msg size
        if msg.size() < min || (max.is_some() && msg.size() > max.unwrap()) {
            Err(Error::HostResponse)
        } else {
            Ok(msg)
        }
    }

    fn extract_header(msg: &ZMsg) -> Result<()> {
        if msg.size() == 0 {
            return Err(Error::HostResponse);
        }

        match try!(msg.popstr().unwrap().or(Err(Error::HostResponse))).as_ref() {
            "Ok" => Ok(()),
            "Err" => {
                if msg.size() == 0 {
                    Err(Error::HostResponse)
                } else {
                    Err(Error::Agent(try!(msg.popstr().unwrap().or(Err(Error::HostResponse)))))
                }
            },
            _ => Err(Error::HostResponse),
        }
    }
}

#[cfg(test)]
mod tests {
    use czmq::{ZMsg, ZSys};
    use host::Host;
    use std::fs;
    use std::thread::spawn;
    use tempdir::TempDir;
    use zfilexfer::File;

    #[test]
    fn test_connect_close() {
        let _ = ::_MOCK_ENV.init();

        let mut host = Host::new();
        assert!(host.connect("localhost", 7101, 7102).is_ok());
        assert!(host.close().is_ok());
    }

    #[test]
    fn test_send_recv() {
        let _ = ::_MOCK_ENV.init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let mut host1 = Host::test_new(None, Some(client), None);
        let mut host2 = Host::test_new(None, Some(server), None);

        let msg = ZMsg::new();
        msg.addstr("Ok").unwrap();
        msg.addstr("moo").unwrap();
        msg.addstr("cow").unwrap();
        host1.send(msg).unwrap();

        let reply = host2.recv(2, Some(2)).unwrap();
        assert_eq!(reply.popstr().unwrap().unwrap(), "moo");
        assert_eq!(reply.popstr().unwrap().unwrap(), "cow");

        let msg = ZMsg::new();
        msg.addstr("No header").unwrap();
        host2.send(msg).unwrap();

        assert!(host1.recv(0, None).is_err());
        let msg = ZMsg::new();
        msg.addstr("Err").unwrap();
        host1.send(msg).unwrap();

        assert!(host2.recv(0, None).is_err());
    }

    #[test]
    fn test_send_file() {
        let _ = ::_MOCK_ENV.init();

        let tempdir = TempDir::new("host_test_send_file").unwrap();
        let path = format!("{}/file.txt", tempdir.path().to_str().unwrap());
        fs::File::create(&path).unwrap();
        let file = File::open(&path, None).unwrap();

        let (client, server) = ZSys::create_pipe().unwrap();
        client.set_rcvtimeo(Some(500));
        server.set_rcvtimeo(Some(500));

        let handle = spawn(move|| {
            let msg = ZMsg::recv(&server).unwrap();
            assert_eq!(msg.popstr().unwrap().unwrap(), "NEW");

            server.flush();
            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, None, Some(client));
        assert!(host.send_file(&file, &path).is_ok());

        handle.join().unwrap();
    }
}
