// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Parser for Intecture data files.
//!
//! # Examples
//!
//! ```no_run
//! ```

// pub mod ffi;

use error::{Error, Result};
use serde_json::{Map, self, Value};
use std::fs;
use std::path::Path;

/// Parser for Intecture data files.
pub struct DataParser;

impl DataParser {
    /// Open a new file and recursively parse its contents.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Value> {
        let mut file = try!(DataFile::new(path));

        for dep in try!(file.dependencies()) {
            try!(file.merge_files(dep));
        }

        Ok(file.into_inner())
    }
}

struct DataFile {
    v: Value,
}

impl DataFile {
    fn new<P: AsRef<Path>>(path: P) -> Result<DataFile> {
        let mut fh = try!(fs::File::open(path.as_ref()));
        let data: Value = try!(serde_json::from_reader(&mut fh));

        if !data.is_object() {
            Err(Error::Generic("Value is not an object".into()))
        } else {
            Ok(DataFile {
                v: data
            })
        }
    }

    fn into_inner(self) -> Value {
        self.v
    }

    fn dependencies(&self) -> Result<Vec<DataFile>> {
        let mut deps = Vec::new();

        if let Some(inc) = self.v.find("_include") {
            if !inc.is_array() {
                return Err(Error::Generic("Value of `_include` is not an array".into()));
            }

            // Loop in reverse order to get lowest importance first
            for i in inc.as_array().unwrap().iter().rev() {
                deps.push(try!(DataFile::new(i.as_str().unwrap())));
            }
        }

        Ok(deps)
    }

    fn merge_files(&mut self, mut child: DataFile) -> Result<()> {
        for dep in try!(child.dependencies()) {
            try!(child.merge_files(dep));
        }

        let mut obj = self.v.as_object_mut().unwrap();
        let child_obj = child.v.as_object().unwrap();

        // Merge values
        for (mut key, value) in obj.iter_mut() {
            Self::merge_values(&mut *key, value, child_obj);
        }

        // Insert any missing values
        for (key, value) in child_obj {
            if obj.contains_key(key) {
                obj.insert(key.clone(), value.clone());
            }
        }

        Ok(())
    }

    fn merge_values(key: &mut String, value: &mut Value, from: &Map<String, Value>) {
        let mut merge_flag = false;

        if key.starts_with("+") {
            key.remove(0);
            merge_flag = true;
        }

        if let Some(cval) = from.get(key) {
            if value != cval {
                // match
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
