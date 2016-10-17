// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! Parser for Intecture data files.

#[macro_use]
mod macros;
mod condition;

use error::{Error, Result};
use serde_json::{self, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn open<P: AsRef<Path>>(path: P) -> Result<Value> {
    let mut p = PathBuf::from("data");
    p.push(path);
    open_raw(&p)
}

fn open_raw<P: AsRef<Path>>(path: P) -> Result<Value> {
    let mut fh = try!(fs::File::open(path.as_ref()));
    let data: Value = try!(serde_json::from_reader(&mut fh));

    if !data.is_object() {
        Err(Error::Generic("Value is not an object".into()))
    } else {
        Ok(data)
    }
}

pub fn merge(mut me: Value, mut last_value: Value) -> Result<Value> {
    for dep in try!(dependencies(&mut me)) {
        last_value = try!(merge(dep, last_value));
    }

    let lv_clone = last_value.clone();
    Ok(try!(merge_values(me, last_value, &lv_clone)))
}

fn dependencies(me: &mut Value) -> Result<Vec<Value>> {
    let mut deps = Vec::new();
    let mut payloads: Vec<String> = Vec::new();

    if let Some(inc) = me.find("_include") {
        if !inc.is_array() {
            return Err(Error::Generic("Value of `_include` is not an array".into()));
        }

        // Loop in reverse order to get lowest importance first
        for i in inc.as_array().unwrap().iter().rev() {
            if let Some(s) = i.as_str() {
                if s.starts_with("payload:") {
                    let (_, payload) = s.split_at(8);
                    let payload = payload.trim();
                    let parts: Vec<&str> = payload.split("::").collect();

                    let mut buf = PathBuf::from("payloads");
                    buf.push(try!(parts.get(0).ok_or(Error::Generic("Empty payload in `_include`".into()))));
                    buf.push("data");
                    buf.push(parts.get(1).unwrap_or(&"default"));
                    buf.set_extension("json");

                    deps.push(try!(open_raw(&buf)));
                    payloads.insert(0, payload.into());
                } else {
                    deps.push(try!(open(s)));
                }
            } else {
                return Err(Error::Generic("Non-string value in `_include`".into()));
            }
        }
    }

    if me.is_object() && !payloads.is_empty() {
        me.as_object_mut().unwrap().insert("_payloads".into(), serde_json::to_value(payloads));
    }

    Ok(deps)
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
                b.push(try!(merge_values(v, Value::Null, parent_from)));
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

                    value = try!(query_value(&parent_from, value)).unwrap_or(Value::Null);
                }

                if key.ends_with("!") {
                    key.pop();
                }
                else if let Some(o1) = from.find(&key) {
                    value = try!(merge_values(value, o1.clone(), &parent_from));
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
                if let Some(v) = try!(query_value(data, opt)) {
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

#[cfg(test)]
mod tests {
    use serde_json::{self, Value};
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_parser() {
        let tempdir = TempDir::new("parser_test").unwrap();
        let mut path = tempdir.path().to_owned();

        path.push("data");
        fs::create_dir(&path).unwrap();
        path.pop();
        path.push("payloads/payload/data");

        fs::create_dir_all(&path).unwrap();
        path.pop();
        path.pop();
        path.pop();

        let expected_value = create_data(&mut path);

        path.push("data/top.json");
        let value = open(&path).unwrap();
        let value = merge(value, Value::Null).unwrap();

        assert_eq!(value, expected_value);
    }

    fn create_data(path: &mut PathBuf) -> Value {
        path.push("data/middle.json");
        let mut fh = fs::File::create(&path).unwrap();
        path.pop();
        path.pop();
        fh.write_all(format!("{{
            \"payload\": {{
                \"b!\": [ 3, 4 ],
                \"d\": [ 987 ]
            }},
            \"variable\": false,
            \"d\": 4,
            \"_include\": [
                \"payload: {}/payloads/payload::default\"
            ]
        }}", path.to_str().unwrap()).as_bytes()).unwrap();

        path.push("data/bottom.json");
        let mut fh = fs::File::create(&path).unwrap();
        path.pop();
        path.pop();
        fh.write_all(b"{
            \"moo\": \"cow\",
            \"payload\": {
                \"b\": [ 5 ],
                \"d\": [ 999 ]
            }
        }").unwrap();

        path.push("payloads/payload/data/default.json");
        let mut fh = fs::File::create(&path).unwrap();
        path.pop();
        path.pop();
        path.pop();
        path.pop();
        fh.write_all(b"{
            \"pvalue\": \"payload\"
        }").unwrap();

        path.push("data/top.json");
        let mut fh = fs::File::create(&path).unwrap();
        path.pop();
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
                \"{}/data/middle.json\",
                \"{0}/data/bottom.json\"
            ]
        }}", path.to_str().unwrap()).as_bytes()).unwrap();

        serde_json::from_str(&format!("{{
            \"_include\": [
                \"{}/data/middle.json\",
                \"{0}/data/bottom.json\",
                \"payload: {0}/payloads/payload::default\"
            ],
            \"_payloads\": [
                \"{0}/payloads/payload::default\"
            ],
            \"a\": 1,
            \"d\": 4,
            \"moo\": \"cow\",
            \"payload\": {{
                \"b\": [ 1, 2, 3, 4 ],
                \"c\": [ 6, 7 ],
                \"d\": [ 123, 987, 999 ]
            }},
            \"pvalue\": \"payload\",
            \"variable\": {{
                \"one\": true,
                \"two\": false
            }}
        }}", path.to_str().unwrap())).unwrap()
    }
}
