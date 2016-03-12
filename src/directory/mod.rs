// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
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
//! let mut host = Host::new();
#![cfg_attr(feature = "remote-run", doc = " host.connect(\"127.0.0.1\", 7101, 7102, 7103).unwrap();")]
//! ```
//!
//! Now you can manage a directory on your managed host.
//!
//! ```no_run
//! # use inapi::{Host, Directory, DirectoryOpts};
//! # let mut host = Host::new();
//! let dir = Directory::new(&mut host, "/path/to/dir").unwrap();
//! dir.create(&mut host, Some(&vec![DirectoryOpts::DoRecursive])).unwrap();
//! dir.set_owner(&mut host, "MyUser", "MyGroup").unwrap();
//! dir.set_mode(&mut host, 644).unwrap();
//! ```

pub mod ffi;

use {FileOwner, Host, Result};
use error::Error;
use target::Target;

/// Options for controlling directory operations.
pub enum DirectoryOpts {
    /// Perform action recursively.
    DoRecursive,
}

/// Container for operating on a directory.
pub struct Directory {
    /// Absolute path to directory on managed host
    path: String,
}

impl Directory {
    /// Create a new Directory struct.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::{Directory, Host};
    /// let mut host = Host::new();
    /// let directory = Directory::new(&mut host, "/path/to/dir");
    /// ```
    pub fn new(host: &mut Host, path: &str) -> Result<Directory> {
        if ! try!(Target::directory_is_directory(host, path)) {
            return Err(Error::Generic("Path is a file".to_string()));
        }

        Ok(Directory {
            path: path.to_string(),
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
    pub fn mv(&self, host: &mut Host, new_path: &str) -> Result<()> {
        Target::directory_mv(host, &self.path, new_path)
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

pub trait DirectoryTarget {
    fn directory_is_directory(host: &mut Host, path: &str) -> Result<bool>;
    fn directory_exists(host: &mut Host, path: &str) -> Result<bool>;
    fn directory_create(host: &mut Host, path: &str, recursive: bool) -> Result<()>;
    fn directory_delete(host: &mut Host, path: &str, recursive: bool) -> Result<()>;
    fn directory_mv(host: &mut Host, path: &str, new_path: &str) -> Result<()>;
    fn directory_get_owner(host: &mut Host, path: &str) -> Result<FileOwner>;
    fn directory_set_owner(host: &mut Host, path: &str, user: &str, group: &str) -> Result<()>;
    fn directory_get_mode(host: &mut Host, path: &str) -> Result<u16>;
    fn directory_set_mode(host: &mut Host, path: &str, mode: u16) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use Host;
    use super::*;
    #[cfg(feature = "remote-run")]
    use std::thread;
    #[cfg(feature = "remote-run")]
    use zmq;

    // XXX local-run tests require FS mocking

    #[cfg(feature = "local-run")]
    #[test]
    fn test_new_ok() {
        let mut host = Host::new();
        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_ok() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_fail() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_err());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_exists() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::exists", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());
        assert_eq!(dir.unwrap().exists(&mut host).unwrap(), false);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_create() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::create", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("1", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());
        assert!(dir.unwrap().create(&mut host, Some(&vec![DirectoryOpts::DoRecursive])).is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_delete() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::delete", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("0", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());
        assert!(dir.unwrap().delete(&mut host, None).is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_mv() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/old", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::mv", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/old", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/new", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let dir = Directory::new(&mut host, "/path/to/old");
        assert!(dir.is_ok());
        assert!(dir.unwrap().mv(&mut host, "/path/to/new").is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_owner() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::get_owner", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Moo", zmq::SNDMORE).unwrap();
            agent_sock.send_str("123", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Cow", zmq::SNDMORE).unwrap();
            agent_sock.send_str("456", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());

        let owner = dir.unwrap().get_owner(&mut host).unwrap();
        assert_eq!(owner.user_name, "Moo");
        assert_eq!(owner.user_uid, 123);
        assert_eq!(owner.group_name, "Cow");
        assert_eq!(owner.group_gid, 456);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_owner() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::set_owner", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("Moo", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("Cow", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());
        assert!(dir.unwrap().set_owner(&mut host, "Moo", "Cow").is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_mode() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::get_mode", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("755", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());
        assert_eq!(dir.unwrap().get_mode(&mut host).unwrap(), 755);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_mode() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("directory::is_directory", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("directory::set_mode", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/path/to/dir", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("644", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let dir = Directory::new(&mut host, "/path/to/dir");
        assert!(dir.is_ok());
        assert!(dir.unwrap().set_mode(&mut host, 644).is_ok());

        agent_mock.join().unwrap();
    }
}
