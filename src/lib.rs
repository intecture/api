extern crate libc;
extern crate zmq;

pub mod command;

pub use command::{Command, CommandResult};

#[derive(Debug)]
pub struct MissingFrameError {
    order: u8,
    name: &'static str
}