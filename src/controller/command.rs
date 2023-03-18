use std::fmt::Display;

use clap::Subcommand;

const BYTES_PER_PID: usize = 4;

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "start program asynchronously")]
    Start,
    #[command(about = "stop program asynchronously")]
    Stop,
    #[command(about = "restart program asynchronously")]
    Restart,
    #[command(about = "kill program and all child processes")]
    Kill,
    #[command(about = "reload program")]
    Reload,
    #[command(about = "print status of program")]
    Status,
    #[command(about = "exit the sup daemon and the process asynchronously")]
    Exit,
}

impl From<&str> for Option<Command> {
    fn from(c: &str) -> Self {
        match c {
            "start" => Some(Command::Start),
            "stop" => Some(Command::Stop),
            "restart" => Some(Command::Restart),
            "kill" => Some(Command::Kill),
            "reload" => Some(Command::Reload),
            "status" => Some(Command::Status),
            "exit" => Some(Command::Exit),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub cmd: Command,
}

#[derive(Debug)]
pub struct Response {
    message: String,
    sup_pid: Option<u32>,
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.sup_pid {
            Some(pid) => write!(f, "{}, pid is {}", self.message, pid),
            None => write!(f, "{}", self.message),
        }
    }
}

impl Response {
    pub fn new(message: String, sup_pid: Option<u32>) -> Self {
        Self { message, sup_pid }
    }
}

impl From<Vec<u8>> for Option<Command> {
    fn from(code: Vec<u8>) -> Self {
        if code.len() != 1 {
            return None;
        }
        match code.get(0).unwrap() {
            0 => Some(Self::Start),
            1 => Some(Self::Stop),
            2 => Some(Self::Restart),
            3 => Some(Self::Kill),
            4 => Some(Self::Reload),
            5 => Some(Self::Status),
            6 => Some(Self::Exit),
            _ => Some(Self::Unknown),
        }
    }
}

impl From<Command> for Vec<u8> {
    fn from(c: Command) -> Self {
        match c {
            Command::Start => vec![0],
            Command::Stop => vec![1],
            Command::Restart => vec![2],
            Command::Kill => vec![3],
            Command::Reload => vec![4],
            Command::Status => vec![5],
            Command::Exit => vec![6],
            Command::Unknown => vec![7],
        }
    }
}

impl From<Vec<u8>> for Request {
    fn from(v: Vec<u8>) -> Self {
        Self { cmd: v.into() }
    }
}

impl From<Request> for Vec<u8> {
    fn from(r: Request) -> Self {
        r.cmd.into()
    }
}

impl From<Response> for Vec<u8> {
    fn from(r: Response) -> Self {
        let mut res = Vec::<u8>::new();
        res.append(&mut r.marshal_sup_pid());
        res.append(&mut r.marshal_msg());
        res
    }
}

impl From<Vec<u8>> for Response {
    fn from(v: Vec<u8>) -> Self {
        let mut s = Self {
            message: String::new(),
            sup_pid: Some(0),
        };
        s.unmarshal_sup_pid(v.index(..BYTES_PER_PID).to_vec());
        s.unmarshal_msg(v.index(BYTES_PER_PID..).to_vec());
        s
    }
}

impl Response {
    const NONE_PID: u32 = 0;
    fn marshal_msg(self) -> Vec<u8> {
        self.message.into_bytes()
    }

    fn marshal_sup_pid(&self) -> Vec<u8> {
        let pid = match self.sup_pid {
            Some(pid) => pid,
            None => Self::NONE_PID,
        };

        vec![
            (pid >> 24) as u8,
            (pid >> 16) as u8,
            (pid >> 8) as u8,
            pid as u8,
        ]
    }

    fn unmarshal_msg(&mut self, v: Vec<u8>) {
        self.message = String::from_utf8(v).unwrap();
    }

    fn unmarshal_sup_pid(&mut self, v: Vec<u8>) {
        let mut pid = match self.sup_pid {
            Some(pid) => pid,
            None => return,
        };
        for (i, e) in v.into_iter().enumerate() {
            pid += (e as u32) << (8 * i);
        }

        if pid == Self::NONE_PID {
            self.sup_pid = None;
            return;
        }

        self.sup_pid = Some(pid);
    }
}
