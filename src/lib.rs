extern crate libc;
extern crate zmq;

pub mod command;
pub mod ffi;

pub use command::{Command, CommandResult};

#[derive(Debug)]
pub struct MissingFrameError {
    order: u8,
    name: &'static str
}
