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
use serde_json::{self, Value};
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
        try!(file.merge(Value::Null));
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
                v: data,
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

    fn merge(&mut self, mut last_value: Value) -> Result<()> {
        for mut dep in try!(self.dependencies()) {
            try!(dep.merge(last_value));
            last_value = dep.v;
        }

        Self::merge_values(&mut self.v, last_value);

        Ok(())
    }

    fn merge_values(into: &mut Value, mut from: Value) {
        match *into {
            Value::Null |
            Value::Bool(_) |
            Value::I64(_) |
            Value::U64(_) |
            Value::F64(_) |
            Value::String(_) => (),
            Value::Array(ref mut a) => {
                if from.is_array() {
                    a.append(from.as_array_mut().unwrap());
                } else {
                    a.push(from);
                }
            },
            Value::Object(ref mut o) => {
                let mut overwrite_keys = Vec::new();

                for (key, value) in o.iter_mut() {
                    if key.ends_with("!") {
                        // Cache key for later removal
                        let mut new_key = key.clone();
                        new_key.pop();
                        overwrite_keys.push(new_key);
                    }
                    else if let Some(o1) = from.find(key) {
                        Self::merge_values(value, o1.clone());
                    }
                }

                for key in overwrite_keys {
                    // Insert new key+value
                    let value = o.remove(&format!("{}!", key)).unwrap();
                    o.insert(key, value);
                }

                // Insert any missing values
                if let Some(o1) = from.as_object() {
                    for (key, value) in o1 {
                        if !o.contains_key(key) {
                            o.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{self, Value};
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_parser() {
        let tempdir = TempDir::new("parser_test").unwrap();
        let mut path = tempdir.path().to_owned();
        let expected_value = create_data(&mut path);

        path.push("top.json");
        let value = DataParser::open(&path).unwrap();

        assert_eq!(value, expected_value);
    }

    fn create_data(path: &mut PathBuf) -> Value {
        path.push("top.json");
        let mut fh = File::create(&path).unwrap();
        path.pop();
        fh.write_all(format!("{{
            \"a\": 1,
            \"payload\": {{
                \"b\": [ 1, 2 ],
                \"c!\": [ 1, 2 ],
                \"d\": [ 123 ]
            }},
            \"variable\": {{
                \"one!\": true,
                \"two\": false
            }},
            \"_include\": [
                \"{0}/middle.json\",
                \"{0}/bottom.json\"
            ]
        }}", &path.to_str().unwrap()).as_bytes()).unwrap();

        path.push("middle.json");
        let mut fh = File::create(&path).unwrap();
        path.pop();
        fh.write_all(b"{
            \"payload\": {
                \"b!\": [ 3, 4 ],
                \"c\": [
                    {
                        \"_\": [ 6, 7 ],
                        \"?\": \"telemetry/os/family=linux\"
                    },
                    {
                        \"_\": [ 8, 9 ],
                        \"?\": \"\"
                    }
                ],
                \"d\": [ 987 ]
            },
            \"variable\": false,
            \"d\": 4
        }").unwrap();

        path.push("bottom.json");
        let mut fh = File::create(&path).unwrap();
        path.pop();
        fh.write_all(b"{
            \"moo\": \"cow\",
            \"payload\": {
                \"b\": [ 5 ],
                \"d\": [ 999 ]
            }
        }").unwrap();

        serde_json::from_str(&format!("{{
            \"_include\": [
                \"{0}/middle.json\",
                \"{0}/bottom.json\"
            ],
            \"a\": 1,
            \"d\": 4,
            \"moo\": \"cow\",
            \"payload\": {{
                \"b\": [ 1, 2, 3, 4 ],
                \"c\": [ 1, 2 ],
                \"d\": [ 123, 987, 999 ]
            }},
            \"variable\": {{
                \"one\": true,
                \"two\": false
            }}
        }}", &path.to_str().unwrap())).unwrap()
    }
}
