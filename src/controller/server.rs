use std::{
    io::{Read, Write},
    marker,
    os::unix::net::UnixStream,
};

use log::error;

use super::command::{CommandHandler, Request, Response, Transport, UnixSocketTp};
use crate::run::threads::{with_num, ThreadsPool};

pub struct Server<T, P>
where
    T: Transport<P>,
    P: Read + Write,
{
    tsp: T,
    tp: ThreadsPool,
    _x: marker::PhantomData<P>,
}

impl Server<UnixSocketTp, UnixStream> {
    pub fn new(socket_path: String) -> Self {
        let mut tsp = UnixSocketTp::new(socket_path);
        let mut tp = ThreadsPool::new(vec![with_num(4)]);
        tp.add_task(Box::new(|| tsp.serve()));
        Self {
            tsp,
            tp,
            _x: marker::PhantomData,
        }
    }

    // TODO: gracefully shutdown
    // TODO: log tracing
    fn run(self) {
        loop {
            match self.tsp.read() {
                Ok(stream) => {
                    let mut v = Vec::<u8>::new();
                    stream.read_to_end(&mut v);
                    if let Err(e) = self.tp.add_task(Box::new(|| {
                        let resp: Vec<u8> = self.handle_command(v.into()).into();
                        stream.write(&resp[..]);
                    })) {
                        error!("add task to threadspool failed: {}", e)
                    }
                }
                Err(e) => {
                    error!("read from threadspoll failed: {}", e);
                }
            }
        }
    }

    fn start(&self) -> Response {
        Response {
            message: "start success".to_string(),
            sup_pid: None,
        }
    }
    fn stop(&self) -> Response {
        Response {
            message: "stop success".to_string(),
            sup_pid: None,
        }
    }
    fn restart(&self) -> Response {
        Response {
            message: "restart success".to_string(),
            sup_pid: None,
        }
    }
    fn kill(&self) -> Response {
        Response {
            message: "kill success".to_string(),
            sup_pid: None,
        }
    }
    fn reload(&self) -> Response {
        Response {
            message: "reload success".to_string(),
            sup_pid: None,
        }
    }
    fn status(&self) -> Response {
        Response {
            message: "get status success".to_string(),
            sup_pid: None,
        }
    }
    fn exit(&self) -> Response {
        Response {
            message: "exit success".to_string(),
            sup_pid: None,
        }
    }
    fn unknown(&self) -> Response {
        Response {
            message: "unknown command".to_string(),
            sup_pid: None,
        }
    }
}

impl CommandHandler for Server<UnixSocketTp, UnixStream> {
    fn handle_command(&self, r: Request) -> Response {
        match r.cmd {
            super::command::Command::Start => self.start(),
            super::command::Command::Stop => self.stop(),
            super::command::Command::Restart => self.restart(),
            super::command::Command::Kill => self.kill(),
            super::command::Command::Reload => self.reload(),
            super::command::Command::Exit => self.exit(),
            super::command::Command::Status => self.status(),
            super::command::Command::Unknown => self.unknown(),
        }
    }
}
