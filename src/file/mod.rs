// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
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
//! let mut host = Host::new();
#![cfg_attr(feature = "remote-run", doc = " host.connect(\"127.0.0.1\", 7101, 7102, 7103).unwrap();")]
//! ```
//!
//! Now you can manage a file on your managed host.
//!
//! ```no_run
//! # use inapi::{Host, File, FileOpts};
//! # let mut host = Host::new();
//! let file = File::new(&mut host, "/path/to/destination_file").unwrap();
#![cfg_attr(feature = "remote-run", doc = " file.upload(&mut host, \"/path/to/local_file\", None);")]
//! file.set_owner(&mut host, "MyUser", "MyGroup").unwrap();
//! file.set_mode(&mut host, 644).unwrap();
//!
#![cfg_attr(feature = "remote-run", doc = " // Now let's upload another file and backup the original")]
#![cfg_attr(feature = "remote-run", doc = " file.upload(&mut host, \"/path/to/new_file\", Some(&vec![FileOpts::BackupExistingFile(\"_bk\".to_string())])).unwrap();")]
#![cfg_attr(feature = "remote-run", doc = "")]
#![cfg_attr(feature = "remote-run", doc = " // Your remote path now has two entries:")]
#![cfg_attr(feature = "remote-run", doc = " // \"/path/to/destination_file\" and \"/path/to/destination_file_bk\"")]
//! ```

pub mod ffi;

use {Host, Result};
use error::Error;
#[cfg(feature = "remote-run")]
use error::MissingFrame;
#[cfg(feature = "remote-run")]
use std::fs;
#[cfg(feature = "remote-run")]
use std::io::{Read, Seek, SeekFrom};
#[cfg(feature = "remote-run")]
use std::hash::{SipHasher, Hasher};
use target::Target;

#[cfg(feature = "remote-run")]
/// Size of each chunk in bytes
const CHUNK_SIZE: u16 = 10240;

/// Options for controlling file upload behaviour.
pub enum FileOpts {
    /// Backup any existing file during upload using the provided
    /// suffix.
    BackupExistingFile(String),
}

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
    path: String,
}

impl File {
    /// Create a new File struct.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::{File, Host};
    /// let mut host = Host::new();
    /// let file = File::new(&mut host, "/path/to/file");
    /// ```
    pub fn new(host: &mut Host, path: &str) -> Result<File> {
        if ! try!(Target::file_is_file(host, path)) {
            return Err(Error::Generic("Path is a directory".to_string()));
        }

        Ok(File {
            path: path.to_string(),
        })
    }

    /// Check if the file exists.
    pub fn exists(&self, host: &mut Host) -> Result<bool> {
        Target::file_exists(host, &self.path)
    }

    #[cfg(feature = "remote-run")]
    /// Upload a file to the managed host.
    pub fn upload(&self, host: &mut Host, local_path: &str, options: Option<&[FileOpts]>) -> Result<()> {
        let mut local_file = try!(fs::File::open(local_path));

        let length = try!(local_file.metadata()).len();
        let total_chunks = (length as f64 / CHUNK_SIZE as f64).ceil() as u64;

        let mut hasher = SipHasher::new();
        let mut buf = [0; CHUNK_SIZE as usize];

        for _ in 0..total_chunks {
            let bytes_read = try!(local_file.read(&mut buf));

            // Ensure that the chunk buffer only contains the number
            // of bytes read, rather than 1024.
            let (sized_buf, _) = buf.split_at(bytes_read);

            hasher.write(&sized_buf);
        }

        let mut download_sock = try!(host.send_file("file::upload", &self.path, hasher.finish(), length, total_chunks, options));

        // Ensure that the Agent acknowledged our request
        try!(host.recv_header());

        loop {
            try!(download_sock.recv_msg(0)); // File path

            let chunk_index: u64;

            match try!(download_sock.recv_string(0)).unwrap().as_ref() {
                "Chk" => {
                    if download_sock.get_rcvmore().unwrap() == false {
                        return Err(Error::Frame(MissingFrame::new("chunk", 2)));
                    }

                    chunk_index = try!(download_sock.recv_string(0)).unwrap().parse::<u64>().unwrap();
                },
                "Err" => {
                    if download_sock.get_rcvmore().unwrap() == false {
                        return Err(Error::Frame(MissingFrame::new("chunk", 2)));
                    }

                    return Err(Error::Agent(try!(download_sock.recv_string(0)).unwrap()));
                },
                "Done" => {
                    return Ok(());
                }
                _ => unreachable!(),
            }

            try!(local_file.seek(SeekFrom::Start(chunk_index * CHUNK_SIZE as u64)));
            let mut unsized_chunk = [0; CHUNK_SIZE as usize];
            let bytes_read = try!(local_file.read(&mut unsized_chunk));

            // Ensure that the chunk buffer only contains the number
            // of bytes read, rather than 1024.
            let (chunk, _) = unsized_chunk.split_at(bytes_read);

            try!(host.send_chunk(&self.path, chunk_index, &chunk));
        }
    }

    /// Delete the file.
    pub fn delete(&self, host: &mut Host) -> Result<()> {
        Target::file_delete(host, &self.path)
    }

