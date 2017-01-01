// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use error::{Error, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

const BIN_PATHS: [&'static str; 6] = [
    "/usr/local/bin",
    "/usr/bin",
    "/bin",
    "/sbin",
    "/usr/sbin",
    "/usr/local/sbin"
];

lazy_static! {
    static ref RESOLVER: Mutex<BinResolver> = Mutex::new(BinResolver::new());
}

pub struct BinResolver {
    cache: HashMap<String, PathBuf>,
}

impl BinResolver {
    fn new() -> BinResolver {
        BinResolver {
            cache: HashMap::new(),
        }
    }

    pub fn resolve(bin: &str) -> Result<PathBuf> {
        let mut br = RESOLVER.lock().unwrap();

        if !br.cache.contains_key(bin) {
            for path in BIN_PATHS.into_iter() {
                let mut buf = PathBuf::from(path);

                if buf.is_dir() {
                    buf.push(bin);

                    if buf.is_file() {
                        br.cache.insert(bin.into(), buf);
                        break;
                    }
                }
            }
        }

        Ok(br.cache.get(bin)
                   .ok_or(Error::Generic(format!("No paths contained the requested binary: {}", bin)))?
                   .to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // XXX Need to mock FS before we can test this
    // #[test]
    // fn test_resolve_ok() {
    //
    // }

    #[test]
    fn test_resolve_fail() {
        // XXX Without mocking the FS, this could potentially return
        // Ok where 'i_am_not_a_bin_script' is a bin script on the
        // target platform.
        assert!(BinResolver::resolve("i_am_not_a_bin_script").is_err());
    }
}
