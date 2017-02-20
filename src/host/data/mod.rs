// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
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
use serde_json::{self, Value, Map};
use std::fs;
use std::path::{Path, PathBuf};

#[doc(hidden)]
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

    if let Some(inc) = me.get("_include") {
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
                    buf.push(parts.get(1).unwrap_or(&"main"));
                    buf.set_extension("json");

                    if let Ok(d) = open_raw(&buf) {
                        deps.push(d);
                    }
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
        me["_payloads"] = json!(payloads);
    }

    Ok(deps)
}

fn merge_values(into: Value, mut from: Value, parent_from: &Value) -> Result<Value> {
    match into {
        Value::Null |
        Value::Bool(_) |
        Value::Number(_) |
        Value::String(_) => Ok(into),
        Value::Array(mut a) => {
            if from.is_array() {
                a.append(from.as_array_mut().unwrap());
            }
            else if !from.is_null() {
                a.push(from);
            }

            let mut b = Vec::new();

            for v in a {
                b.push(try!(merge_values(v, Value::Null, parent_from)));
            }

            Ok(Value::Array(b))
        },
        Value::Object(o) => {
            let mut obj = Map::new();

            for (mut key, mut value) in o {
                if key.ends_with("?") || key.ends_with("?!") {
                    if key.pop().unwrap() == '!' {
                        key.pop();
                        key.push('!');
                    }

                    value = try!(query_value(&parent_from, value)).unwrap_or(Value::Null);
                }

                let mut merge_val = Value::Null;

                if key.ends_with("!") {
                    key.pop();
                }
                else if let Some(o1) = from.get(&key) {
                    merge_val = o1.clone();
                }

                value = try!(merge_values(value, merge_val, &parent_from));

                obj.insert(key, value);
            }

            // Insert any missing values
            if let Some(o1) = from.as_object() {
                for (key, value) in o1 {
                    if !obj.contains_key(key) {
                        obj.insert(key.clone(), value.clone());
                    }
                }
            }

            Ok(Value::Object(obj))
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
                } else {
                    return Ok(Some(v));
                }
            }
        },
        _ => return Ok(Some(value)),
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
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
        let mut fh = fs::File::create(format!("{}/data/middle.json", path.display())).unwrap();
        let payload_path = format!("{}/payloads/payload::default", path.display());
        let payload = format!("payload: {}", payload_path);
        let mut data = json!({
            "payload": {
                "b!": [ 3, 4 ],
                "d": [ 987 ]
            },
            "variable": false,
            "d": 4,
            "_include": [
                payload
            ]
        });
        fh.write_all(data.to_string().as_bytes()).unwrap();

        fh = fs::File::create(format!("{}/data/bottom.json", path.display())).unwrap();
        data = json!({
            "moo": "cow",
            "payload": {
                "b": [ 5 ],
                "d": [ 999 ]
            }
        });
        fh.write_all(data.to_string().as_bytes()).unwrap();

        fh = fs::File::create(format!("{}/payloads/payload/data/default.json", path.display())).unwrap();
        data = json!({
            "pvalue": "payload"
        });
        fh.write_all(data.to_string().as_bytes()).unwrap();

        fh = fs::File::create(format!("{}/data/top.json", path.display())).unwrap();
        let middle = format!("{}/data/middle.json", path.display());
        let bottom = format!("{}/data/bottom.json", path.display());
        data = json!({
            "a": 1,
            "payload": {
                "b": [ 1, 2 ],
                "c?": [
                    {
                        "_": [ 6, 7 ],
                        "?": "/variable = false"
                    },
                    {
                        "_": [ 8, 9 ]
                    }
                ],
                "d": [ 123 ]
            },
            "variable": {
                "one!": true,
                "two": false
            },
            "_include": [ middle, bottom ]
        });
        fh.write_all(data.to_string().as_bytes()).unwrap();

        json!({
            "_include": [ middle, bottom, payload ],
            "_payloads": [ payload_path ],
            "a": 1,
            "d": 4,
            "moo": "cow",
            "payload": {
                "b": [ 1, 2, 3, 4 ],
                "c": [ 6, 7 ],
                "d": [ 123, 987, 999 ]
            },
            "pvalue": "payload",
            "variable": {
                "one": true,
                "two": false
            }
        })
    }
}
