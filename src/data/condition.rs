// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use error::{Error, Result};
use std::iter::Peekable;
use std::str::Chars;

pub fn parse(query: &str) -> Result<bool> {
    let mut iter = query.chars().peekable();
    let result = try!(group(&mut iter));
    // let q: String = iter.collect();
    Ok(result)
}

fn group(iter: &mut Peekable<Chars>) -> Result<bool> {
    let mut status = true;

    for c in iter {
        match c {
            '&' if iter.peek() == Some(&'&') && !status => break,
            '&' if iter.peek() == Some(&'&') => { iter.next(); },
            '|' if iter.peek() == Some(&'|') => { iter.next(); },
            '(' => status = try!(group(iter)),
            ')' => break,
            _ => status = try!(condition(iter)),
        }
    }

    Ok(status)
}

// new querybuilder
// -> new condgroup <--------------
//   -> If group: new condgroup >-^
//   -> Else: new cond <---------------------------<
//     -> cond state == POINTER                    |
//     -> If quote: match until quote              |
//     -> Else: match until operator               |
//       -> cond state = OPERATOR                  |
//       -> match operator...                      |
//         -> cond state = VALUE                   |
//           -> If quote: match until quote        |
//           -> Else: match until EOF/whitespace   |
//           -> Return result bool                 |
//   -> If operator exists: match operator...      |
//     -> If AND:                                  |
//       -> If previous cond == true >-------------^
//       -> Else: Return false
//     -> Else if OR:
//       ->
//     -> Else: Return Err

fn condition(iter: &mut Peekable<Chars>) -> Result<bool> {
    let pointer = try!(scan_delim(iter).ok_or(Error::InvalidCondition("Missing JSON pointer".into())));
    // XXX Lookup value from top level JSON Value
    let json_value = "";

    // Remove whitespace
    iter.take_while(|c| *c == ' ' || *c == '\t' || *c == '\n');

    let operator = match iter.next() {
        Some('=') => "=",
        Some('!') if iter.peek() == Some(&'=') => { iter.next(); "!=" },
        Some('<') if iter.peek() == Some(&'=') => { iter.next(); "<=" },
        Some('<') => "<",
        Some('>') if iter.peek() == Some(&'=') => { iter.next(); ">=" },
        Some('>') => ">",
        _ => return Err(Error::InvalidCondition("Unknown comparison operator".into())),
    };

    // Remove whitespace
    iter.take_while(|c| *c == ' ' || *c == '\t' || *c == '\n');

    let value = try!(scan_delim(iter).ok_or(Error::InvalidCondition("Missing comparison value".into())));

    match operator {
        "=" if json_value == value => return Ok(true),
        "!=" if json_value != value => return Ok(true),
        "<" | "<=" | ">" | ">=" => {
            if let Ok(json_int) = json_value.parse::<u32>() {
                if let Ok(value_int) = value.parse::<u32>() {
                    if (operator == "<" && json_int < value_int) ||
                       (operator == "<=" && json_int <= value_int) ||
                       (operator == ">" && json_int > value_int) ||
                       (operator == ">=" && json_int >= value_int) {
                        return Ok(true);
                    }
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(false)
}

fn scan_delim(iter: &mut Peekable<Chars>) -> Option<String> {
    let mut escape = false;
    let mut found = false;

    let mut delim = match iter.peek() {
        Some(&'"') => {
            iter.next();
            '"'
        },
        Some(&'\'') => {
            iter.next();
            '\''
        },
        _ => ' ',
    };

    let contents: String = iter.by_ref()
                               .take_while(|c| if *c == delim && !escape {
                                   found = true;
                                   true
                               } else {
                                   escape = *c == '\\';
                                   false
                               })
                               .cloned()
                               .collect();

    if found {
        Some(contents)
    } else {
        None
    }
}
