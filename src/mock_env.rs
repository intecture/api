// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::{ZCert, ZSock, ZSys};
use project::{Language, ProjectConfig};
use std::env::set_current_dir;
use std::thread::{JoinHandle, spawn};
use tempdir::TempDir;
use write_conf;

pub struct MockEnv {
    _auth_handler: JoinHandle<()>,
    pub _proj_dir: TempDir,
}

impl MockEnv {
    pub fn new() -> MockEnv {
        let proj_dir = TempDir::new("remote_host_connect").unwrap();
        set_current_dir(proj_dir.path()).unwrap();

        let cert = ZCert::new().unwrap();
        cert.save_secret("user.crt").unwrap();

        let cert = ZCert::new().unwrap();
        cert.save_public("auth.crt").unwrap();
        cert.save_secret(".auth_secret.crt").unwrap();

        ZSys::init();
        let mut sock = ZSock::new(::czmq::SocketType::REP);
        let cert = ZCert::load(".auth_secret.crt").unwrap();
        cert.apply(&mut sock);
        sock.set_curve_server(true);
        sock.set_zap_domain("mock_auth_server");
        let port = sock.bind("tcp://127.0.0.1:*[60000-]").unwrap();

        let config = ProjectConfig {
            language: Language::Rust,
            auth_server: "127.0.0.1".into(),
            auth_api_port: 0,
            auth_update_port: port as u32,
        };
        write_conf(&config, "project.json").unwrap();

        let handle = spawn(move|| MockEnv::auth_handler(sock));

        MockEnv {
            _auth_handler: handle,
            _proj_dir: proj_dir,
        }
    }

    // This fn exists to coerce lazy_static into initialising a new
    // MockEnv instance.
    pub fn init(&self) {}

    fn auth_handler(mut sock: ZSock) {
        loop {
            sock.recv_str().unwrap().unwrap();

            let reply = ::czmq::ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("0000000000000000000000000000000000000000").unwrap();
            reply.send(&mut sock).unwrap();
        }
    }
}
