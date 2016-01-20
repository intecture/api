// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The primitive for managing files on a managed host.
//!
//! # Examples
//!
//! Initialise a new Host using your managed host's IP address and
//! port number:
//!
//! ```no_run
//! # use inapi::Host;
//! let mut host = Host::new();
#![cfg_attr(feature = "remote-run", doc = " host.connect(\"127.0.0.1\", 7101).unwrap();")]
//! ```
//!
//! Now ...
//!
//! ```no_run
//! # use inapi::{Host, File};
//! # let mut host = Host::new();
//! ```

// pub mod ffi;

use {Host, Target};

/// Container for operating on a file.
pub struct File {
    path: String,
    exists: bool,
}

// EXAMPLE
// let f = try!(File::new("/usr/local/bin/moo.sh"));
// println!("{}", f.exists());
// try!(f.upload("templates/moo.sh"));
// f.setMode(644);
// f.delete();

impl File {
    /// Create a new File struct.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::File;
    /// let file = File::new(&mut host, "/path/to/file");
    /// ```
    pub fn new(host: &mut Host, path: &str) -> Result<File> {
        Target::file_exists(path);

        Ok(File {
            path: remote_path.to_string(),
            exists: exists,
        })
    }

    pub fn exists();

    pub fn upload();

    pub fn delete();

    pub fn getMode() -> Result<u32>;

    pub fn setMode(&self, host: &mut Host, mode: u32) -> Result<()> {

    }
}

pub trait FileTarget {
    fn setMode(host: &mut Host, path: &str) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use Host;
    use super::*;

    #[test]
    fn test_() {

    }
}
