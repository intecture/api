// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

//! The primitive for managing files on a managed host.
//!
//! # Examples
//!
//! Initialise a new Host using your managed host's IP address and
//! port number:
//!
//! ```no_run
//! # use inapi::Host;
#![cfg_attr(feature = "local-run", doc = "let path: Option<String> = None;")]
#![cfg_attr(feature = "local-run", doc = "let mut host = Host::local(path).unwrap();")]
#![cfg_attr(feature = "remote-run", doc = "let mut host = Host::connect(\"data/nodes/mynode.json\").unwrap();")]
//! ```
//!
//! Now you can manage a file on your managed host.
//!
//! ```no_run
//! # use inapi::{Host, File, FileOptions};
#![cfg_attr(feature = "local-run", doc = "# let path: Option<String> = None;")]
#![cfg_attr(feature = "local-run", doc = "# let mut host = Host::local(path).unwrap();")]
#![cfg_attr(feature = "remote-run", doc = "# let mut host = Host::connect(\"data/nodes/mynode.json\").unwrap();")]
//! let file = File::new(&mut host, "/path/to/destination_file").unwrap();
#![cfg_attr(feature = "remote-run", doc = " file.upload(&mut host, \"/path/to/local_file\", None);")]
//! file.set_owner(&mut host, "MyUser", "MyGroup").unwrap();
//! file.set_mode(&mut host, 644).unwrap();
//!
#![cfg_attr(feature = "remote-run", doc = " // Now let's upload another file and backup the original")]
#![cfg_attr(feature = "remote-run", doc = " file.upload(&mut host, \"/path/to/new_file\", Some(&vec![FileOptions::BackupExisting(\"_bk\".to_string())])).unwrap();")]
#![cfg_attr(feature = "remote-run", doc = "")]
#![cfg_attr(feature = "remote-run", doc = " // Your remote path now has two entries:")]
#![cfg_attr(feature = "remote-run", doc = " // \"/path/to/destination_file\" and \"/path/to/destination_file_bk\"")]
//! ```

pub mod ffi;

use error::Result;
use host::Host;
#[cfg(feature = "remote-run")]
use host::HostSendRecv;
use error::Error;
#[cfg(feature = "remote-run")]
use std::fs;
use std::path::{Path, PathBuf};
use target::Target;
#[cfg(feature = "remote-run")]
use zfilexfer;

/// Owner's user and group for a file.
#[derive(Debug)]
pub struct FileOwner {
    /// User name
    pub user_name: String,
    /// User UID
    pub user_uid: u64,
    /// Group name
    pub group_name: String,
    /// Group GID
    pub group_gid: u64,
}

/// Container for operating on a file.
pub struct File {
    /// Absolute path to file on managed host
    path: PathBuf,
}

impl File {
    /// Create a new File struct.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::{File, Host};
    #[cfg_attr(feature = "local-run", doc = "let path: Option<String> = None;")]
    #[cfg_attr(feature = "local-run", doc = "let mut host = Host::local(path).unwrap();")]
    #[cfg_attr(feature = "remote-run", doc = "let mut host = Host::connect(\"data/nodes/mynode.json\").unwrap();")]
    /// let file = File::new(&mut host, "/path/to/file");
    /// ```
    pub fn new<P: AsRef<Path>>(host: &mut Host, path: P) -> Result<File> {
        if ! try!(Target::file_is_file(host, path.as_ref())) {
            return Err(Error::Generic("Path is a directory".to_string()));
        }

        Ok(File {
            path: path.as_ref().into(),
        })
    }

    /// Check if the file exists.
    pub fn exists(&self, host: &mut Host) -> Result<bool> {
        Target::file_exists(host, &self.path)
    }

    #[cfg(feature = "remote-run")]
    /// Upload a file to the managed host.
    pub fn upload<P: AsRef<Path>>(&self, host: &mut Host, local_path: P, options: Option<&[zfilexfer::FileOptions]>) -> Result<()> {
        let mut file = try!(zfilexfer::File::open(&local_path, options));
        host.send_file(&mut file, &self.path)
    }

    #[cfg(feature = "remote-run")]
    /// Upload a file handle to the managed host.
    pub fn upload_file(&self, host: &mut Host, file: fs::File, options: Option<&[zfilexfer::FileOptions]>) -> Result<()> {
        let mut zfile = try!(zfilexfer::File::open_file(file, options));
        host.send_file(&mut zfile, &self.path)
    }

    /// Delete the file.
    pub fn delete(&self, host: &mut Host) -> Result<()> {
        Target::file_delete(host, &self.path)
    }

    /// Move the file to a new path.
    pub fn mv<P: AsRef<Path>>(&mut self, host: &mut Host, new_path: P) -> Result<()> {
        let new_path = new_path.as_ref().to_owned();
        try!(Target::file_mv(host, &self.path, &new_path));
        self.path = new_path;
        Ok(())
    }

