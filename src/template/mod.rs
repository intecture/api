// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! A File primitive wrapper for managing file templates.
//!
//! # Examples
//!
//! ```no_run
//! # use inapi::{HashBuilder, Template};
//! let template = Template::new("/path/to/template").unwrap();
//! let data = HashBuilder::new().insert_string("key", "value");
//! let file = template.render_to_file(data).unwrap();
//! ```

pub mod ffi;

use error::Result;
use error::Error;
use rustache::{HashBuilder, render_file};
use std::convert::Into;
use std::fs;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tempfile::tempfile;

/// Container for rendering and uploading templates.
pub struct Template {
    path: PathBuf,
}

impl Template {
    /// Create a new File struct.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Template> {
        if !path.as_ref().exists() {
            return Err(Error::Generic("Template path does not exist".into()));
        }

        Ok(Template {
            path: path.as_ref().to_owned(),
        })
    }

    /// Render template to file.
    pub fn render_to_file<'a>(&self, data: HashBuilder<'a>) -> Result<fs::File> {
        let stream = try!(render_file(self.path.to_str().expect("Invalid template path"), data));
        let mut fh = try!(tempfile());
        try!(fh.write_all(stream.as_slice()));
        try!(fh.seek(SeekFrom::Start(0)));
        Ok(fh)
    }
}

#[cfg(test)]
mod tests {
    use rustache::HashBuilder;
    use std::fs;
    use std::io::{Read, Write};
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_new() {
        let tempdir = TempDir::new("template_test_render").unwrap();
        let template_path = format!("{}/template", tempdir.path().to_str().unwrap());

        assert!(Template::new(&template_path).is_err());

        fs::File::create(&template_path).unwrap();
        assert!(Template::new(&template_path).is_ok());
    }

    #[test]
    fn test_render_to_file() {
        let tempdir = TempDir::new("template_test_render").unwrap();
        let template_path = format!("{}/template", tempdir.path().to_str().unwrap());

        let template_str = "Hello, {{name}}!".to_string();
        let data = HashBuilder::new().insert_string("name", "Jasper Beardly");

        let mut fh = fs::File::create(&template_path).unwrap();
        fh.write_all(template_str.as_bytes()).unwrap();

        let template = Template::new(&template_path).unwrap();
        let mut fh = template.render_to_file(data).unwrap();
        let mut content = String::new();
        fh.read_to_string(&mut content).unwrap();
        assert_eq!(content, "Hello, Jasper Beardly!");
    }
}