    /// Move the file to a new path.
    pub fn mv(&mut self, host: &mut Host, new_path: &str) -> Result<()> {
        try!(Target::file_mv(host, &self.path, new_path));
        self.path = new_path.to_string();
        Ok(())
    }

    /// Copy the file to a new path.
    pub fn copy(&self, host: &mut Host, new_path: &str) -> Result<()> {
        Target::file_copy(host, &self.path, new_path)
    }

    /// Get the file's owner.
    pub fn get_owner(&self, host: &mut Host) -> Result<FileOwner> {
        Target::file_get_owner(host, &self.path)
    }

    // Set the file's owner.
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

pub trait FileTarget {
    fn file_is_file(host: &mut Host, path: &str) -> Result<bool>;
    fn file_exists(host: &mut Host, path: &str) -> Result<bool>;
    fn file_delete(host: &mut Host, path: &str) -> Result<()>;
    fn file_mv(host: &mut Host, path: &str, new_path: &str) -> Result<()>;
    fn file_copy(host: &mut Host, path: &str, new_path: &str) -> Result<()>;
    fn file_get_owner(host: &mut Host, path: &str) -> Result<FileOwner>;
    fn file_set_owner(host: &mut Host, path: &str, user: &str, group: &str) -> Result<()>;
    fn file_get_mode(host: &mut Host, path: &str) -> Result<u16>;
    fn file_set_mode(host: &mut Host, path: &str, mode: u16) -> Result<()>;
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
        let file = File::new(&mut host, "/path/to/file");
        assert!(file.is_ok());
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_ok() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_new_ok").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_new_ok").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let file = File::new(&mut host, "/tmp/test");
        assert!(file.is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_new_fail() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_new_fail").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_new_fail").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let file = File::new(&mut host, "/tmp/test");
        assert!(file.is_err());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_exists() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_exists").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::exists", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("0", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_exists").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let file = File::new(&mut host, "/tmp/test");
        assert!(file.is_ok());
        assert_eq!(file.unwrap().exists(&mut host).unwrap(), false);

        agent_mock.join().unwrap();
    }

    // XXX Need to mock FS before we can test upload effectively
    // #[test]
    // fn test_upload() {
    // }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_delete() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_delete").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::delete", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_delete").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let file = File::new(&mut host, "/tmp/test");
        assert!(file.is_ok());
        assert!(file.unwrap().delete(&mut host).is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_mv() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_mv").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/old", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::mv", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/old", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/new", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_mv").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let file = File::new(&mut host, "/tmp/old");
        assert!(file.is_ok());
        assert!(file.unwrap().mv(&mut host, "/tmp/new").is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_copy() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_copy").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/existing", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::copy", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/existing", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/new", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_copy").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let file = File::new(&mut host, "/tmp/existing");
        assert!(file.is_ok());
        assert!(file.unwrap().copy(&mut host, "/tmp/new").is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_owner() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_get_owner").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::get_owner", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Moo", zmq::SNDMORE).unwrap();
            agent_sock.send_str("123", zmq::SNDMORE).unwrap();
            agent_sock.send_str("Cow", zmq::SNDMORE).unwrap();
            agent_sock.send_str("456", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_get_owner").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let file = File::new(&mut host, "/tmp/test");
        assert!(file.is_ok());

        let owner = file.unwrap().get_owner(&mut host).unwrap();
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
        agent_sock.bind("inproc://test_set_owner").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::set_owner", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("Moo", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("Cow", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_set_owner").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let file = File::new(&mut host, "/tmp/test");
        assert!(file.is_ok());
        assert!(file.unwrap().set_owner(&mut host, "Moo", "Cow").is_ok());

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_get_mode() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_get_mode").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::get_mode", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("755", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_get_mode").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let file = File::new(&mut host, "/tmp/test");
        assert!(file.is_ok());
        assert_eq!(file.unwrap().get_mode(&mut host).unwrap(), 755);

        agent_mock.join().unwrap();
    }

    #[cfg(feature = "remote-run")]
    #[test]
    fn test_set_mode() {
        let mut ctx = zmq::Context::new();
        let mut agent_sock = ctx.socket(zmq::REP).unwrap();
        agent_sock.bind("inproc://test_set_mode").unwrap();

        let agent_mock = thread::spawn(move || {
            assert_eq!("file::is_file", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", zmq::SNDMORE).unwrap();
            agent_sock.send_str("1", 0).unwrap();

            assert_eq!("file::set_mode", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("/tmp/test", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), true);
            assert_eq!("644", agent_sock.recv_string(0).unwrap().unwrap());
            assert_eq!(agent_sock.get_rcvmore().unwrap(), false);

            agent_sock.send_str("Ok", 0).unwrap();
        });

        let mut sock = ctx.socket(zmq::REQ).unwrap();
        sock.set_linger(0).unwrap();
        sock.connect("inproc://test_set_mode").unwrap();

        let mut host = Host::test_new(None, Some(sock), None, None);

        let file = File::new(&mut host, "/tmp/test");
        assert!(file.is_ok());
        assert!(file.unwrap().set_mode(&mut host, 644).is_ok());

        agent_mock.join().unwrap();
    }
}
