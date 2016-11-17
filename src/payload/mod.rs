// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Payloads are self-contained projects that encapsulate a specific
//! feature or system function. Think of them as reusable chunks of
//! code that can be run across multiple hosts. Any time you have a
//! task that you want to repeat, it should probably go into a
//! payload.
//!
//! For example, a payload could handle installing a specific
//! package, such as Nginx. Or, you could create a payload that
//! configures iptables.
//!
//! # Examples
//!
//! ```no_run
//! # use inapi::{Host, Payload};
#![cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(Some(\"nodes/mynode.json\")).unwrap();")]
#![cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"nodes/mynode.json\").unwrap();")]
//! let payload = Payload::new("nginx::install").unwrap(); // format is "payload::executable"
//! payload.run(&mut host, None).unwrap();
//! ```

pub mod config;
pub mod ffi;

use czmq::{ZMsg, ZPoller, ZSock, SocketType, ZSys};
use error::{Error, Result};
use host::{Host,HostSendRecv};
use project::Language;
use self::config::Config;
use serde_json;
use std::env::{current_dir, set_current_dir};
use std::process;
use std::path::PathBuf;
use std::thread;
use zdaemon::ConfigFile;

/// Container for running a Payload.
pub struct Payload {
    /// Path to the payload directory.
    path: PathBuf,
    /// Name of the executable/source file to run.
    artifact: Option<String>,
    /// Language the payload is written in.
    language: Language,
}

impl Payload {
    /// Create a new Payload using the payload::artifact notation.
    /// This notation is simply "payload" + separator ("::") +
    /// "executable/source file". For example: "nginx::install".
    ///
    /// By default, payloads live in
    /// <project root>/payloads/<payload_name>. Thus the payload name
    /// "nginx" will resolve to <project root>/payloads/nginx/. You
    /// can also specify an absolute path to your payload, which will
    /// override the resolved path.
    ///
    /// ```no_run
    /// # use inapi::Payload;
    /// // Using standard payload/artifact notation...
    /// let payload = Payload::new("iptables::update").unwrap();
    ///
    /// // Using an absolute path...
    /// let payload = Payload::new("/mnt/intecture/payloads/iptables::update").unwrap();
    /// ```
    pub fn new(payload_artifact: &str) -> Result<Payload> {
        let mut parts: Vec<&str> = payload_artifact.split("::").collect();
        let payload = if parts.len() > 0 {
            parts.remove(0)
        } else {
            return Err(Error::Generic("Invalid payload string".into()));
        };

        let mut buf = PathBuf::from("payloads");
        buf.push(payload);

        buf.push("payload.json");
        let config = try!(Config::load(&buf));
        buf.pop();

        // Check dependencies
        if let Some(deps) = config.dependencies {
            try!(Self::check_deps(&deps));
        }

        Ok(Payload {
            path: buf,
            artifact: if parts.len() > 0 {
                Some(parts.remove(0).into())
            } else {
                None
            },
            language: config.language,
        })
    }

    /// Compile a payload's source code. This function is also called
    /// by Payload::run(), but is useful for precompiling payloads
    /// ahead of time to catch build errors early.
    ///
    /// Note that this is only useful for compiled languages. If this
    /// function is run on a payload that uses an interpreted
    /// language, it will safely be ignored.
    pub fn build(&self) -> Result<()> {
        let mut make_path = self.path.clone();
        make_path.push("Makefile");

        match self.language {
            Language::C | Language::Rust if make_path.exists() && make_path.is_file() => {
                let current_dir = try!(current_dir());
                try!(set_current_dir(&self.path));

                let output = try!(process::Command::new("make")
                                                   .stdout(process::Stdio::inherit())
                                                   .output());

                try!(set_current_dir(&current_dir));

                if !output.status.success() {
                    return Err(Error::BuildFailed(try!(String::from_utf8(output.stderr))).into());
                }
            },
            Language::Rust => {
                let manifest_path = format!("{}/Cargo.toml", self.path.to_str().unwrap());
                let output = try!(process::Command::new("cargo")
                                                   .args(&["build", "--release", "--manifest-path", &manifest_path])
                                                   .stdout(process::Stdio::inherit())
                                                   .output());
                if !output.status.success() {
                    return Err(Error::BuildFailed(try!(String::from_utf8(output.stderr))).into());
                }
            },
            _ => ()
        }

        Ok(())
    }

