// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use std::env;
use super::{Result, Target, TargetInterface};
use target::{default_base as default, linux_base as linux, redhat_base as redhat};
use telemetry::{FsMount, Netif};

impl TargetInterface for Target {
    fn hostname() -> Result<String> {
        default::hostname()
    }

    fn arch() -> String {
        env::consts::ARCH.to_string()
    }

    fn family() -> String {
        "redhat".to_string()
    }

    fn platform() -> String {
        "redhat".to_string()
    }

    fn version() -> Result<String> {
        redhat::version()
    }

    fn memory() -> Result<u64> {
        linux::memory()
    }

    fn cpu_vendor() -> Result<String> {
        linux::cpu_vendor()
    }

    fn cpu_brand_string() -> Result<String> {
        linux::cpu_brand_string()
    }

    fn cpu_cores() -> Result<u32> {
        linux::cpu_cores()
    }

    fn fs() -> Result<Vec<FsMount>> {
        default::fs()
    }

    fn net() -> Result<Vec<Netif>> {
        linux::net()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_fs() {
        // XXX Not a proper test. Requires mocking.
        assert!(Target::fs().is_ok());
    }
}