// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The primitive for managing directories on a managed host.
//!
//! # Examples
//!
//! Initialise a new Host using your managed host's IP address and
//! port number:
//!
//! ```no_run
//! # use inapi::Host;
#![cfg_attr(feature = "local-run", doc = "let mut host = Host::local(None);")]
#![cfg_attr(feature = "remote-run", doc = "let mut host = Host::connect(\"data/nodes/mynode.json\").unwrap();")]
//! ```
//!
//! Now you can manage a directory on your managed host.
//!
//! ```no_run
//! # use inapi::{Host, Directory, DirectoryOpts};
//! # let mut host = Host::local(None);
//! let dir = Directory::new(&mut host, "/path/to/dir").unwrap();
//! dir.create(&mut host, Some(&vec![DirectoryOpts::DoRecursive])).unwrap();
//! dir.set_owner(&mut host, "MyUser", "MyGroup").unwrap();
//! dir.set_mode(&mut host, 644).unwrap();
//! ```

pub mod ffi;

use error::{Error, Result};
use file::FileOwner;
use host::Host;
use std::path::{Path, PathBuf};
use target::Target;

/// Options for controlling directory operations.
pub enum DirectoryOpts {
    /// Perform action recursively.
    DoRecursive,
}

/// Container for operating on a directory.
pub struct Directory {
    /// Absolute path to directory on managed host
    path: PathBuf,
}

impl Directory {
    /// Create a new Directory struct.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::{Directory, Host};
    /// let mut host = Host::local(None);
    /// let directory = Directory::new(&mut host, "/path/to/dir");
    /// ```
    pub fn new<P: AsRef<Path>>(host: &mut Host, path: P) -> Result<Directory> {
        if ! try!(Target::directory_is_directory(host, path.as_ref())) {
            return Err(Error::Generic("Path is a file".to_string()));
        }

        Ok(Directory {
            path: path.as_ref().into(),
        })
    }

    /// Check if the directory exists.
    pub fn exists(&self, host: &mut Host) -> Result<bool> {
        Target::directory_exists(host, &self.path)
    }

    /// Create the directory.
    pub fn create(&self, host: &mut Host, options: Option<&[DirectoryOpts]>) -> Result<()> {
        let mut recursive = false;

        if let Some(opts) = options {
            for opt in opts {
                match opt {
                    &DirectoryOpts::DoRecursive => recursive = true,
                }
            }
        }

        Target::directory_create(host, &self.path, recursive)
    }

    /// Delete the directory.
    pub fn delete(&self, host: &mut Host, options: Option<&[DirectoryOpts]>) -> Result<()> {
        let mut recursive = false;

        if let Some(opts) = options {
            for opt in opts {
                match opt {
                    &DirectoryOpts::DoRecursive => recursive = true,
                }
            }
        }

        Target::directory_delete(host, &self.path, recursive)
    }

    /// Move the directory to a new path.
    pub fn mv<P: AsRef<Path>>(&mut self, host: &mut Host, new_path: P) -> Result<()> {
        let new_path = new_path.as_ref().to_owned();
        try!(Target::directory_mv(host, &self.path, &new_path));
        self.path = new_path;
        Ok(())
    }

    /// Get the directory's owner.
    pub fn get_owner(&self, host: &mut Host) -> Result<FileOwner> {
        Target::directory_get_owner(host, &self.path)
    }

    // Set the directory's owner.
    pub fn set_owner(&self, host: &mut Host, user: &str, group: &str) -> Result<()> {
        Target::directory_set_owner(host, &self.path, user, group)
    }

    /// Get the directory's permissions mask.
    pub fn get_mode(&self, host: &mut Host) -> Result<u16> {
        Target::directory_get_mode(host, &self.path)
    }

    /// Set the directory's permissions mask.
    pub fn set_mode(&self, host: &mut Host, mode: u16) -> Result<()> {
        Target::directory_set_mode(host, &self.path, mode)
    }
}

pub trait DirectoryTarget<P: AsRef<Path>> {
    fn directory_is_directory(host: &mut Host, path: P) -> Result<bool>;
    fn directory_exists(host: &mut Host, path: P) -> Result<bool>;
    fn directory_create(host: &mut Host, path: P, recursive: bool) -> Result<()>;
    fn directory_delete(host: &mut Host, path: P, recursive: bool) -> Result<()>;
    fn directory_mv(host: &mut Host, path: P, new_path: P) -> Result<()>;
    fn directory_get_owner(host: &mut Host, path: P) -> Result<FileOwner>;
    fn directory_set_owner(host: &mut Host, path: P, user: &str, group: &str) -> Result<()>;
    fn directory_get_mode(host: &mut Host, path: P) -> Result<u16>;
    fn directory_set_mode(host: &mut Host, path: P, mode: u16) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use Host;
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    use super::*;
    #[cfg(feature = "remote-run")]
    use std::thread;

    // XXX local-run tests require FS mocking

    #[cfg(feature = "local-run")]
    #[test]
    fn test_new_ok() {
        let mut host = Host::local(None);
        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_ok() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::is_directory", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::is_directory", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_err());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_exists() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::is_directory", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::exists", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("0").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir").unwrap();
        assert!(!dir.exists(&mut host).unwrap());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_create() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::is_directory", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::create", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());
            assert_eq!("1", req.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir").unwrap();
        assert!(dir.create(&mut host, Some(&vec![DirectoryOpts::DoRecursive])).is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_delete() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::is_directory", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::delete", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());
            assert_eq!("0", req.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir").unwrap();
        assert!(dir.delete(&mut host, None).is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_mv() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::is_directory", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/old", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::mv", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/old", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/new", req.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let mut dir = Directory::new(&mut host, "/path/to/old").unwrap();
        assert!(dir.mv(&mut host, "/path/to/new").is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_owner() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::is_directory", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::get_owner", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("user").unwrap();
            rep.addstr("123").unwrap();
            rep.addstr("group").unwrap();
            rep.addstr("123").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());

        let owner = dir.unwrap().get_owner(&mut host).unwrap();
        assert_eq!(owner.user_name, "user");
        assert_eq!(owner.user_uid, 123);
        assert_eq!(owner.group_name, "group");
        assert_eq!(owner.group_gid, 123);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_owner() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::is_directory", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::set_owner", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());
            assert_eq!("user", req.popstr().unwrap().unwrap());
            assert_eq!("group", req.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());
        assert!(dir.unwrap().set_owner(&mut host, "user", "group").is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_mode() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::is_directory", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::get_mode", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("755").unwrap();
            rep.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());
        assert_eq!(dir.unwrap().get_mode(&mut host).unwrap(), 755);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_mode() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::is_directory", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());

            let rep = ZMsg::new();
            rep.addstr("Ok").unwrap();
            rep.addstr("1").unwrap();
            rep.send(&mut server).unwrap();

            let req = ZMsg::recv(&mut server).unwrap();
            assert_eq!("directory::set_mode", req.popstr().unwrap().unwrap());
            assert_eq!("/path/to/dir", req.popstr().unwrap().unwrap());
            assert_eq!("644", req.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());
        assert!(dir.unwrap().set_mode(&mut host, 644).is_ok());

        agent_mock.join().unwrap();
    }
}
