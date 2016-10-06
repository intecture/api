// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Parser for Intecture data files.

macro_rules! want_macro {
    ($t:expr, $n:ident, $isf:ident, $asf:ident) => {
        /// Helper that returns an Option<$t>.
        /// You can optionally pass a JSON pointer to retrieve a
        /// nested key.
        #[macro_export]
        macro_rules! $n {
            ($v:expr) => (if $v.$isf() {
                Some($v.$asf().unwrap())
            } else {
                None
            });

            ($v:expr => $p:expr) => (if let Some(v) = $v.pointer($p) {
                $n!(v)
            } else {
                None
            });
        }
    }
}

want_macro!("null", wantnull, is_null, as_null);
want_macro!("bool", wantbool, is_boolean, as_bool);
want_macro!("i64", wanti64, is_i64, as_i64);
want_macro!("u64", wantu64, is_u64, as_u64);
want_macro!("f64", wantf64, is_f64, as_f64);
want_macro!("string", wantstr, is_string, as_str);
want_macro!("array", wantarray, is_array, as_array);
want_macro!("object", wantobj, is_object, as_object);

macro_rules! need_macro {
    ($t:expr, $n:ident, $isf:ident, $asf:ident) => {
        /// Helper that returns a Result<$t>.
        /// You can optionally pass a JSON pointer to retrieve a
        /// nested key.
        #[macro_export]
        macro_rules! $n {
            ($v:expr) => (if $v.$isf() {
                Ok($v.$asf().unwrap())
            } else {
                Err(::error::Error::Generic(format!("Value is not $t")))
            });

            ($v:expr => $p:expr) => (if let Some(v) = $v.pointer($p) {
                $n!(v)
            } else {
                Err(::error::Error::Generic(format!("Could not find {} in data", $p)))
            });
        }
    }
}

need_macro!("null", neednull, is_null, as_null);
need_macro!("bool", needbool, is_boolean, as_bool);
need_macro!("i64", needi64, is_i64, as_i64);
need_macro!("u64", needu64, is_u64, as_u64);
need_macro!("f64", needf64, is_f64, as_f64);
need_macro!("string", needstr, is_string, as_str);
need_macro!("array", needarray, is_array, as_array);
need_macro!("object", needobj, is_object, as_object);

mod condition;
pub mod ffi;

use error::{Error, Result};
use serde_json::{self, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Parser for Intecture data files.
pub struct DataParser;

impl DataParser {
    /// Open a new file and recursively parse its contents.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Value> {
        let file = try!(DataFile::new(path));
        Ok(try!(file.merge(Value::Null)))
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

    fn merge(self, mut last_value: Value) -> Result<Value> {
        for dep in try!(self.dependencies()) {
            last_value = try!(dep.merge(last_value));
        }

        let lv_clone = last_value.clone();
        Ok(try!(Self::merge_values(self.v, last_value, &lv_clone)))
    }

    fn merge_values(into: Value, mut from: Value, parent_from: &Value) -> Result<Value> {
        match into {
            Value::Null |
            Value::Bool(_) |
            Value::I64(_) |
            Value::U64(_) |
            Value::F64(_) |
            Value::String(_) => Ok(into),
            Value::Array(mut a) => {
                if from.is_array() {
                    a.append(from.as_array_mut().unwrap());
                } else {
                    a.push(from);
                }

                let mut b = Vec::new();

                for v in a {
                    b.push(try!(Self::merge_values(v, Value::Null, parent_from)));
                }

                Ok(Value::Array(b))
            },
            Value::Object(o) => {
                let mut new: BTreeMap<String, Value> = BTreeMap::new();

                for (mut key, mut value) in o {
                    if key.ends_with("?") || key.ends_with("?!") {
                        if key.pop().unwrap() == '!' {
                            key.pop();
                            key.push('!');
                        }

                        value = try!(Self::query_value(&parent_from, value)).unwrap_or(Value::Null);
                    }

                    if key.ends_with("!") {
                        key.pop();
                    }
                    else if let Some(o1) = from.find(&key) {
                        value = try!(Self::merge_values(value, o1.clone(), &parent_from));
                    }

                    new.insert(key, value);
                }

                // Insert any missing values
                if let Some(o1) = from.as_object() {
                    for (key, value) in o1 {
                        if !new.contains_key(key) {
                            new.insert(key.clone(), value.clone());
                        }
                    }
                }

                Ok(Value::Object(new))
            }
        }
    }

    fn query_value(data: &Value, value: Value) -> Result<Option<Value>> {
        match value {
            Value::Array(a) => {
                for opt in a {
                    if let Some(v) = try!(Self::query_value(data, opt)) {
                        return Ok(Some(v));
                    }
                }
            },
            Value::Object(mut o) => {
                if let Some(v) = o.remove("_") {
                    if let Some(q) = o.get("?") {
                        match *q {
                            Value::String(ref s) => {
                                if try!(condition::eval(data, s)) {
                                    return Ok(Some(v));
                                }
                            },
                            _ => return Err(Error::Generic("Query must be string".into())),
                        };
                    }

                    return Ok(Some(v));
                }
            },
            _ => return Ok(Some(value)),
        }

        Ok(None)
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
                \"c?\": [
                    {{
                        \"_\": [ 6, 7 ],
                        \"?\": \"/variable = false\"
                    }},
                    {{
                        \"_\": [ 8, 9 ]
                    }}
                ],
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
                \"c\": [ 6, 7 ],
                \"d\": [ 123, 987, 999 ]
            }},
            \"variable\": {{
                \"one\": true,
                \"two\": false
            }}
        }}", &path.to_str().unwrap())).unwrap()
    }
}
