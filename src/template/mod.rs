// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The primitive for opening and rendering templates.
//!
//! # Examples
//!
//! ```no_run
//! # use inapi::{MapBuilder, Template};
//! let template = Template::new("/path/to/template").unwrap();
//! let data = MapBuilder::new().insert_str("key", "value").build();
//! let rendered_file = template.render_data(&data).unwrap();
//! ```
//!
//! To upload the rendered file to your host, you can pass it
//! straight into the File primitive:
//!
//! ```no_run
//! # use inapi::{File, Host, MapBuilder, Template};
//! # let template = Template::new("/path/to/template").unwrap();
//! # let data = MapBuilder::new().insert_str("key", "value").build();
//! # let rendered_file = template.render_data(&data).unwrap();
#![cfg_attr(feature = "local-run", doc = "let mut host = Host::local(None);")]
#![cfg_attr(feature = "remote-run", doc = "let mut host = Host::connect(\"data/nodes/mynode.json\").unwrap();")]
//!
//! let file = File::new(&mut host, "/path/to/remote/file").unwrap();
//! file.upload_file(&mut host, rendered_file, None).unwrap();
//! ```

pub mod ffi;

use error::Result;
use error::Error;
use mustache;
use rustc_serialize::Encodable;
use std::convert::Into;
use std::fs;
use std::io::{Seek, SeekFrom};
use std::path::Path;
use tempfile::tempfile;

/// Container for rendering and uploading templates.
pub struct Template {
    inner: mustache::Template,
}

impl Template {
    /// Create a new File struct.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Template> {
        if !path.as_ref().exists() {
            return Err(Error::Generic("Template path does not exist".into()));
        }

        Ok(Template {
            inner: try!(mustache::compile_path(path.as_ref())),
        })
    }

    /// Render template to file using generic Encodable data.
    pub fn render<T: Encodable>(&self, data: &T) -> Result<fs::File> {
        let mut fh = try!(tempfile());
        try!(self.inner.render(&mut fh, data));

        // Reset cursor to beginning of file for reading
        try!(fh.seek(SeekFrom::Start(0)));
        Ok(fh)
    }

    /// Render template to file using a Data instance.
    pub fn render_data(&self, data: &mustache::Data) -> Result<fs::File> {
        let mut fh = try!(tempfile());
        self.inner.render_data(&mut fh, data);

        // Reset cursor to beginning of file for reading
        try!(fh.seek(SeekFrom::Start(0)));
        Ok(fh)
    }
}

#[cfg(test)]
mod tests {
    use mustache::MapBuilder;
    use std::fs;
    use std::io::{Read, Write};
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_new() {
        let tempdir = TempDir::new("template_test_render").unwrap();
        let template_path = format!("{}/template.mustache", tempdir.path().to_str().unwrap());

        assert!(Template::new(&template_path).is_err());

        fs::File::create(&template_path).unwrap();
        Template::new(&template_path).unwrap();
        assert!(Template::new(&template_path).is_ok());
    }

    #[test]
    fn test_render() {
        let tempdir = TempDir::new("template_test_render").unwrap();
        let template_path = format!("{}/template.mustache", tempdir.path().to_str().unwrap());

        let template_str = "Hello, {{name}}!".to_string();
        let data = TestData {
            name: "Jasper Beardly"
        };

        let mut fh = fs::File::create(&template_path).unwrap();
        fh.write_all(template_str.as_bytes()).unwrap();

        let template = Template::new(&template_path).unwrap();
        let mut fh = template.render(&data).unwrap();
        let mut content = String::new();
        fh.read_to_string(&mut content).unwrap();
        assert_eq!(content, "Hello, Jasper Beardly!");
    }

    #[test]
    fn test_render_data_to_file() {
        let tempdir = TempDir::new("template_test_render").unwrap();
        let template_path = format!("{}/template.mustache", tempdir.path().to_str().unwrap());

        let template_str = "Hello, {{name}}!".to_string();
        let data = MapBuilder::new().insert_str("name", "Jasper Beardly").build();

        let mut fh = fs::File::create(&template_path).unwrap();
        fh.write_all(template_str.as_bytes()).unwrap();

        let template = Template::new(&template_path).unwrap();
        let mut fh = template.render_data(&data).unwrap();
        let mut content = String::new();
        fh.read_to_string(&mut content).unwrap();
        assert_eq!(content, "Hello, Jasper Beardly!");
    }

    #[derive(RustcEncodable)]
    struct TestData {
        name: &'static str,
    }
}
