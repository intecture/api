// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use hostname::get_hostname;
use regex::Regex;
use std::process;
use telemetry::FsMount;

pub fn hostname() -> Result<String> {
    match get_hostname() {
        Some(name) => Ok(name),
        None => Err("Could not determine hostname".into()),
    }
}

pub enum FsFieldOrder {
    Filesystem,
    Size,
    Used,
    Available,
    Capacity,
    Mount,
    Blank,
}

// pub fn fs() -> Result<Vec<FsMount>> {
//     self::parse_fs(&[
//         self::FsFieldOrder::Filesystem,
//         self::FsFieldOrder::Size,
//         self::FsFieldOrder::Used,
//         self::FsFieldOrder::Available,
//         self::FsFieldOrder::Capacity,
//         self::FsFieldOrder::Mount,
//     ])
// }

pub fn parse_fs(fields: &[FsFieldOrder]) -> Result<Vec<FsMount>> {
    let mount_out = process::Command::new("df")
                                     .arg("-Pk")
                                     .output()
                                     .chain_err(|| ErrorKind::SystemCommand("sysctl"))?;
    let mount = String::from_utf8(mount_out.stdout).chain_err(|| ErrorKind::SystemCommandOutput("sysctl"))?;

    let mut pattern = "(?m)^".to_string();

    for field in fields {
        match *field {
            FsFieldOrder::Filesystem => pattern.push_str("(?P<fs>.+?)"),
            FsFieldOrder::Size => pattern.push_str("(?P<size>[0-9]+)"),
            FsFieldOrder::Used => pattern.push_str("(?P<used>[0-9]+)"),
            FsFieldOrder::Available => pattern.push_str("(?P<available>[0-9]+)"),
            FsFieldOrder::Capacity => pattern.push_str("(?P<capacity>[0-9]{1,3})%"),
            FsFieldOrder::Mount => pattern.push_str("(?P<mount>/.*)"),
            FsFieldOrder::Blank => pattern.push_str(r"[^\s]+"),
        }

        pattern.push_str(r"[\s]*");
    }

    pattern.push_str("$");

    let regex = Regex::new(&pattern).unwrap();
    let mut fs = vec!();

    let lines: Vec<&str> = mount.lines().collect();
    for line in lines {
        if let Some(cap) = regex.captures(line) {
            fs.push(FsMount {
                filesystem: cap.name("fs").unwrap().as_str().to_string(),
                mountpoint: cap.name("mount").unwrap().as_str().to_string(),
                size: cap.name("size").unwrap().as_str().parse::<u64>()
                        .chain_err(|| format!("could not discern {} from sysctl output", "size of mount"))?,
                used: cap.name("used").unwrap().as_str().parse::<u64>()
                        .chain_err(|| format!("could not discern {} from sysctl output", "used space"))?,
                available: cap.name("available").unwrap().as_str().parse::<u64>()
                        .chain_err(|| format!("could not discern {} from sysctl output", "available space"))?,
                capacity: cap.name("capacity").unwrap().as_str().parse::<f32>()
                        .chain_err(|| format!("could not discern {} from sysctl output", "mount capacity"))? / 100f32,
            });
        }
    };

    Ok(fs)
}