    /// Copy the file to a new path.
    pub fn copy<P: AsRef<Path>>(&self, host: &mut Host, new_path: P) -> Result<()> {
        let new_path = new_path.as_ref().to_owned();
        Target::file_copy(host, &self.path, &new_path)
    }

    /// Get the file's owner.
    pub fn get_owner(&self, host: &mut Host) -> Result<FileOwner> {
        Target::file_get_owner(host, &self.path)
    }

    /// Set the file's owner.
    pub fn set_owner(&self, host: &mut Host, user: &str, group: &str) -> Result<()> {
        Target::file_set_owner(host, &self.path, user, group)
    }

    /// Get the file's permissions mask.
    pub fn get_mode(&self, host: &mut Host) -> Result<u16> {
        Target::file_get_mode(host, &self.path)
    }

    /// Set the file's permissions mask.
    pub fn set_mode(&self, host: &mut Host, mode: u16) -> Result<()> {
        Target::file_set_mode(host, &self.path, mode)
    }
}

pub trait FileTarget<P: AsRef<Path>> {
    fn file_is_file(host: &mut Host, path: P) -> Result<bool>;
    fn file_exists(host: &mut Host, path: P) -> Result<bool>;
    fn file_delete(host: &mut Host, path: P) -> Result<()>;
    fn file_mv(host: &mut Host, path: P, new_path: P) -> Result<()>;
    fn file_copy(host: &mut Host, path: P, new_path: P) -> Result<()>;
    fn file_get_owner(host: &mut Host, path: P) -> Result<FileOwner>;
    fn file_set_owner(host: &mut Host, path: P, user: &str, group: &str) -> Result<()>;
    fn file_get_mode(host: &mut Host, path: P) -> Result<u16>;
    fn file_set_mode(host: &mut Host, path: P, mode: u16) -> Result<()>;
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "remote-run")]
    use czmq::{ZMsg, ZSys};
    use host::Host;
    #[cfg(feature = "remote-run")]
    use std::thread;
    use super::*;

    #[cfg(feature = "local-run")]
    #[test]
    fn test_new_ok() {
        let path: Option<String> = None;
        let mut host = Host::local(path).unwrap();
        let file = File::new(&mut host, "/path/to/file");
        assert!(file.is_ok());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::is_file", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            server.recv_str().unwrap().unwrap();

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("0").unwrap();
            reply.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let file = File::new(&mut host, "/tmp/test");
        assert!(file.is_ok());

        let file = File::new(&mut host, "/tmp/test");
        assert!(file.is_err());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_exists() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::is_file", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::exists", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let file = File::new(&mut host, "/tmp/test").unwrap();
        assert!(file.exists(&mut host).unwrap());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_delete() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::is_file", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::delete", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let file = File::new(&mut host, "/tmp/test").unwrap();
        assert!(file.delete(&mut host).is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_mv() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::is_file", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/old", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::mv", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/old", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/new", msg.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let mut file = File::new(&mut host, "/tmp/old").unwrap();
        assert!(file.mv(&mut host, "/tmp/new").is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_copy() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::is_file", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/existing", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::copy", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/existing", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/new", msg.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let file = File::new(&mut host, "/tmp/existing").unwrap();
        assert!(file.copy(&mut host, "/tmp/new").is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_owner() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::is_file", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::get_owner", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("user").unwrap();
            reply.addstr("123").unwrap();
            reply.addstr("group").unwrap();
            reply.addstr("123").unwrap();
            reply.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let file = File::new(&mut host, "/tmp/test").unwrap();
        let owner = file.get_owner(&mut host).unwrap();
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
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::is_file", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::set_owner", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());
            assert_eq!("user", msg.popstr().unwrap().unwrap());
            assert_eq!("group", msg.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let file = File::new(&mut host, "/tmp/test").unwrap();
        assert!(file.set_owner(&mut host, "user", "group").is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_mode() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::is_file", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::get_mode", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("755").unwrap();
            reply.send(&mut server).unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let file = File::new(&mut host, "/tmp/test").unwrap();
        assert_eq!(file.get_mode(&mut host).unwrap(), 755);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_mode() {
        ZSys::init();

        let (client, mut server) = ZSys::create_pipe().unwrap();

        let agent_mock = thread::spawn(move || {
            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::is_file", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());

            let reply = ZMsg::new();
            reply.addstr("Ok").unwrap();
            reply.addstr("1").unwrap();
            reply.send(&mut server).unwrap();

            let msg = ZMsg::recv(&mut server).unwrap();
            assert_eq!("file::set_mode", msg.popstr().unwrap().unwrap());
            assert_eq!("/tmp/test", msg.popstr().unwrap().unwrap());
            assert_eq!("644", msg.popstr().unwrap().unwrap());

            server.send_str("Ok").unwrap();
        });

        let mut host = Host::test_new(None, Some(client), None, None);

        let file = File::new(&mut host, "/tmp/test").unwrap();
        assert!(file.set_mode(&mut host, 644).is_ok());

        agent_mock.join().unwrap();
    }
}
