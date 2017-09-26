// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use regex::Regex;
use std::{process, str};
// use std::path::Path;
// use super::default;

// pub fn file_get_owner<P: AsRef<Path>>(path: P) -> Result<FileOwner> {
//     Ok(FileOwner {
//         user_name: default::file_stat(path.as_ref(), vec!["-f", "%Su"])?,
//         user_uid: default::file_stat(path.as_ref(), vec!["-f", "%u"])?.parse::<u64>().unwrap(),
//         group_name: default::file_stat(path.as_ref(), vec!["-f", "%Sg"])?,
//         group_gid: default::file_stat(path.as_ref(), vec!["-f", "%g"])?.parse::<u64>().unwrap()
//     })
// }

// pub fn file_get_mode<P: AsRef<Path>>(path: P) -> Result<u16> {
//     Ok(default::file_stat(path, vec!["-f", "%Lp"])?.parse::<u16>().unwrap())
// }

pub fn version() -> Result<(String, u32, u32)> {
    let output = process::Command::new("uname")
                                  .arg("-r")
                                  .output()
                                  .chain_err(|| ErrorKind::SystemCommand("uname"))?;
    let version_str = str::from_utf8(&output.stdout).unwrap().trim();
    let regex = Regex::new(r"([0-9]+)\.([0-9]+)-[A-Z]+").chain_err(|| "could not create new Regex instance")?;
    let errstr = format!("Expected OS version format `u32.u32`, got: '{}'", version_str);
    if let Some(cap) = regex.captures(version_str) {
        let version_maj = cap.get(1).unwrap().as_str().parse().chain_err(|| ErrorKind::SystemCommandOutput("uname"))?;
        let version_min = cap.get(2).unwrap().as_str().parse().chain_err(|| ErrorKind::SystemCommandOutput("uname"))?;
        Ok((version_str.into(), version_maj, version_min))
    } else {
        Err(errstr.into())
    }
}

pub fn get_sysctl_item(item: &str) -> Result<String> {
    // @todo Cache output of sysctl
    let sysctl_out = process::Command::new("sysctl")
                                      .arg("-a")
                                      .output()
                                      .chain_err(|| ErrorKind::SystemCommand("sysctl"))?;
    let sysctl = String::from_utf8(sysctl_out.stdout).chain_err(|| ErrorKind::SystemCommandOutput("sysctl"))?;

    let exp = format!("{}: (.+)", item);
    let regex = Regex::new(&exp).chain_err(|| "could not create new Regex instance")?;

    if let Some(cap) = regex.captures(&sysctl) {
        Ok(cap.get(1).unwrap().as_str().into())
    } else {
        Err(ErrorKind::InvalidTelemetryKey { cmd: "sysctl", key: item.into() }.into())
    }
}
