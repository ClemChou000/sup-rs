use anyhow::{Context, Result};
use log::debug;
use tokio::{io::AsyncWriteExt, net::UnixStream};

use super::command::{Request, Response};

pub struct Client {
    s: String,
}

impl Client {
    pub fn new(socket_path: String) -> Self {
        Self { s: socket_path }
    }

    pub async fn request(&mut self, req: Request) -> Result<Response> {
        debug!("sending request {:?}", req);
        let req: Vec<u8> = req.into();
        UnixStream::connect(self.s)
            .await
            .context(format!("connect to {} failed", self.s))?
            .write(&req)
            .await
            .context(format!("write request to {} failed", self.s))?;

        Ok()
    }
}
