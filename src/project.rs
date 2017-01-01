// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use zdaemon::ConfigFile;

#[repr(C)]
#[derive(Clone, Debug, PartialEq, RustcDecodable, RustcEncodable)]
/// The payload's programming language.
pub enum Language {
    C,
    Php,
    Rust,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct ProjectConfig {
    pub language: Language,
    pub auth_server: String,
    pub auth_api_port: u32,
    pub auth_update_port: u32,
}

impl ConfigFile for ProjectConfig {}
