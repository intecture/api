// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Manages the connection between the API and a server.

pub mod local;
pub mod remote;

use errors::*;
use futures::Future;
use remote::{Request, Response};
use std::io;
use telemetry::Telemetry;
use tokio_core::reactor::Handle;
use tokio_proto::streaming::{Body, Message};

/// Trait for local and remote host types.
pub trait Host: Clone {
    /// Get `Telemetry` for this host.
    fn telemetry(&self) -> &Telemetry;
    /// Get `Handle` to Tokio reactor.
    fn handle(&self) -> &Handle;
    #[doc(hidden)]
    fn request(&self, request: Request) ->
        Box<Future<Item = Message<Response, Body<Vec<u8>, io::Error>>, Error = Error>>
    {
        self.request_msg(Message::WithoutBody(request))
    }
    #[doc(hidden)]
    fn request_msg(&self, Message<Request, Body<Vec<u8>, io::Error>>) ->
        Box<Future<Item = Message<Response, Body<Vec<u8>, io::Error>>, Error = Error>>;
}
