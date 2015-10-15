use ::MissingFrameError;
use std::convert;
use zmq;

pub struct Command {
    pub cmd: String,
}

pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Command {
    pub fn new(cmd: &str) -> Command {
        Command {
            cmd: String::from(cmd),
        }
    }

    pub fn exec(&mut self, sock: &mut zmq::Socket) -> Result<CommandResult, CommandError> {
        try!(sock.send_str("command::exec", zmq::SNDMORE));
        try!(sock.send_str(&self.cmd, 0));

        let status = try!(sock.recv_string(0));
        if status.unwrap() == "Err" {
            if sock.get_rcvmore().unwrap() == false {
                return Err(CommandError::FrameError(MissingFrameError { order: 1, name: "err_msg" }));
            }

            return Err(CommandError::AgentError(try!(sock.recv_string(0)).unwrap()));
        }

        let exit_code = try!(sock.recv_msg(0)).as_str().unwrap().parse::<i32>().unwrap();

        if sock.get_rcvmore().unwrap() == false {
            return Err(CommandError::FrameError(MissingFrameError { order: 1, name: "stdout" }));
        }

        let stdout = try!(sock.recv_string(0)).unwrap();

        if sock.get_rcvmore().unwrap() == false {
            return Err(CommandError::FrameError(MissingFrameError { order: 1, name: "stderr" }));
        }

        let stderr = try!(sock.recv_string(0)).unwrap();

        Ok(CommandResult {
            exit_code: exit_code,
            stdout: stdout,
            stderr: stderr,
        })
    }
}

#[derive(Debug)]
pub enum CommandError {
    AgentError(String),
    FrameError(MissingFrameError),
    ZmqError(zmq::Error),
}

impl convert::From<zmq::Error> for CommandError {
	fn from(err: zmq::Error) -> CommandError {
		CommandError::ZmqError(err)
	}
}
