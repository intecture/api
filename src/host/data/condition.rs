// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use error::{Error, Result};
use serde_json::Value;
use std::fmt;
use std::iter::{Enumerate, Peekable};
use std::str::Chars;
use std::vec::IntoIter;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    GroupInit,
    GroupTerm,
    Pointer(String),
    Value(Value),
    Cop(ComparisonOperator),
    Lop(LogicalOperator),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::GroupInit => write!(f, "("),
            Token::GroupTerm => write!(f, ")"),
            Token::Pointer(ref s) => write!(f, "{}", s),
            Token::Value(ref v) => write!(f, "{}", v),
            Token::Cop(ref c) => write!(f, "{}", c),
            Token::Lop(ref l) => write!(f, "{}", l),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ComparisonOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanEqualTo,
    LessThan,
    LessThanEqualTo,
}

impl fmt::Display for ComparisonOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ComparisonOperator::Equals => write!(f, "="),
            ComparisonOperator::NotEquals => write!(f, "!="),
            ComparisonOperator::GreaterThan => write!(f, ">"),
            ComparisonOperator::GreaterThanEqualTo => write!(f, ">="),
            ComparisonOperator::LessThan => write!(f, "<"),
            ComparisonOperator::LessThanEqualTo => write!(f, "<="),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum LogicalOperator {
    And,
    Or,
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LogicalOperator::And => write!(f, " && "),
            LogicalOperator::Or => write!(f, " || "),
        }
    }
}

pub fn eval(data: &Value, query: &str) -> Result<bool> {
    let mut iter = query.chars().peekable();
    let mut tokens = tokenize(&mut iter);

    // In order to avoid code duplication, the token vector is
    // wrapped in group tokens in order that we can pass it directly
    // to the group parser.
    // Note that only the GroupTerm token is required as the group
    // parser expects that a higher level group has already matched
    // the Group (start) token.
    tokens.push(Token::GroupTerm);
    let mut iter = tokens.into_iter().enumerate();
    Ok(try!(parse(&mut iter, data)))
}

fn tokenize(iter: &mut Peekable<Chars>) -> Vec<Token> {
    let mut buf = Vec::new();
    let mut buf_is_value = true;
    let mut escape = false;
    let mut quotes: Option<char> = None;
    let mut skip = 0;
    let mut tokens = Vec::new();
    let whitespace = ' ';

    while let Some(c) = iter.next() {
        {
            let next = iter.peek().unwrap_or(&whitespace);

            match c {
                '(' if !escape && quotes.is_none() => {
                    buf = tokenize_buf(&mut tokens, buf, buf_is_value, quotes.is_some());
                    tokens.push(Token::GroupInit);
                },
                ')' if !escape && quotes.is_none() => {
                    buf = tokenize_buf(&mut tokens, buf, buf_is_value, quotes.is_some());
                    tokens.push(Token::GroupTerm);
                },
                '&' if !escape && quotes.is_none() && *next == '&' => {
                    skip = 1;
                    buf = tokenize_buf(&mut tokens, buf, buf_is_value, quotes.is_some());
                    tokens.push(Token::Lop(LogicalOperator::And));
                },
                '|' if !escape && quotes.is_none() && *next == '|' => {
                    skip = 1;
                    buf = tokenize_buf(&mut tokens, buf, buf_is_value, quotes.is_some());
                    tokens.push(Token::Lop(LogicalOperator::Or));
                },
                '=' if !escape && quotes.is_none() => {
                    if *next == '=' {
                        skip = 1;
                    }
                    buf = tokenize_buf(&mut tokens, buf, buf_is_value, quotes.is_some());
                    tokens.push(Token::Cop(ComparisonOperator::Equals));
                },
                '!' if !escape && quotes.is_none() && *next == '=' => {
                    skip = 1;
                    buf = tokenize_buf(&mut tokens, buf, buf_is_value, quotes.is_some());
                    tokens.push(Token::Cop(ComparisonOperator::NotEquals));
                },
                '>' if !escape && quotes.is_none() => {
                    buf = tokenize_buf(&mut tokens, buf, buf_is_value, quotes.is_some());
                    if *next == '=' {
                        skip = 1;
                        tokens.push(Token::Cop(ComparisonOperator::GreaterThanEqualTo));
                    } else {
                        tokens.push(Token::Cop(ComparisonOperator::GreaterThan));
                    }
                },
                '<' if !escape && quotes.is_none() => {
                    buf = tokenize_buf(&mut tokens, buf, buf_is_value, quotes.is_some());
                    if *next == '=' {
                        skip = 1;
                        tokens.push(Token::Cop(ComparisonOperator::LessThanEqualTo));
                    } else {
                        tokens.push(Token::Cop(ComparisonOperator::LessThan));
                    }
                },
                '"' | '\'' if !escape && (quotes.is_none() || quotes == Some(c)) => {
                    if quotes.is_none() {
                        quotes = Some(c);
                        buf_is_value = true;
                    } else {
                        quotes = None;
                        buf = tokenize_buf(&mut tokens, buf, true, true);
                    }
                },
                '/' if buf.is_empty() && quotes.is_none() => {
                    buf_is_value = false;
                    buf.push(c);
                },
                '\\' => {
                    if escape {
                        buf.push(c);
                    }
                },
                _ if c.is_whitespace() && !escape && quotes.is_none() => {
                    buf = tokenize_buf(&mut tokens, buf, buf_is_value, quotes.is_some());
                },
                _ => {
                    if buf.is_empty() {
                        buf_is_value = true;
                    }

                    buf.push(c);
                },
            }
        }

        while skip > 0 {
            iter.next();
            skip -= 1;
        }

        escape = !escape && c == '\\';
    }

    if !buf.is_empty() {
        tokenize_buf(&mut tokens, buf, buf_is_value, quotes.is_some());
    }

    tokens
}