    /// Execute the payload's artifact.
    ///
    /// For compiled languages, the artifact will be executed
    /// directly. For interpreted languages, the artifact will be
    /// passed as an argument to the interpreter.
    ///
    /// ```no_run
    /// # use inapi::{Host, Payload};
    #[cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(Some(\"nodes/mynode.json\")).unwrap();")]
    #[cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"nodes/mynode.json\").unwrap();")]
    /// let payload = Payload::new("iptables::configure").unwrap();
    /// payload.run(&mut host, Some(vec![
    ///     "add_rule",
    ///     "..."
    /// ])).unwrap();
    /// ```
    pub fn run(&self, host: &mut Host, user_args: Option<Vec<&str>>) -> Result<()> {
        // XXX This ugly cloning is a wasteful solution. New ZMQ
        // socket types (SERVER & CLIENT) are threadsafe, which will
        // allow us to thread the ZMQ poller and avoid having to
        // clone half the world. When these are implemented, this
        // code should be changed.

        // Build payload to make sure it's up to date
        try!(self.build());

        let artifact_default = self.artifact.as_ref().map(|a| &**a).unwrap_or("main");
        let api_endpoint = format!("ipc://{}/{}_api.ipc", self.path.to_str().unwrap(), artifact_default);
        let mut api_pipe = ZSock::new(SocketType::DEALER);
        try!(api_pipe.bind(&api_endpoint));

        let file_endpoint = format!("ipc://{}/{}_file.ipc", self.path.to_str().unwrap(), artifact_default);
        let mut file_pipe = ZSock::new(SocketType::DEALER);
        try!(file_pipe.bind(&file_endpoint));

        let (mut parent, child) = try!(ZSys::create_pipe());
        let language = self.language.clone();
        let mut payload_path = self.path.clone();
        let artifact = self.artifact.clone();
        let user_args_c: Option<Vec<String>> = match user_args {
            Some(a) => Some(a.into_iter().map(|arg| String::from(arg)).collect()),
            None => None,
        };

        let handle = thread::spawn(move || {
            match language {
                Language::C => {
                    payload_path.push(artifact.as_ref().map(|a| &**a).unwrap_or("main"));

                    let mut args = vec![api_endpoint, file_endpoint];
                    if let Some(mut a) = user_args_c {
                        args.append(&mut a);
                    }

                    let output = try!(process::Command::new(payload_path.to_str().unwrap())
                                                       .args(&args)
                                                       .stdout(process::Stdio::inherit())
                                                       .output());

                    if !output.status.success() {
                        try!(child.signal(0));
                        return Err(Error::RunFailed(try!(String::from_utf8(output.stderr))).into());
                    }
                },
                Language::Php => {
                    payload_path.push("src");
                    payload_path.push(artifact.as_ref().map(|a| &**a).unwrap_or("main"));
                    if payload_path.extension().is_none() {
                        payload_path.set_extension("php");
                    }

                    let mut args = vec![payload_path.to_str().unwrap().into(), api_endpoint, file_endpoint];
                    if let Some(mut a) = user_args_c {
                        args.append(&mut a);
                    }

                    let output = try!(process::Command::new("php")
                                                       .args(&args)
                                                       .stdout(process::Stdio::inherit())
                                                       .output());

                    if !output.status.success() {
                        try!(child.signal(0));
                        return Err(Error::RunFailed(try!(String::from_utf8(output.stderr))).into());
                    }
                },
                Language::Rust => {
                    if let Some(a) = artifact {
                        payload_path.push("target/release");
                        payload_path.push(a);
                    } else {
                        let dirname = try!(try!(payload_path.file_stem().ok_or(Error::RunFailed("Invalid payload path".into())))
                                                            .to_str().ok_or(Error::RunFailed("Invalid payload path".into())))
                                                            .to_owned();
                        payload_path.push("target/release");
                        payload_path.push(&dirname);
                    }

                    let mut args = vec![
                        api_endpoint,
                        file_endpoint
                    ];

                    if let Some(mut a) = user_args_c {
                        args.append(&mut a);
                    }

                    let output = try!(process::Command::new(&payload_path)
                                                       .args(&args)
                                                       .stdout(process::Stdio::inherit())
                                                       .output());

                    if !output.status.success() {
                        try!(child.signal(0));
                        return Err(Error::RunFailed(try!(String::from_utf8(output.stderr))).into());
                    }
                }
            }

            try!(child.signal(0));
            Ok(())
        });

        // Send data to payload
        let json = try!(serde_json::to_string(host.data()));
        try!(api_pipe.send_str(&json));

        let mut poller = try!(ZPoller::new());
        try!(poller.add(&mut parent));
        try!(poller.add(&mut api_pipe));
        try!(poller.add(&mut file_pipe));

        loop {
            let sock: Option<ZSock> = poller.wait(None);
            if let Some(mut s) = sock {
                if s == api_pipe {
                    let req = try!(ZMsg::recv(&mut s));
                    try!(host.send(req));

                    let reply = try!(host.recv_raw());
                    try!(reply.send(&mut s));
                }
                else if s == file_pipe {
                    let req = try!(ZMsg::recv(&mut s));
                    try!(host.send_file(req));

                    let reply = try!(host.recv_file_raw());
                    try!(reply.send(&mut s));
                }
                else if s == parent {
                    break;
                } else {
                    unreachable!();
                }
            }

            if poller.terminated() {
                break;
            }
        }

        let cmd_result: Result<()> = try!(handle.join());
        try!(cmd_result);

        Ok(())
    }

