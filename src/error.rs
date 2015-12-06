use rustc_serialize::json;
use std::{convert, error, fmt, io, result};
use zmq;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// An error string returned from the host's Intecture Agent
    Agent(String),
    /// JSON decoder error
    JsonDecoder(json::DecoderError),
    /// Message frames missing in the response from host's Intecture Agent
    Frame(MissingFrame),
    /// IO error
    Io(io::Error),
    /// ZMQ error
    Zmq(zmq::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Agent(ref e) => write!(f, "Agent error: {}", e),
            Error::JsonDecoder(ref e) => write!(f, "JSON decoder error: {}", e),
            Error::Frame(ref e) => write!(f, "Missing frame {} in message: {}", e.order, e.name),
            Error::Io(ref e) => write!(f, "IO error: {}", e),
            Error::Zmq(ref e) => write!(f, "ZeroMQ error: {}", e),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Agent(ref e) => e,
            Error::JsonDecoder(ref e) => e.description(),
            Error::Frame(_) => "The Agent's reply was missing a part ('frame') of the expected message",
            Error::Io(ref e) => e.description(),
            Error::Zmq(ref e) => e.description(),
        }
    }
}

impl convert::From<json::DecoderError> for Error {
    fn from(err: json::DecoderError) -> Error {
        Error::JsonDecoder(err)
    }
}

impl convert::From<MissingFrame> for Error {
    fn from(err: MissingFrame) -> Error {
        Error::Frame(err)
    }
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl convert::From<zmq::Error> for Error {
    fn from(err: zmq::Error) -> Error {
        Error::Zmq(err)
    }
}

#[derive(Debug)]
pub struct MissingFrame {
    name: String,
    order: u8,
}

impl MissingFrame {
    pub fn new(name: &str, order: u8) -> MissingFrame {
        MissingFrame {
            name: name.to_string(),
            order: order,
        }
    }
}