fn tokenize_buf(tokens: &mut Vec<Token>, buf: Vec<char>, value: bool, quotes: bool) -> Vec<char> {
    if !buf.is_empty() {
        if value {
            // Attempt to match integer
            if !quotes && buf.iter().all(|&c| c.is_digit(10) || c == '.' || c == '-') {
                if buf.contains(&'.') {
                    let s: String = buf.into_iter().collect();

                    match s.parse::<f64>() {
                        Ok(i) => tokens.push(Token::Value(Value::F64(i))),
                        Err(_) => tokens.push(Token::Value(Value::String(s))),
                    }
                } else if buf.starts_with(&['-']) {
                    let s: String = buf.into_iter().collect();

                    match s.parse::<i64>() {
                        Ok(i) => tokens.push(Token::Value(Value::I64(i))),
                        Err(_) => tokens.push(Token::Value(Value::String(s))),
                    }
                } else {
                    let s: String = buf.into_iter().collect();

                    match s.parse::<u64>() {
                        Ok(i) => tokens.push(Token::Value(Value::U64(i))),
                        Err(_) => tokens.push(Token::Value(Value::String(s))),
                    }
                }
            } else {
                let s: String = buf.into_iter().collect();
                if !quotes && s.to_lowercase() == "true" {
                    tokens.push(Token::Value(Value::Bool(true)));
                } else if !quotes && s.to_lowercase() == "false" {
                    tokens.push(Token::Value(Value::Bool(false)));
                } else if !quotes && s.to_lowercase() == "null" {
                    tokens.push(Token::Value(Value::Null));
                } else {
                    tokens.push(Token::Value(Value::String(s)));
                }
            }
        } else {
            tokens.push(Token::Pointer(buf.into_iter().collect()));
        }
    }

    Vec::new()
}

