use anyhow::{Context, Ok, Result};
use log::debug;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use super::command::{Request, Response};

pub struct Client {
    s: String,
}

impl Client {
    pub fn new(socket_path: String) -> Self {
        Self { s: socket_path }
    }

    pub async fn request(&self, req: Request) -> Result<Response> {
        debug!("sending request {:?}", req);
        let req: Vec<u8> = req.into();
        let mut stream = UnixStream::connect(&self.s)
            .await
            .context(format!("connect to {} failed", self.s))?;

        stream
            .write(&req)
            .await
            .context(format!("write request to {} failed", self.s))?;

        stream.shutdown().await?;
        debug!("write request done");

        let mut resp = Vec::<u8>::new();
        let mut buf = [0; 1024];
        while stream.read(&mut buf).await.context("read resp failed")? != 0 {
            debug!("read resp");
            resp.append(&mut buf.to_vec());
        }

        Ok(resp.into())
    }
}
