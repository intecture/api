// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! A connection to the local machine.

use errors::*;
use futures::{future, Future};
use remote::{Executable, Request, Response, ResponseResult};
use std::io;
use std::sync::Arc;
use super::Host;
use telemetry::{self, Telemetry};
// use telemetry::Telemetry;
use tokio_core::reactor::Handle;
use tokio_proto::streaming::{Body, Message};

/// A `Host` type that talks directly to the local machine.
#[derive(Clone)]
pub struct Local {
    inner: Arc<Inner>,
    handle: Handle,
}

struct Inner {
    telemetry: Option<Telemetry>,
}

impl Local {
    /// Create a new `Host` targeting the local machine.
    pub fn new(handle: &Handle) -> Box<Future<Item = Local, Error = Error>> {
        let mut host = Local {
            inner: Arc::new(Inner { telemetry: None }),
            handle: handle.clone(),
        };

        Box::new(telemetry::Telemetry::load(&host)
            .chain_err(|| "Could not load telemetry for host")
            .map(|t| {
                Arc::get_mut(&mut host.inner).unwrap().telemetry = Some(t);
                host
            }))
    }
}

impl Host for Local {
    fn telemetry(&self) -> &Telemetry {
        self.inner.telemetry.as_ref().unwrap()
    }

    fn handle(&self) -> &Handle {
        &self.handle
    }

    #[doc(hidden)]
    fn request_msg(&self, msg: Message<Request, Body<Vec<u8>, io::Error>>) ->
        Box<Future<Item = Message<Response, Body<Vec<u8>, io::Error>>, Error = Error>>
    {
        Box::new(msg.into_inner()
           .exec(self)
           .and_then(|mut msg| {
               let body = msg.take_body();
               match msg.into_inner() {
                   ResponseResult::Ok(response) => if let Some(body) = body {
                       future::ok(Message::WithBody(response, body))
                   } else {
                       future::ok(Message::WithoutBody(response))
                   },
                   ResponseResult::Err(e) => future::err(e.into()),
               }
           }))
    }
}
