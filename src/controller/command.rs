/// vec<u8> is standard data structure for creating request && response
use std::{fmt::Display, ops::Index};

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

#[derive(Debug)]
pub struct Request {
    pub cmd: Option<Command>,
}

impl From<Vec<u8>> for Request {
    fn from(code: Vec<u8>) -> Self {
        if code.len() != 1 {
            return Self { cmd: None };
        }
        Self {
            cmd: match code.get(0).unwrap() {
                0 => Some(Command::Start),
                1 => Some(Command::Stop),
                2 => Some(Command::Restart),
                3 => Some(Command::Kill),
                4 => Some(Command::Reload),
                5 => Some(Command::Status),
                6 => Some(Command::Exit),
                _ => None,
            },
        }
    }
}

impl From<Request> for Vec<u8> {
    fn from(c: Request) -> Self {
        if let Some(cmd) = c.cmd {
            match cmd {
                Command::Start => vec![0],
                Command::Stop => vec![1],
                Command::Restart => vec![2],
                Command::Kill => vec![3],
                Command::Reload => vec![4],
                Command::Status => vec![5],
                Command::Exit => vec![6],
            }
        } else {
            vec![7]
        }
    }
}
#[derive(Debug)]
pub struct Response {
    message: String,
    sup_pid: Option<u32>,
}

impl Response {
    const INVALID_PID: u32 = 0;

    pub fn new(message: String, sup_pid: Option<u32>) -> Self {
        Self { message, sup_pid }
    }

    fn marshal_msg(self) -> Vec<u8> {
        self.message.into_bytes()
    }

    fn marshal_sup_pid(&self) -> Vec<u8> {
        let pid = match self.sup_pid {
            Some(pid) => pid,
            None => Self::INVALID_PID,
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

        if pid == Self::INVALID_PID {
            self.sup_pid = None;
            return;
        }

        self.sup_pid = Some(pid);
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

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.sup_pid {
            Some(pid) => write!(f, "{}, pid is {}", self.message, pid),
            None => write!(f, "{}", self.message),
        }
    }
}
