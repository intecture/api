// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use command::providers::ExitStatus;
use errors::*;
use futures::{future, Future};
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::{mpsc, oneshot};
use host::{Host, HostType};
use host::local::Local;
use host::remote::Plain;
use provider::Provider;
use remote::{CommandRequest, CommandResponse, Executable, ExecutableResult,
             GenericRequest, Request, Response, ResponseResult};
use serde_json;
use std::io::{self, BufReader};
use std::process::{Command, Stdio};
use std::result;
use super::CommandProvider;
use tokio_core::reactor::Handle;
use tokio_io::io::lines;
use tokio_process::CommandExt;
use tokio_proto::streaming::{Body, Message};

#[derive(Clone)]
pub struct Generic;
struct LocalGeneric;
struct RemoteGeneric;

impl<H: Host + 'static> Provider<H> for Generic {
    fn available(host: &H) -> Box<Future<Item = bool, Error = Error>> {
        match host.get_type() {
            HostType::Local(_) => LocalGeneric::available(),
            HostType::Remote(r) => RemoteGeneric::available(r),
        }
    }

    fn try_new(host: &H) -> Box<Future<Item = Option<Generic>, Error = Error>> {
        let host = host.clone();
        Box::new(Self::available(&host)
            .and_then(|available| {
                if available {
                    future::ok(Some(Generic))
                } else {
                    future::ok(None)
                }
            }))
    }
}

impl<H: Host + 'static> CommandProvider<H> for Generic {
    fn exec(&self, host: &H, handle: &Handle, cmd: &str, shell: &[String]) ->
        Box<Future<Item = (
            Box<Stream<Item = String, Error = Error>>,
            Box<Future<Item = ExitStatus, Error = Error>>
        ), Error = Error>>
    {
        match host.get_type() {
            HostType::Local(_) => LocalGeneric::exec(handle, cmd, shell),
            HostType::Remote(r) => RemoteGeneric::exec(r, cmd, shell),
        }
    }
}

impl LocalGeneric {
    fn available() -> Box<Future<Item = bool, Error = Error>> {
        Box::new(future::ok(cfg!(unix)))
    }

    fn exec(handle: &Handle, cmd: &str, shell: &[String]) ->
        Box<Future<Item = (
            Box<Stream<Item = String, Error = Error>>,
            Box<Future<Item = ExitStatus, Error = Error>>
        ), Error = Error>>
    {
        let cmd = cmd.to_owned();
        let shell = shell.to_owned();
        let (shell, shell_args) = match shell.split_first() {
            Some((s, a)) => (s, a),
            None => return Box::new(future::err("Invalid shell provided".into())),
        };

        let child = Command::new(shell)
            .args(shell_args)
            .arg(&cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn_async(handle)
            .chain_err(|| "Command execution failed");
        let mut child = match child {
            Ok(c) => c,
            Err(e) => return Box::new(future::err(e)),
        };

        let stdout = child.stdout().take().unwrap();
        let outbuf = BufReader::new(stdout);
        let stderr = child.stderr().take().unwrap();
        let errbuf = BufReader::new(stderr);
        let lines = lines(outbuf).select(lines(errbuf));

        Box::new(future::ok(
            (Box::new(lines.then(|r| r.chain_err(|| "Command execution failed"))) as Box<Stream<Item = String, Error = Error>>,
            Box::new(child.then(|r| match r.chain_err(|| "Command execution failed") {
                Ok(c) => future::ok(ExitStatus {
                    success: c.success(),
                    code: c.code(),
                }),
                Err(e) => future::err(e)
            })) as Box<Future<Item = ExitStatus, Error = Error>>)
        ))
    }
}

impl RemoteGeneric {
    fn available(host: &Plain) -> Box<Future<Item = bool, Error = Error>> {
        let runnable = Request::Command(
                          CommandRequest::Generic(
                              GenericRequest::Available));
        Box::new(host.call_req(runnable)
            .chain_err(|| ErrorKind::Request { endpoint: "Command::Generic", func: "available" })
            .map(|msg| match msg.into_inner() {
                Response::Command(CommandResponse::Available(b)) => b,
                _ => unreachable!(),
            }))
    }

    fn exec(host: &Plain, cmd: &str, shell: &[String]) ->
        Box<Future<Item = (
            Box<Stream<Item = String, Error = Error>>,
            Box<Future<Item = ExitStatus, Error = Error>>
        ), Error = Error>>
    {
        let runnable = Request::Command(
                          CommandRequest::Generic(
                              GenericRequest::Exec(cmd.into(), shell.to_owned())));
        Box::new(host.call_req(runnable)
            .chain_err(|| ErrorKind::Request { endpoint: "Command::Generic", func: "exec" })
            .map(|mut msg| {
                let (tx, rx) = oneshot::channel::<ExitStatus>();
                let mut tx_share = Some(tx);
                let mut found = false;
                (
                    Box::new(msg.take_body()
                        .expect("Command::exec reply missing body stream")
                        .filter_map(move |v| {
                            let s = String::from_utf8_lossy(&v).to_string();

                            // @todo This is a heuristical approach which is fallible
                            if !found && s.starts_with("ExitStatus:") {
                                let (_, json) = s.split_at(11);
                                match serde_json::from_str(json) {
                                    Ok(status) => {
                                        // @todo What should happen if this fails?
                                        let _ = tx_share.take().unwrap().send(status);
                                        found = true;
                                        return None;
                                    },
                                    _ => (),
                                }
                            }

                            Some(s)
                        })
                        .then(|r| r.chain_err(|| "Command execution failed"))
                    ) as Box<Stream<Item = String, Error = Error>>,
                    Box::new(rx.chain_err(|| "Buffer dropped before ExitStatus was sent"))
                        as Box<Future<Item = ExitStatus, Error = Error>>
                )
            }))
    }
}

impl Executable for GenericRequest {
    fn exec(self, _: &Local, handle: &Handle) -> ExecutableResult {
        match self {
            GenericRequest::Available => Box::new(
                LocalGeneric::available()
                    .map(|b| Message::WithoutBody(
                        ResponseResult::Ok(
                            Response::Command(
                                CommandResponse::Available(b)))))),
            GenericRequest::Exec(cmd, shell) => {
                let handle = handle.clone();
                Box::new(LocalGeneric::exec(&handle, &cmd, &shell)
                    .and_then(move |(lines, status)| {
                        let (tx1, body) = Body::pair();
                        let tx2 = tx1.clone();
                        let msg = Message::WithBody(ResponseResult::Ok(Response::Command(CommandResponse::Exec)), body);
                        let stream = lines.map(|s| Ok(s.into_bytes()))
                            .forward(tx1.sink_map_err(|e| Error::with_chain(e, "Could not forward command output to Body")))
                            .join(status.and_then(|s| match serde_json::to_string(&s)
                                .chain_err(|| "Could not serialize `ExitStatus` struct")
                                {
                                    Ok(s) => {
                                        let mut frame = "ExitStatus:".to_owned();
                                        frame.push_str(&s);
                                        Box::new(tx2.send(Ok(frame.into_bytes()))
                                            .map_err(|e| Error::with_chain(e, "Could not forward command output to Body"))
                                        ) as Box<Future<Item = mpsc::Sender<result::Result<Vec<u8>, io::Error>>, Error = Error>>
                                    },
                                    Err(e) => Box::new(future::err(e)),
                                }))
                            // @todo We should repatriate these errors somehow
                            .map(|_| ())
                            .map_err(|_| ());
                        handle.spawn(stream);
                        future::ok(msg)
                    }))
            },
        }
    }
}
