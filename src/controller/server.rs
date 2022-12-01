use std::{
    io::{Read, Write},
    marker,
    os::unix::net::UnixStream,
    sync::Arc,
};

use log::error;

use super::{
    command::{CommandHandler, Request, Response, Transport, UnixSocketTp},
    error::ProcessErr,
};
use crate::run::threads::{with_num, ThreadsPool};

pub struct Server<T, P>
where
    T: Transport<P>,
    P: Read + Write,
{
    tsp: Arc<T>,
    tp: ThreadsPool,
    _x: marker::PhantomData<P>,
}

impl Server<UnixSocketTp, UnixStream> {
    pub fn new(socket_path: String) -> Result<Self, ProcessErr> {
        let tsp = Arc::new(UnixSocketTp::new(socket_path));
        let tp = ThreadsPool::new(vec![with_num(4)]);
        let cp_tsp = tsp.clone();
        if let Err(e) = tp.add_task(Box::new(move || cp_tsp.serve())) {
            return Err(ProcessErr::CreateServerFailed(e.to_string()));
        }
        Ok(Self {
            tsp,
            tp,
            _x: marker::PhantomData,
        })
    }

    // TODO: gracefully shutdown
    // TODO: log tracing
    pub fn run(&self) {
        loop {
            match self.tsp.read() {
                Ok(mut stream) => {
                    let mut v = Vec::<u8>::new();
                    if let Err(e) = stream.read_to_end(&mut v) {
                        error!("read stream failed: {}", e);
                        continue;
                    }
                    if let Err(e) = self.tp.add_task(Box::new(move || {
                        let resp: Vec<u8> = Self::handle_command(v.into()).into();
                        if let Err(e) = stream.write(&resp[..]) {
                            error!("write to stream failed: {}", e)
                        }
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

    fn start() -> Response {
        Response::new("start success".to_string(), None)
    }
    fn stop() -> Response {
        Response::new("stop success".to_string(), None)
    }
    fn restart() -> Response {
        Response::new("restart success".to_string(), None)
    }
    fn kill() -> Response {
        Response::new("kill success".to_string(), None)
    }
    fn reload() -> Response {
        Response::new("reload success".to_string(), None)
    }
    fn status() -> Response {
        Response::new("get status success".to_string(), None)
    }
    fn exit() -> Response {
        Response::new("exit success".to_string(), None)
    }
    fn unknown() -> Response {
        Response::new("unknown command".to_string(), None)
    }
}

impl CommandHandler for Server<UnixSocketTp, UnixStream> {
    fn handle_command(r: Request) -> Response {
        match r.cmd {
            super::command::Command::Start => Self::start(),
            super::command::Command::Stop => Self::stop(),
            super::command::Command::Restart => Self::restart(),
            super::command::Command::Kill => Self::kill(),
            super::command::Command::Reload => Self::reload(),
            super::command::Command::Exit => Self::exit(),
            super::command::Command::Status => Self::status(),
            super::command::Command::Unknown => Self::unknown(),
        }
    }
}
