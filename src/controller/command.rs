use std::{
    fs,
    io::{Read, Write},
    ops::Index,
    os::unix::net::{UnixListener, UnixStream},
    path::Path,
};

use log::error;

const BYTES_PER_PID: usize = 4;

use crossbeam::{
    channel::{Receiver, Sender},
    select,
};

use super::error::ProcessErr;

pub trait CommandHandler {
    fn handle_command(r: Request) -> Response;
}

pub trait Transport<T>
where
    T: Read + Write,
{
    fn connect(&mut self);
    fn serve(&self);
    fn read(&self) -> Result<T, ProcessErr>;
    fn write(self, v: Vec<u8>) -> Result<T, ProcessErr>;
}

pub struct UnixSocketTp {
    socket_path: String,
    stream: Option<UnixStream>,
    listen_recv: Option<Receiver<UnixStream>>,
    listen_send: Option<Sender<UnixStream>>,
}

impl UnixSocketTp {
    pub fn new(socket_path: String) -> Self {
        Self {
            socket_path,
            stream: None,
            listen_recv: None,
            listen_send: None,
        }
    }
}

impl Transport<UnixStream> for UnixSocketTp {
    fn connect(&mut self) {
        let stream = match UnixStream::connect(self.socket_path.as_str()) {
            Err(e) => panic!("connect to socket {} failed: {}", self.socket_path, e),
            Ok(stream) => stream,
        };
        self.stream = Some(stream);
    }

    fn serve(&self) {
        if Path::new(self.socket_path.as_str()).exists() {
            fs::remove_file(self.socket_path.as_str()).unwrap();
        }

        let listener = match UnixListener::bind(self.socket_path.as_str()) {
            Err(e) => panic!("bind socket {} failed: {}", self.socket_path, e),
            Ok(l) => l,
        };

        loop {
            let (unix_stream, _) = match listener.accept() {
                Ok((s, a)) => (s, a),
                Err(e) => {
                    error!("accept stream failed: {}", e);
                    continue;
                }
            };
            match &self.listen_send {
                Some(s) => match s.send(unix_stream) {
                    Ok(_) => {}
                    Err(e) => error!("send to channel failed: {}", e),
                },
                None => error!("listen channel must be inited before used"),
            }
        }
    }

    fn read(&self) -> Result<UnixStream, ProcessErr> {
        match &self.listen_recv {
            Some(rcv) => {
                select! {
                recv(rcv) -> msg => {
                    match msg {
                        Ok(t) => {
                            Ok(t)
                        },
                        Err(e) => {
                            Err(ProcessErr::ReadFromChannelFail(e.to_string()))
                        },
                    }
                }
                }
            }
            None => Err(ProcessErr::ChannelUsedBeforeInited("recv".to_string())),
        }
    }

    // TODO: convert self to &mut self?
    fn write(self, v: Vec<u8>) -> Result<UnixStream, ProcessErr> {
        match self.stream {
            Some(mut s) => match s.write(v.index(..)) {
                Ok(_) => {
                    if let Err(e) = s.shutdown(std::net::Shutdown::Write) {
                        return Err(ProcessErr::ShutdownStreamFailed(
                            "write".to_string(),
                            e.to_string(),
                        ));
                    }
                    Ok(s)
                }
                Err(e) => Err(ProcessErr::WriteToStreamFailed(e.to_string())),
            },
            None => Err(ProcessErr::StreamUsedBeforeInited("write".to_string())),
        }
    }
}

pub enum Command {
    Start,
    Stop,
    Restart,
    Kill,
    Reload,
    Status,
    Exit,

    Unknown,
}

pub struct Request {
    pub cmd: Command,
}

#[derive(Debug)]
pub struct Response {
    message: String,
    sup_pid: Option<u32>,
}

impl Response {
    pub fn new(message: String, sup_pid: Option<u32>) -> Self {
        Self { message, sup_pid }
    }
}

impl From<Vec<u8>> for Command {
    fn from(code: Vec<u8>) -> Self {
        if code.len() != 1 {
            return Self::Unknown;
        }
        match code.get(0).unwrap() {
            0 => Self::Start,
            1 => Self::Stop,
            2 => Self::Restart,
            3 => Self::Kill,
            4 => Self::Reload,
            5 => Self::Status,
            6 => Self::Exit,
            _ => Self::Unknown,
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
    fn marshal_msg(self) -> Vec<u8> {
        self.message.into_bytes()
    }

    fn marshal_sup_pid(&self) -> Vec<u8> {
        let pid = match self.sup_pid {
            Some(pid) => pid,
            None => 0,
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
            None => 0,
        };
        for (i, e) in v.into_iter().enumerate() {
            pid += (e as u32) << (8 * i);
        }

        self.sup_pid = Some(pid);
    }
}
