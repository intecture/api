// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Host primitive.

#[macro_use]
pub mod data;
pub mod ffi;
pub mod telemetry;

pub use self::telemetry::TelemetryTarget;

#[cfg(feature = "remote-run")]
use czmq::{ZCert, ZMsg, ZSock, SocketType};
#[cfg(feature = "remote-run")]
use error::Error;
use error::Result;
#[cfg(feature = "remote-run")]
use serde_json;
use serde_json::Value;
#[cfg(feature = "remote-run")]
use std::mem;
use std::path::Path;
use std::rc::Rc;
#[cfg(feature = "remote-run")]
use zfilexfer;

#[cfg(feature = "local-run")]
/// Primitive for communicating with a managed host.
///
///# Examples
///
/// ```no_run
/// # use inapi::{Command, Host};
///let path: Option<String> = None;
///let mut host = Host::local(path).unwrap();
///
///let cmd = Command::new("whoami");
///let result = cmd.exec(&mut host).unwrap();
/// ```
pub struct Host {
    /// Data for host, comprising data files and telemetry
    data: Rc<Value>,
}

#[cfg(feature = "remote-run")]
/// Primitive for communicating with a managed host.
///
///# Examples
///
/// ```no_run
/// # use inapi::{Command, Host};
///let mut host = Host::connect("hosts/myhost.json").unwrap();
///
///let cmd = Command::new("whoami");
///let result = cmd.exec(&mut host).unwrap();
/// ```
pub struct Host {
    /// Hostname or IP of managed host
    pub hostname: String,
    /// API socket
    api_sock: Option<ZSock>,
    /// File transfer socket
    file_sock: Option<ZSock>,
    /// Data for host, comprising data files and telemetry
    data: Rc<Value>,
}

impl Host {
    #[cfg(feature = "local-run")]
    /// Create a new Host connected to localhost.
    pub fn local<P: AsRef<Path>>(path: Option<P>) -> Result<Host> {
        let mut me = Host {
            data: Rc::new(Value::Null),
        };

        let telemetry = try!(telemetry::Telemetry::init(&mut me));

        match path {
            Some(p) => {
                let value = try!(data::open(p));
                me.data = Rc::new(try!(data::merge(value, telemetry)));
            },
            None => me.data = Rc::new(telemetry),
        }

        Ok(me)
    }

    #[cfg(feature = "remote-run")]
    /// Create a new Host connected to the endpoint specified in the
    /// data file.
    ///
    /// This function expects to find the following keys in the root
    /// namespace: "hostname", "api_port", "file_port".
    pub fn connect<P: AsRef<Path>>(path: P) -> Result<Host> {
        let value = try!(data::open(path.as_ref()));
        let mut me = try!(Self::connect_endpoint(try!(needstr!(value => "/hostname")),
                                                 try!(needu64!(value => "/api_port")) as u32,
                                                 try!(needu64!(value => "/file_port")) as u32));

        let mut telemetry = Rc::new(Value::Null);
        mem::swap(&mut telemetry, &mut me.data);
        // We can use unwrap() here safely as we can guarantee that
        // there is only one strong reference to telemetry.
        me.data = Rc::new(try!(data::merge(value, Rc::try_unwrap(telemetry).unwrap())));

        Ok(me)
    }

    #[cfg(feature = "remote-run")]
    /// Create a new Host connected to the specified endpoint. Note
    /// that this function does not load any user data.
    pub fn connect_endpoint(hostname: &str, api_port: u32, file_port: u32) -> Result<Host> {
        let user_cert = try!(ZCert::load("user.crt"));
        let server_cert = try!(Self::lookup_server_cert(hostname, &user_cert));

        let mut api_sock = ZSock::new(SocketType::REQ);
        user_cert.apply(&mut api_sock);
        api_sock.set_curve_serverkey(server_cert.public_txt());
        api_sock.set_sndtimeo(Some(10000)); // 10 seconds
        api_sock.set_rcvtimeo(Some(10000));
        try!(api_sock.connect(&format!("tcp://{}:{}", hostname, api_port)));
        api_sock.set_sndtimeo(Some(1800000)); // 30 minutes
        api_sock.set_rcvtimeo(Some(1800000));

        let mut file_sock = ZSock::new(SocketType::DEALER);
        user_cert.apply(&mut file_sock);
        file_sock.set_curve_serverkey(server_cert.public_txt());
        file_sock.set_sndtimeo(Some(10000)); // 10 seconds
        file_sock.set_rcvtimeo(Some(10000));
        try!(file_sock.connect(&format!("tcp://{}:{}", hostname, file_port)));
        file_sock.set_sndtimeo(Some(1800000)); // 30 minutes
        file_sock.set_rcvtimeo(Some(1800000));

        let mut me = Host {
            hostname: hostname.into(),
            api_sock: Some(api_sock),
            file_sock: Some(file_sock),
            data: Rc::new(Value::Null),
        };
        me.data = Rc::new(try!(telemetry::Telemetry::init(&mut me)));

        Ok(me)
    }