fn parse(tokens: &mut Enumerate<IntoIter<Token>>, data: &Value) -> Result<bool> {
    let mut status = false;
    let mut previous = (Token::GroupInit, Token::GroupInit);

    while let Some((i, token)) = tokens.next() {
        match token {
            Token::GroupInit if match previous.0 { Token::GroupInit | Token::Lop(_) => true, _ => false } =>
                {
                    status = try!(parse(tokens, data));

                    // Correct the previous tokens as the actual
                    // values are destroyed in the child parse() call
                    previous = (Token::GroupTerm, Token::GroupInit);
                    continue;
                },
            Token::GroupTerm if match previous.0 { Token::Cop(_) | Token::Lop(_) => false, _ => true} => break,
            Token::Cop(_) if match previous.0 { Token::Pointer(_) | Token::Value(_) => true, _ => false } &&
                             match previous.1 { Token::Cop(_) => false, _ => true} => (),
            Token::Lop(ref l) if match previous.0 { Token::Pointer(_) | Token::Value(_) | Token::GroupTerm => true, _ => false } &&
                             match previous.1 { Token::Lop(_) => false, _ => true} =>
                if (*l == LogicalOperator::And && !status) || (*l == LogicalOperator::Or && status) {
                    // Consume remaining tokens in this group as
                    // we've finished processing this group.
                    for (_, t) in tokens {
                        if t == Token::GroupTerm {
                            break;
                        }
                    }
                    break;
                },
            Token::Pointer(_) |
            Token::Value(_) if match previous.0 { Token::GroupInit | Token::Lop(_) => true, _ => false } => (),
            Token::Pointer(_) |
            Token::Value(_) if match previous.0 { Token::Cop(_) => true, _ => false } &&
                               match previous.1 { Token::Pointer(_) | Token::Value(_) => true, _ => false } =>
                match previous.0 {
                    Token::Cop(ref c) => status = try!(eval_condition(&previous.1, &c, &token, data)),
                    _ => unreachable!(),
                },
            _ => {
                let mut err_str = format!("Unexpected token {} in query excerpt ...", token);

                if i > 1 {
                    err_str.push_str(&format!("{}", previous.1));
                }

                if i > 0 {
                    err_str.push_str(&format!("{}", previous.0));
                    err_str.push_str(&format!("{}", token));
                }

                for _ in 0..2 {
                    if let Some((_, tok)) = tokens.next() {
                        err_str.push_str(&format!(" {}", tok));
                    }
                }

                err_str.push_str("...");

                return Err(Error::QueryParser(err_str));
            }
        }

        previous = (token, previous.0);
    }

    Ok(status)
}

