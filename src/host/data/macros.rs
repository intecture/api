// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

macro_rules! want_macro {
    ($t:expr, $n:ident, $isf:ident, $asf:ident) => {
        /// Helper that returns an Option<Value::$t>.
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

want_macro!("Null", wantnull, is_null, as_null);
want_macro!("Bool", wantbool, is_boolean, as_bool);
want_macro!("I64", wanti64, is_i64, as_i64);
want_macro!("U64", wantu64, is_u64, as_u64);
want_macro!("F64", wantf64, is_f64, as_f64);
want_macro!("String", wantstr, is_string, as_str);
want_macro!("Array", wantarray, is_array, as_array);
want_macro!("Object", wantobj, is_object, as_object);

// XXX We can't auto-generate macros due to the eager expansion of
// `$crate`, which causes any external libs to look for ::error::...
// instead of inapi::error::...
//
// macro_rules! need_macro {
//     ($t:expr, $n:ident, $isf:ident, $asf:ident) => {
//         /// Helper that returns a Result<Value::$t>.
//         /// You can optionally pass a JSON pointer to retrieve a
//         /// nested key.
//         #[macro_export]
//         macro_rules! $n {
//             ($v:expr) => (if $v.$isf() {
//                 Ok($v.$asf().unwrap())
//             } else {
//                 Err($crate::Error::Generic(format!("Value is not $t")))
//             });
//
//             ($v:expr => $p:expr) => (if let Some(v) = $v.pointer($p) {
//                 $n!(v)
//             } else {
//                 Err($crate::Error::Generic(format!("Could not find {} in data", $p)))
//             });
//         }
//     }
// }
//
// need_macro!("Null", neednull, is_null, as_null);
// need_macro!("Bool", needbool, is_boolean, as_bool);
// need_macro!("I64", needi64, is_i64, as_i64);
// need_macro!("U64", needu64, is_u64, as_u64);
// need_macro!("F64", needf64, is_f64, as_f64);
// need_macro!("String", needstr, is_string, as_str);
// need_macro!("Array", needarray, is_array, as_array);
// need_macro!("Object", needobj, is_object, as_object);

/// Helper that returns a Result<Value::Null>.
/// You can optionally pass a JSON pointer to retrieve a nested key.
#[macro_export]
macro_rules! neednull {
    ($v:expr) => (if $v.is_null() {
        Ok($v.as_null().unwrap())
    } else {
        Err($crate::Error::Generic(format!("Value is not null")))
    });

    ($v:expr => $p:expr) => (if let Some(v) = $v.pointer($p) {
        neednull!(v)
    } else {
        Err($crate::Error::Generic(format!("Could not find {} in data", $p)))
    });
}

/// Helper that returns a Result<Value::Bool>.
/// You can optionally pass a JSON pointer to retrieve a nested key.
#[macro_export]
macro_rules! needbool {
    ($v:expr) => (if $v.is_boolean() {
        Ok($v.as_bool().unwrap())
    } else {
        Err($crate::Error::Generic(format!("Value is not boolean")))
    });

    ($v:expr => $p:expr) => (if let Some(v) = $v.pointer($p) {
        needbool!(v)
    } else {
        Err($crate::Error::Generic(format!("Could not find {} in data", $p)))
    });
}

/// Helper that returns a Result<Value::I64>.
/// You can optionally pass a JSON pointer to retrieve a nested key.
#[macro_export]
macro_rules! needi64 {
    ($v:expr) => (if $v.is_i64() {
        Ok($v.as_i64().unwrap())
    } else {
        Err($crate::Error::Generic(format!("Value is not i64")))
    });

    ($v:expr => $p:expr) => (if let Some(v) = $v.pointer($p) {
        needi64!(v)
    } else {
        Err($crate::Error::Generic(format!("Could not find {} in data", $p)))
    });
}

/// Helper that returns a Result<Value::U64>.
/// You can optionally pass a JSON pointer to retrieve a nested key.
#[macro_export]
macro_rules! needu64 {
    ($v:expr) => (if $v.is_u64() {
        Ok($v.as_u64().unwrap())
    } else {
        Err($crate::Error::Generic(format!("Value is not u64")))
    });

    ($v:expr => $p:expr) => (if let Some(v) = $v.pointer($p) {
        needu64!(v)
    } else {
        Err($crate::Error::Generic(format!("Could not find {} in data", $p)))
    });
}

/// Helper that returns a Result<Value::F64>.
/// You can optionally pass a JSON pointer to retrieve a nested key.
#[macro_export]
macro_rules! needf64 {
    ($v:expr) => (if $v.is_f64() {
        Ok($v.as_f64().unwrap())
    } else {
        Err($crate::Error::Generic(format!("Value is not f64")))
    });

    ($v:expr => $p:expr) => (if let Some(v) = $v.pointer($p) {
        needf64!(v)
    } else {
        Err($crate::Error::Generic(format!("Could not find {} in data", $p)))
    });
}

/// Helper that returns a Result<Value::String>.
/// You can optionally pass a JSON pointer to retrieve a nested key.
#[macro_export]
macro_rules! needstr {
    ($v:expr) => (if $v.is_string() {
        Ok($v.as_str().unwrap())
    } else {
        Err($crate::Error::Generic(format!("Value is not string")))
    });

    ($v:expr => $p:expr) => (if let Some(v) = $v.pointer($p) {
        needstr!(v)
    } else {
        Err($crate::Error::Generic(format!("Could not find {} in data", $p)))
    });
}

/// Helper that returns a Result<Value::Array>.
/// You can optionally pass a JSON pointer to retrieve a nested key.
#[macro_export]
macro_rules! needarray {
    ($v:expr) => (if $v.is_array() {
        Ok($v.as_array().unwrap())
    } else {
        Err($crate::Error::Generic(format!("Value is not array")))
    });

    ($v:expr => $p:expr) => (if let Some(v) = $v.pointer($p) {
        needarray!(v)
    } else {
        Err($crate::Error::Generic(format!("Could not find {} in data", $p)))
    });
}

/// Helper that returns a Result<Value::Object>.
/// You can optionally pass a JSON pointer to retrieve a nested key.
#[macro_export]
macro_rules! needobj {
    ($v:expr) => (if $v.is_object() {
        Ok($v.as_object().unwrap())
    } else {
        Err($crate::Error::Generic(format!("Value is not object")))
    });

    ($v:expr => $p:expr) => (if let Some(v) = $v.pointer($p) {
        needobj!(v)
    } else {
        Err($crate::Error::Generic(format!("Could not find {} in data", $p)))
    });
}