    #[cfg(feature = "remote-run")]
    /// Create a new Host specifically for use inside a payload.
    pub fn connect_payload(api_endpoint: &str, file_endpoint: &str) -> Result<Host> {
        let api_sock = ZSock::new(SocketType::DEALER);
        try!(api_sock.connect(api_endpoint));

        let file_sock = ZSock::new(SocketType::DEALER);
        try!(file_sock.connect(file_endpoint));

        let data_json = try!(api_sock.recv_str()).unwrap();
        let data = try!(serde_json::from_str(&data_json));

        Ok(Host {
            hostname: "payload".into(),
            api_sock: Some(api_sock),
            file_sock: Some(file_sock),
            data: Rc::new(data),
        })
    }

    /// Get data for Host.
    pub fn data(&self) -> &Value {
        &self.data
    }

    /// Get a reference counted version of data for Host.
    pub fn data_owned(&self) -> Rc<Value> {
        self.data.clone()
    }

    #[cfg(feature = "remote-run")]
    fn lookup_server_cert(hostname: &str, user_cert: &ZCert) -> Result<ZCert> {
        let auth_cert = try!(ZCert::load("auth.crt"));

        let mut auth_sock = ZSock::new(SocketType::REQ);
        user_cert.apply(&mut auth_sock);
        auth_sock.set_curve_serverkey(auth_cert.public_txt());
        auth_sock.set_sndtimeo(Some(10000));
        auth_sock.set_rcvtimeo(Some(10000));
        try!(auth_sock.connect(&format!("tcp://{}:{}", ::PROJECT_CONFIG.auth_server, ::PROJECT_CONFIG.auth_api_port)));

        // Get server cert from Auth server
        let msg = ZMsg::new();
        try!(msg.addstr("cert::lookup"));
        try!(msg.addstr(hostname));
        try!(msg.send(&mut auth_sock));

        let reply = try!(ZMsg::recv(&mut auth_sock));

        if reply.size() != 2 {
            return Err(Error::HostResponse);
        }

        match try!(reply.popstr().unwrap().or(Err(Error::HostResponse))).as_ref() {
            "Ok" => {
                let pk = try!(reply.popstr().unwrap().or(Err(Error::HostResponse)));
                Ok(try!(ZCert::from_txt(&pk, "0000000000000000000000000000000000000000")))
            },
            "Err" => Err(Error::Auth(try!(reply.popstr().unwrap().or(Err(Error::HostResponse))))),
            _ => Err(Error::HostResponse),
        }
    }

    #[cfg(all(test, feature = "remote-run"))]
    pub fn test_new(hostname: Option<String>, api_sock: Option<ZSock>, file_sock: Option<ZSock>, data: Option<Value>) -> Host {
        let host = Host {
            hostname: hostname.unwrap_or(String::new()),
            api_sock: api_sock,
            file_sock: file_sock,
            data: match data {
                Some(d) => Rc::new(d),
                None => Rc::new(Value::Null),
            },
        };

        host
    }
}