fn eval_condition(predicate: &Token, cop: &ComparisonOperator, criteria: &Token, data: &Value) -> Result<bool> {
    let result = match *cop {
        ComparisonOperator::Equals =>
            try!(resolve_pointer(predicate, data)) == try!(resolve_pointer(criteria, data)),
        ComparisonOperator::NotEquals =>
            try!(resolve_pointer(predicate, data)) != try!(resolve_pointer(criteria, data)),
        ComparisonOperator::GreaterThan => {
            let pv = try!(resolve_pointer(predicate, data));
            let cv = try!(resolve_pointer(criteria, data));

            match pv {
                Value::F64(v1) if match cv { Value::F64(_) => true, _ => false } => v1 > cv.as_f64().unwrap(),
                Value::I64(v1) if match cv { Value::I64(_) => true, _ => false } => v1 > cv.as_i64().unwrap(),
                Value::U64(v1) if match cv { Value::U64(_) => true, _ => false } => v1 > cv.as_u64().unwrap(),
                _ => return Err(Error::QueryParser(format!("Cannot compare {:?} > {:?}. Values must be integers of same type.", pv, cv))),
            }
        },
        ComparisonOperator::GreaterThanEqualTo => {
            let pv = try!(resolve_pointer(predicate, data));
            let cv = try!(resolve_pointer(criteria, data));

            match pv {
                Value::F64(v1) if match cv { Value::F64(_) => true, _ => false } => v1 >= cv.as_f64().unwrap(),
                Value::I64(v1) if match cv { Value::I64(_) => true, _ => false } => v1 >= cv.as_i64().unwrap(),
                Value::U64(v1) if match cv { Value::U64(_) => true, _ => false } => v1 >= cv.as_u64().unwrap(),
                _ => return Err(Error::QueryParser(format!("Cannot compare {:?} >= {:?}. Values must be integers of same type.", pv, cv))),
            }
        },
        ComparisonOperator::LessThan => {
            let pv = try!(resolve_pointer(predicate, data));
            let cv = try!(resolve_pointer(criteria, data));

            match pv {
                Value::F64(v1) if match cv { Value::F64(_) => true, _ => false } => v1 < cv.as_f64().unwrap(),
                Value::I64(v1) if match cv { Value::I64(_) => true, _ => false } => v1 < cv.as_i64().unwrap(),
                Value::U64(v1) if match cv { Value::U64(_) => true, _ => false } => v1 < cv.as_u64().unwrap(),
                _ => return Err(Error::QueryParser(format!("Cannot compare {:?} < {:?}. Values must be integers of same type.", pv, cv))),
            }
        },
        ComparisonOperator::LessThanEqualTo => {
            let pv = try!(resolve_pointer(predicate, data));
            let cv = try!(resolve_pointer(criteria, data));

            match pv {
                Value::F64(v1) if match cv { Value::F64(_) => true, _ => false } => v1 <= cv.as_f64().unwrap(),
                Value::I64(v1) if match cv { Value::I64(_) => true, _ => false } => v1 <= cv.as_i64().unwrap(),
                Value::U64(v1) if match cv { Value::U64(_) => true, _ => false } => v1 <= cv.as_u64().unwrap(),
                _ => return Err(Error::QueryParser(format!("Cannot compare {:?} <= {:?}. Values must be integers of same type.", pv, cv))),
            }
        },
    };

    Ok(result)
}

fn resolve_pointer(token: &Token, data: &Value) -> Result<Value> {
    match *token {
        Token::Pointer(ref p) => match data.pointer(p) {
            Some(v) => Ok(v.clone()),
            // Currently favouring Null value over error. Experience
            // might suggest that a warning/error is more appropriate.
            None => Ok(Value::Null),
            // None => Err(Error::QueryParser(format!("JSON pointer \"{}\" does not exist in data", p)))
        },
        Token::Value(ref v) => Ok(v.clone()),
        _ => Err(Error::QueryParser("Token must be Pointer or Value token".into())),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;
    use std::collections::BTreeMap;
    use super::{ComparisonOperator, LogicalOperator, Token, eval, tokenize};

    #[test]
    fn test_eval() {
        let mut map = BTreeMap::new();
        map.insert("a".into(), Value::String("z".into()));
        map.insert("b".into(), Value::String("z".into()));
        map.insert("c".into(), Value::String("d".into()));
        map.insert("d".into(), Value::U64(1));
        map.insert("e".into(), Value::I64(2));
        map.insert("f".into(), Value::I64(1));

        let data = Value::Object(map);
        assert!(eval(&data, "(((/a=/b && /c!='e') || /d <= 0) || /e > /f) && /fake = NULL").expect("Query result bool"));
    }

    #[test]
    fn test_tokenize() {
        let expect_tokens = vec![
            Token::Pointer("/this/is/a/tok\\en".into()),
            Token::Cop(ComparisonOperator::Equals),
            Token::Value(Value::String("!=".into())),
            Token::Lop(LogicalOperator::And),
            Token::GroupInit,
            Token::Value(Value::String("value".into())),
            Token::Cop(ComparisonOperator::LessThanEqualTo),
            Token::Pointer("/path/to/=token".into()),
            Token::GroupTerm,
            Token::Value(Value::U64(1)),
            Token::Value(Value::I64(-1)),
            Token::Value(Value::F64(1.2)),
        ];

        let test_str = "/this/is/a/tok\\\\en = \"!=\" && (value<=/path/to/\\=token) 1 -1 1.2";
        let mut iter = test_str.chars().peekable();

        let tokens = tokenize(&mut iter);
        assert_eq!(tokens, expect_tokens);
    }
}