    fn check_deps(payloads: &[String]) -> Result<()> {
        for payload in payloads {
            try!(Payload::new(payload));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use czmq::{ZSock, SocketType};
    use host::Host;
    use project::Language;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use std::thread;
    use super::*;
    use super::config::Config;
    use tempdir::TempDir;
    use zdaemon::ConfigFile;

    #[test]
    fn test_new_nodeps() {
        let _ = ::_MOCK_ENV.init();

        let tempdir = TempDir::new("test_payload_new_nodeps").unwrap();
        let mut buf = tempdir.path().to_owned();

        create_cargo_proj(&mut buf);

        let conf = Config {
            author: "Dr. Hibbert".into(),
            repository: "https://github.com/dhibbz/hehehe.git".into(),
            language: Language::Rust,
            dependencies: Some(vec!["missing_payload".into()]),
        };

        buf.push("payload.json");
        conf.save(&buf).unwrap();
        buf.pop();

        assert!(Payload::new(buf.to_str().unwrap()).is_err());
    }

    pub fn test_build_c() {
        let tempdir = TempDir::new("test_payload_build_c").unwrap();
        let mut buf = tempdir.path().to_owned();

        buf.push("Makefile");
        let mut fh = fs::File::create(&buf).unwrap();
        fh.write_all(b"all:
\ttouch test").unwrap();
        buf.pop();

        let conf = Config {
            author: "Dr. Hibbert".into(),
            repository: "https://github.com/dhibbz/hehehe.git".into(),
            language: Language::C,
            dependencies: None,
        };

        buf.push("payload.json");
        conf.save(&buf).unwrap();
        buf.pop();

        let payload = Payload::new(buf.to_str().unwrap()).unwrap();
        payload.build().unwrap();

        buf.push("test");
        assert!(buf.exists());
    }

    pub fn test_run() {
        let tempdir = TempDir::new("test_payload_run").unwrap();
        let mut buf = tempdir.path().to_owned();

        create_cargo_proj(&mut buf);

        let conf = Config {
            author: "Dr. Hibbert".into(),
            repository: "https://github.com/dhibbz/hehehe.git".into(),
            language: Language::Rust,
            dependencies: None,
        };

        buf.push("payload.json");
        conf.save(&buf).unwrap();
        buf.pop();

        let payload_name = buf.into_os_string().into_string().unwrap();
        let payload_name_clone = payload_name.clone();

        let handle = thread::spawn(move || {
            let s = ZSock::new(SocketType::DEALER);
            s.connect(&format!("ipc://{}/main_api.ipc", payload_name_clone)).unwrap();
            s.recv_str().unwrap().unwrap();
        });

        let mut host = Host::test_new(None, None, None, None);
        let payload = Payload::new(&payload_name).unwrap();
        payload.run(&mut host, Some(vec!["abc"])).unwrap();

        handle.join().unwrap();
    }

    pub fn create_cargo_proj(buf: &mut PathBuf) {
        buf.push("payload/src");
        fs::create_dir_all(&buf).unwrap();

        buf.push("main.rs");
        let mut fh = fs::File::create(&buf).unwrap();
        fh.write_all(b"fn main() {}").unwrap();
        buf.pop();
        buf.pop();

        buf.push("Cargo.toml");
        let mut fh = fs::File::create(&buf).unwrap();
        fh.write_all(b"[package]
name = \"payload\"
version = \"0.1.0\"
authors = [\"Don Duck <quack@goosehat.rz>\"]").unwrap();
        buf.pop();
    }
}