#[cfg(feature = "remote-run")]
pub trait HostSendRecv {
    fn send(&mut self, msg: ZMsg) -> Result<()>;
    fn send_file(&mut self, msg: ZMsg) -> Result<()>;
    fn send_fs_file<P: AsRef<Path>>(&mut self, file: &mut zfilexfer::File, remote_path: P) -> Result<()>;
    fn recv(&mut self, min: usize, max: Option<usize>) -> Result<ZMsg>;
    fn recv_raw(&mut self) -> Result<ZMsg>;
    fn recv_file_raw(&mut self) -> Result<ZMsg>;
    fn extract_header(msg: &ZMsg) -> Result<()>;
}

#[cfg(feature = "remote-run")]
impl HostSendRecv for Host {
    fn send(&mut self, msg: ZMsg) -> Result<()> {
        if self.api_sock.is_none() {
            return Err(Error::HostDisconnected);
        }

        try!(msg.send(self.api_sock.as_mut().unwrap()));
        Ok(())
    }

    fn send_file(&mut self, msg: ZMsg) -> Result<()> {
        if self.file_sock.is_none() {
            return Err(Error::HostDisconnected);
        }

        try!(msg.send(self.file_sock.as_mut().unwrap()));
        Ok(())
    }

    fn send_fs_file<P: AsRef<Path>>(&mut self, file: &mut zfilexfer::File, remote_path: P) -> Result<()> {
        if self.file_sock.is_none() {
            return Err(Error::HostDisconnected);
        }

        try!(file.send(self.file_sock.as_mut().unwrap(), remote_path));
        Ok(())
    }

    fn recv(&mut self, min: usize, max: Option<usize>) -> Result<ZMsg> {
        if self.api_sock.is_none() {
            return Err(Error::HostDisconnected);
        }

        let msg = try!(ZMsg::recv(self.api_sock.as_mut().unwrap()));
        try!(Self::extract_header(&msg));

        // Check msg size
        if msg.size() < min || (max.is_some() && msg.size() > max.unwrap()) {
            Err(Error::HostResponse)
        } else {
            Ok(msg)
        }
    }

    fn recv_raw(&mut self) -> Result<ZMsg> {
        if self.api_sock.is_none() {
            return Err(Error::HostDisconnected);
        }

        Ok(try!(ZMsg::recv(self.api_sock.as_mut().unwrap())))
    }

    fn recv_file_raw(&mut self) -> Result<ZMsg> {
        if self.file_sock.is_none() {
            return Err(Error::HostDisconnected);
        }

        Ok(try!(ZMsg::recv(self.file_sock.as_mut().unwrap())))
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

#[cfg(feature = "remote-run")]
#[cfg(test)]
mod tests {
    use czmq::{ZMsg, ZSock, SocketType, ZSys};
    use std::fs;
    use std::thread;
    use super::*;
    use tempdir::TempDir;
    use zfilexfer::File;

    #[test]
    fn test_connect_payload() {
        let api = ZSock::new(SocketType::DEALER);
        let port = api.bind("tcp://127.0.0.1:*").unwrap();
        let api_endpoint = format!("tcp://127.0.0.1:{}", port);

        let handle = thread::spawn(move || {
            api.send_str(r#"{
                "key": "value"
            }"#).unwrap();

            api.recv_str().unwrap().unwrap();
        });

        let mut host = Host::connect_payload(&api_endpoint, "inproc://file_endpoint").unwrap();
        assert_eq!(host.data()["key"], json!("value"));

        let msg = ZMsg::new();
        msg.addstr("test").unwrap();
        host.send(msg).unwrap();

        handle.join().unwrap();
    }

    #[test]
    fn test_send_recv() {
        let _ = ::_MOCK_ENV.init();

        let (client, server) = ZSys::create_pipe().unwrap();

        let mut host1 = Host::test_new(None, Some(client), None, None);
        let mut host2 = Host::test_new(None, Some(server), None, None);

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
        let mut file = File::open(&path, None).unwrap();

        let (client, mut server) = ZSys::create_pipe().unwrap();
        client.set_rcvtimeo(Some(500));
        server.set_rcvtimeo(Some(500));

        let handle = thread::spawn(move|| {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!(msg.popstr().unwrap().unwrap(), "NEW");

            server.flush();
            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, None, Some(client), None);
        assert!(host.send_fs_file(&mut file, &path).is_ok());

        handle.join().unwrap();
    }
}
