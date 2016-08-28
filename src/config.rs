// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use zdaemon::ConfigFile;

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Config {
    pub language: String,
    pub artifact: String,
    pub auth_server: String,
}

impl ConfigFile for Config {}

#[cfg(all(test, feature = "remote-run"))]
impl Config {
    pub fn new(language: &str, artifact: &str, auth_server: &str) -> Config {
        Config {
            language: language.into(),
            artifact: artifact.into(),
            auth_server: auth_server.into(),
        }
    }
}
