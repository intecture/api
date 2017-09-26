// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use regex::Regex;
use std::fs;
use std::io::Read;

pub fn version() -> Result<(String, u32, u32, u32)> {
    let mut fh = fs::File::open("/etc/redhat-release").chain_err(|| ErrorKind::SystemFile("/etc/redhat-release"))?;
    let mut fc = String::new();
    fh.read_to_string(&mut fc).unwrap();

    let regex = Regex::new(r"release ([0-9]+)(?:\.([0-9]+)(?:\.([0-9]+))?)?").unwrap();
    if let Some(cap) = regex.captures(&fc) {
        let version_maj = cap.get(1).unwrap().as_str()
                             .parse().chain_err(|| ErrorKind::SystemFileOutput("/etc/redhat-release"))?;
        let version_min = match cap.get(2) {
            Some(v) => v.as_str().parse().chain_err(|| ErrorKind::SystemFileOutput("/etc/redhat-release"))?,
            None => 0,
        };
        let version_patch = match cap.get(3) {
            Some(v) => v.as_str().parse().chain_err(|| ErrorKind::SystemFileOutput("/etc/redhat-release"))?,
            None => 0,
        };
        let version_str = format!("{}.{}.{}", version_maj, version_min, version_patch);
        Ok((version_str, version_maj, version_min, version_patch))
    } else {
        Err(ErrorKind::SystemFileOutput("/etc/redhat-release").into())
    }
}
