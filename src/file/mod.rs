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
#![cfg_attr(feature = "remote-run", doc = " host.connect(\"127.0.0.1\", 7101).unwrap();")]
//! ```
//!
//! Now ...
//!
//! ```no_run
//! # use inapi::{Host, File};
//! # let mut host = Host::new();
//! ```

// pub mod ffi;

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
const CHUNK_SIZE: u16 = 10240;

pub enum FileOpts {
    BackupExistingFile(String),
}

/// Container for operating on a file.
pub struct File {
    path: String,
}

impl File {
    /// Create a new File struct.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use inapi::File;
    /// let file = File::new(&mut host, "/path/to/file");
    /// ```
    pub fn new(host: &mut Host, path: &str) -> Result<File> {
        if ! try!(Target::file_is_file(host, path)) {
            return Err(Error::Generic("Path is not a file".to_string()));
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

    /// Get the file's permissions mode.
    pub fn get_mode(&self, host: &mut Host) -> Result<u16> {
        Target::file_get_mode(host, &self.path)
    }

    /// Set the file's permissions mode.
    pub fn set_mode(&self, host: &mut Host, mode: u16) -> Result<()> {
        Target::file_set_mode(host, &self.path, mode)
    }
}

pub trait FileTarget {
    fn file_is_file(host: &mut Host, path: &str) -> Result<bool>;
    fn file_exists(host: &mut Host, path: &str) -> Result<bool>;
    fn file_delete(host: &mut Host, path: &str) -> Result<()>;
    fn file_get_mode(host: &mut Host, path: &str) -> Result<u16>;
    fn file_set_mode(host: &mut Host, path: &str, mode: u16) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use Host;
    use super::*;

    #[test]
    fn test_() {

    }
}
