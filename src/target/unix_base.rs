// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use error::{Error, Result};
use file::FileOwner;
use host::telemetry::Netif;
use regex::Regex;
use std::{process, str};
use std::path::Path;
use super::default_base as default;

pub fn file_get_owner<P: AsRef<Path>>(path: P) -> Result<FileOwner> {
    Ok(FileOwner {
        user_name: try!(default::file_stat(path.as_ref(), vec!["-f", "%Su"])),
        user_uid: try!(default::file_stat(path.as_ref(), vec!["-f", "%u"])).parse::<u64>().unwrap(),
        group_name: try!(default::file_stat(path.as_ref(), vec!["-f", "%Sg"])),
        group_gid: try!(default::file_stat(path.as_ref(), vec!["-f", "%g"])).parse::<u64>().unwrap()
    })
}

pub fn file_get_mode<P: AsRef<Path>>(path: P) -> Result<u16> {
    Ok(try!(default::file_stat(path, vec!["-f", "%Lp"])).parse::<u16>().unwrap())
}

pub fn version() -> Result<(String, u32, u32)> {
    let output = try!(process::Command::new("uname").arg("-r").output());
    let version_str = str::from_utf8(&output.stdout).unwrap().trim();
    let regex = Regex::new(r"([0-9]+)\.([0-9]+)-[A-Z]+").unwrap();
    if let Some(cap) = regex.captures(version_str) {
        let version_maj = cap.get(1).unwrap().as_str().parse()?;
        let version_min = cap.get(2).unwrap().as_str().parse()?;
        Ok((version_str.into(), version_maj, version_min))
    } else {
        Err(Error::Generic("Could not match OS version".into()))
    }
}

pub fn get_sysctl_item(item: &str) -> Result<String> {
    // XXX This result should be cached
    let sysctl_out = try!(process::Command::new("sysctl").arg("-a").output());
    let sysctl = String::from_utf8(sysctl_out.stdout).unwrap();

    let exp = format!("{}: (.+)", item);
    let regex = Regex::new(&exp).unwrap();

    if let Some(cap) = regex.captures(&sysctl) {
        Ok(cap.get(1).unwrap().as_str().into())
    } else {
        Err(Error::Generic("Could not match sysctl item".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sysctl_item_err() {
        assert!(super::get_sysctl_item("moo").is_err());
    }
}
