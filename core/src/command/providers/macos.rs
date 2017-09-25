// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use ExecutableProvider;
use host::*;
use telemetry::{Telemetry, TelemetryProvider};

pub struct Macos<'a> {
    host: &'a Host,
}

#[derive(Serialize, Deserialize)]
pub enum RemoteProvider {
    Available,
    Load
}

impl <'de>ExecutableProvider<'de> for RemoteProvider {
    fn exec(&self, host: &Host) -> Result<()> {
        match *self {
            RemoteProvider::Available => {
                Macos::available(host);
                Ok(())
            }
            RemoteProvider::Load => {
                let p = Macos { host };
                let _ = p.load();
                Ok(())
            }
        }
    }
}

impl <'a>TelemetryProvider<'a> for Macos<'a> {
    fn available(host: &Host) -> bool {
        if host.is_local() {
            cfg!(macos)
        } else {
            unimplemented!();
            // let r = RemoteProvider::Available;
            // self.host.send(r).chain_err(|| ErrorKind::RemoteProvider("Telemetry", "available"))?;
            // let t: Telemetry = self.host.recv()?;
            // Ok(t)
        }
    }

    fn try_new(host: &'a Host) -> Option<Macos<'a>> {
        if Self::available(host) {
            Some(Macos { host })
        } else {
            None
        }
    }

    fn load(&self) -> Result<Telemetry> {
        if self.host.is_local() {

        } else {
            // let r = RemoteProvider::Load;
            // self.host.send(r).chain_err(|| ErrorKind::RemoteProvider("Telemetry", "load"))?;
            // let t: Telemetry = self.host.recv()?;
            // Ok(t)
        }

        unimplemented!();
    }
}
