// Copyright 2015-2017 Intecture Developers.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use errors::*;
use regex::Regex;
use std::{fs, process, str};
use std::io::Read;

pub fn cpu_vendor() -> Result<String> {
    get_cpu_item("vendor_id")
}

pub fn cpu_brand_string() -> Result<String> {
    get_cpu_item("model name")
}

pub fn cpu_cores() -> Result<u32> {
    Ok(get_cpu_item("cpu cores")?
        .parse::<u32>()
        .chain_err(|| ErrorKind::InvalidTelemetryKey {
            cmd: "/proc/cpuinfo",
            key: "cpu cores".into()
        })?)
}

fn get_cpu_item(item: &str) -> Result<String> {
    // @todo Cache file content
    let mut fh = fs::File::open("/proc/cpuinfo").chain_err(|| ErrorKind::SystemFile("/proc/cpuinfo"))?;
    let mut cpuinfo = String::new();
    fh.read_to_string(&mut cpuinfo).chain_err(|| ErrorKind::SystemFileOutput("/proc/cpuinfo"))?;;

    let pattern = format!(r"(?m)^{}\s+: (.+)$", item);
    let regex = Regex::new(&pattern).unwrap();
    let capture = regex.captures(&cpuinfo);

    if let Some(cap) = capture {
        Ok(cap.get(1).unwrap().as_str().to_string())
    } else {
        Err(ErrorKind::InvalidTelemetryKey { cmd: "/proc/cpuinfo", key: item.into() }.into())
    }
}

pub fn memory() -> Result<u64> {
    let output = process::Command::new("free").arg("-b").output().chain_err(|| ErrorKind::SystemCommand("free"))?;
    let regex = Regex::new(r"(?m)^Mem:\s+([0-9]+)").chain_err(|| "could not create new Regex instance")?;
    let capture = regex.captures(str::from_utf8(&output.stdout).chain_err(|| ErrorKind::SystemCommandOutput("free"))?.trim());

    if let Some(cap) = capture {
        Ok(cap.get(1).unwrap().as_str().parse::<u64>().chain_err(|| ErrorKind::SystemFileOutput("/etc/redhat-release"))?)
    } else {
        Err(ErrorKind::SystemCommandOutput("free").into())
    }
}
