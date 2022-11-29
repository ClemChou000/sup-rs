use std::{
    io::{Read, Write},
    marker,
    os::unix::net::UnixStream,
};

use super::{
    command::{Request, Response, Transport, UnixSocketTp},
    error::ProcessErr,
};

struct Client<T, P>
where
    T: Transport<P>,
    P: Read + Write,
{
    tsp: T,
    _x: marker::PhantomData<P>,
}

impl Client<UnixSocketTp, UnixStream> {
    pub fn new(socket_path: String) -> Self {
        let mut tsp = UnixSocketTp::new(socket_path);
        tsp.connect();
        Self {
            tsp,
            _x: marker::PhantomData,
        }
    }

    pub fn start(self) -> Result<Response, ProcessErr> {
        self.request(Request {
            cmd: super::command::Command::Start,
        })
    }

    pub fn stop(self) -> Result<Response, ProcessErr> {
        self.request(Request {
            cmd: super::command::Command::Stop,
        })
    }

    pub fn restart(self) -> Result<Response, ProcessErr> {
        self.request(Request {
            cmd: super::command::Command::Restart,
        })
    }

    pub fn kill(self) -> Result<Response, ProcessErr> {
        self.request(Request {
            cmd: super::command::Command::Kill,
        })
    }

    pub fn reload(self) -> Result<Response, ProcessErr> {
        self.request(Request {
            cmd: super::command::Command::Reload,
        })
    }

    pub fn status(self) -> Result<Response, ProcessErr> {
        self.request(Request {
            cmd: super::command::Command::Status,
        })
    }

    pub fn exit(self) -> Result<Response, ProcessErr> {
        self.request(Request {
            cmd: super::command::Command::Exit,
        })
    }

    fn request(self, req: Request) -> Result<Response, ProcessErr> {
        match self.tsp.write(req.into()) {
            Ok(mut resp_stream) => {
                let mut v = Vec::<u8>::new();
                if let Err(e) = resp_stream.read_to_end(&mut v) {
                    return Err(ProcessErr::ReadFromStreamFail(e.to_string()));
                };
                Ok(v.into())
            }
            Err(e) => Err(e),
        }
    }
}
