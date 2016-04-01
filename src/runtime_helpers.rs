// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::ZCert;
use {Error, Result};
#[cfg(not(test))]
use std::env::{self, Args};
use std::process::exit;
#[cfg(test)]
use tempdir::TempDir;

#[cfg(test)]
lazy_static! {
    static ref TMP_DIR: TempDir = TempDir::new("test_runtime_args").unwrap();
    pub static ref RUNTIME_ARGS: RuntimeArgs = RuntimeArgs::new_test();
}
#[cfg(not(test))]
lazy_static! {
    pub static ref RUNTIME_ARGS: RuntimeArgs = RuntimeArgs::new_usage(env::args());
}

pub struct RuntimeArgs {
    pub user_cert: ZCert,
    pub server_cert_path: String,
    pub user_args: Vec<String>,
}

unsafe impl Sync for RuntimeArgs {}

impl RuntimeArgs {
    #[cfg(test)]
    fn new_test() -> RuntimeArgs {
        let server_cert = ZCert::new().unwrap();
        server_cert.save(&format!("{}/localhost.crt", TMP_DIR.path().to_str().unwrap())).unwrap();

        let user_cert = ZCert::new().unwrap();
        let user_path = format!("{}/user.crt", TMP_DIR.path().to_str().unwrap());
        user_cert.save(&user_path).unwrap();

         RuntimeArgs::new(vec![
             "/fake/runnable".to_string(),
             user_path,
             TMP_DIR.path().to_str().unwrap().to_string(),
         ]).unwrap()
    }

    #[cfg(not(test))]
    fn new_usage(args: Args) -> RuntimeArgs {
        let args: Vec<String> = args.collect();
        let ra = Self::new(args);

        if ra.is_err() {
            println!("You should not invoke this project manually!");
            println!("Usage: incli run [<script_args>]");
            exit(1);
        }

        ra.unwrap()
    }

    fn new(args: Vec<String>) -> Result<RuntimeArgs> {
        let mut args = args;

        if args.len() < 3 {
            return Err(Error::Generic("Missing args".to_string()));
        }

        // Remove filename
        args.remove(0);

        Ok(RuntimeArgs {
            user_cert: try!(ZCert::load(&args.remove(0))),
            server_cert_path: args.remove(0),
            user_args: args,
        })
    }

    pub fn expect_user_args<'a>(&'a mut self, required_args: &[&str]) -> &'a [String] {
        if self.user_args.len() != required_args.len() {
            // Generate usage string
            let mut usage = String::from("Usage: incli run");
            for arg in required_args {
                usage.push_str(" <");
                usage.push_str(arg);
                usage.push_str(">");
            }

            println!("{}", usage);
            exit(1);
        }

        &self.user_args
    }
}

#[cfg(test)]
mod tests {
    use czmq::ZCert;
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_new_ok() {
        let dir = TempDir::new("test_new_ok").unwrap();

        let user_path = format!("{}/user.crt", dir.path().to_str().unwrap());
        let server_path = format!("{}/localhost.crt", dir.path().to_str().unwrap());

        let user_cert = ZCert::new().unwrap();
        user_cert.save(&user_path).unwrap();

        let runtime_args = RuntimeArgs::new(vec![
            "/path/to/project/runnable".to_string(),
            user_path,
            server_path
        ]);
        assert!(runtime_args.is_ok());
        assert_eq!(runtime_args.unwrap().user_args.len(), 0);
    }

    #[test]
    fn test_new_noargs() {
        let runtime_args = RuntimeArgs::new(vec!["/path/to/project/runnable".to_string()]);
        assert!(runtime_args.is_err());
    }

    #[test]
    fn test_new_usrargs() {
        let dir = TempDir::new("test_new_usrargs").unwrap();

        let user_path = format!("{}/user.crt", dir.path().to_str().unwrap());
        let server_path = format!("{}/localhost.crt", dir.path().to_str().unwrap());

        let user_cert = ZCert::new().unwrap();
        user_cert.save(&user_path).unwrap();

         let runtime_args = RuntimeArgs::new(vec![
             "/path/to/project/runnable".to_string(),
             user_path,
             server_path,
             "user1".to_string(),
             "user2".to_string(),
         ]).unwrap();
        assert_eq!(runtime_args.user_args.get(0).unwrap(), "user1");
        assert_eq!(runtime_args.user_args.get(1).unwrap(), "user2");
    }
}
