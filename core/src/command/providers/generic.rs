// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use futures::{future, Future};
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use provider::Provider;
use remote::{ExecutableResult, ProviderName, Response, ResponseResult};
use serde_json;
use std::io::{self, BufReader};
use std::process::{Command, Stdio};
use std::result;
use super::{CommandProvider, ExitStatus};
use tokio_core::reactor::Handle;
use tokio_io::io::lines;
use tokio_process::CommandExt;
use tokio_proto::streaming::{Body, Message};

/// The generic `Command` provider.
pub struct Generic;

impl Provider for Generic {
    fn available() -> Result<bool> {
        Ok(cfg!(unix))
    }

    fn name(&self) -> ProviderName {
        ProviderName::CommandGeneric
    }
}

impl CommandProvider for Generic {
    #[doc(hidden)]
    fn exec(&self, handle: &Handle, cmd: &str, shell: &[String]) -> ExecutableResult {
        let (shell, shell_args) = match shell.split_first() {
            Some((s, a)) => (s, a),
            None => return Box::new(future::err("Invalid shell provided".into())),
        };

        let child = Command::new(shell)
            .args(shell_args)
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn_async(handle)
            .chain_err(|| "Command execution failed");
        let mut child = match child {
            Ok(c) => c,
            Err(e) => return Box::new(future::err(e)),
        };

        let (tx1, body) = Body::pair();
        let tx2 = tx1.clone();

        let stdout = child.stdout().take().unwrap();
        let outbuf = BufReader::new(stdout);
        let stderr = child.stderr().take().unwrap();
        let errbuf = BufReader::new(stderr);

        let status = child.map_err(|e| Error::with_chain(e, ErrorKind::Msg("Command execution failed".into())))
            .and_then(|s| {
                let status = ExitStatus {
                    success: s.success(),
                    code: s.code(),
                };
                match serde_json::to_string(&status)
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
                }
            });

        let stream = lines(outbuf)
            .select(lines(errbuf))
            .map(|s| Ok(s.into_bytes()))
            .map_err(|e| Error::with_chain(e, ErrorKind::Msg("Command execution failed".into())))
            .forward(tx1.sink_map_err(|e| Error::with_chain(e, "Could not forward command output to Body")))
            .join(status)
            // @todo We should repatriate these errors somehow
            .map(|_| ())
            .map_err(|_| ());

        handle.spawn(stream);

        Box::new(future::ok(Message::WithBody(ResponseResult::Ok(Response::Null), body)))
    }
}
