// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate regex;

#[cfg(all(feature = "local-run", not(feature = "remote-run")))]
use regex::Regex;
#[cfg(all(feature = "local-run", not(feature = "remote-run")))]
use std::{env, fs};
#[cfg(all(feature = "local-run", not(feature = "remote-run")))]
use std::io::Read;

#[cfg(all(feature = "local-run", feature = "remote-run"))]
fn main() {
    panic!("Mutually exclusive features `local-run` and `remote-run`. You must only enable one.");
}

#[cfg(all(not(feature = "local-run"), not(feature = "remote-run")))]
fn main() {
    panic!("Missing feature `local-run` or `remote-run`. You must enable one.");
}

#[cfg(all(feature = "local-run", not(feature = "remote-run")))]
fn main() {
    let os = fingerprint_os();
    println!("cargo:rustc-cfg=in_os_family=\"{}\"", os.family);
    println!("cargo:rustc-cfg=in_os_platform=\"{}\"", os.platform);
}

#[cfg(all(feature = "remote-run", not(feature = "local-run")))]
fn main() {
	println!("cargo:rustc-link-search=native=/usr/local/lib");
}

#[cfg(all(feature = "local-run", not(feature = "remote-run")))]
struct Os {
    family: String,
    platform: String,
}

/// Fingerprint the OS more granularly than Rust to ensure we build
/// the right modules.
#[cfg(all(feature = "local-run", not(feature = "remote-run")))]
fn fingerprint_os() -> Os {
    if cfg!(target_os = "linux") {
        // Red Hat family
        if let Ok(mut fh) = fs::File::open("/etc/redhat-release") {
            let mut fc = String::new();
            fh.read_to_string(&mut fc).unwrap();

            let regex = Regex::new(r"^([A-Za-z ]+?)(?: AS)? release").unwrap();
            if let Some(cap) = regex.captures(&fc) {
                let platform = match cap.at(1).unwrap() {
                    "Red Hat Enterprise Linux" => "rhel".to_string(),
                    _ => cap.at(1).unwrap().to_string().to_lowercase(),
                };

                return Os {
                    family: "redhat".to_string(),
                    platform: platform,
                };
            }
        }
        // Ubuntu
        else if let Ok(_) = fs::metadata("/etc/lsb-release") {
            return Os {
                family: "debian".to_string(),
                platform: "ubuntu".to_string(),
            };
        }
        // Debian
        else if let Ok(_) = fs::metadata("/etc/debian_version") {
            return Os {
                family: "debian".to_string(),
                platform: "debian".to_string(),
            };
        }

        panic!("Unknown Linux distro");
    } else if cfg!(any(target_os = "macos", target_os = "freebsd")) {
        return Os {
            family: env::consts::FAMILY.to_string(),
            platform: env::consts::OS.to_string(),
        };
    }

    panic!("Unsupported distro");
}
