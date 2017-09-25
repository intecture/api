// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Host primitive.

use errors::*;
use telemetry::{self, Telemetry};

pub struct Host {
    telemetry: Option<Telemetry>,
    sock: Option<u32>,
}

impl Host {
    /// Create a new Host targeting the local machine.
    pub fn local() -> Result<Host> {
        let mut host = Host {
            telemetry: None,
            sock: None
        };

        host.telemetry = Some(telemetry::load(&host)?);

        Ok(host)
    }

    /// Create a new Host connected to addr.
    pub fn connect(_addr: &str) -> Result<Host> {
        unimplemented!();
        // @todo Connect to sock...

        // let mut host = Host {
        //     telemetry: None,
        //     sock: None
        // };
        //
        // let provider = telemetry::factory(&host)?;
        // host.telemetry = Some(provider.load()?);
        //
        // Ok(host)
    }

    /// Retrieve Telemetry
    pub fn telemetry(&self) -> &Telemetry {
        self.telemetry.as_ref().unwrap()
    }

    pub fn is_local(&self) -> bool {
        self.sock.is_none()
    }
}
