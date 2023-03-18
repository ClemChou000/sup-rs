use anyhow::{Context, Result};
use log::{error, info};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

use super::command::{Request, Response};

pub struct Server {
    listener: UnixListener,
}

impl Server {
    pub fn new(socket_path: String) -> Result<Self> {
        let listener = UnixListener::bind(socket_path)
            .context(format!("bind socket path {socket_path} failed"))?;

        Ok(Self { listener })
    }

    pub async fn run(&self) {
        let buf = Vec::<u8>::new();
        loop {
            match self.listener.accept().await {
                Ok((mut socket, addr)) => {
                    info!("accept socket from {:?}", addr);
                    if let Err(e) = Self::handle_socket(&mut socket).await {
                        error!("handle socket failed: {e}")
                    };
                }
                Err(e) => {
                    error!("accept socket failed: {e}");
                }
            }
        }
    }

    async fn handle_socket(socket: &mut UnixStream) -> Result<()> {
        let mut buf = String::new();
        socket.read_to_string(&mut buf).await?;

        let req: Request = buf.as_bytes().to_vec().into();
        let res: Vec<u8> = Self::handle_command(req).into();
        socket.write(&res).await?;
        Ok(())
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

    fn handle_command(r: Request) -> Response {
        if let Some(cmd) = r.cmd {
            match cmd {
                super::command::Command::Start => Self::start(),
                super::command::Command::Stop => Self::stop(),
                super::command::Command::Restart => Self::restart(),
                super::command::Command::Kill => Self::kill(),
                super::command::Command::Reload => Self::reload(),
                super::command::Command::Exit => Self::exit(),
                super::command::Command::Status => Self::status(),
            }
        } else {
            Self::unknown()
        }
    }
}
