use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Ok, Result};
use dashmap::DashSet;
use log::{error, info};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::mpsc,
};

use crate::config::config::Log;

use super::error::RotaterErr::PathInvalid;

pub struct Rotater {
    // if recv none, finish
    signal_rotate_recv: mpsc::Receiver<Log>,
    signal_rotate_send: mpsc::Sender<Log>,
}

// rotater is singleton
impl Rotater {
    pub fn new(channel_length: usize) -> Result<Self> {
        let (send, recv) = mpsc::channel(channel_length);

        let s = Self {
            signal_rotate_send: send,
            signal_rotate_recv: recv,
        };

        tokio::spawn(async move { s.run().await });

        Ok(s)
    }

    // send log conf to backend rotater
    pub async fn add_rotate_task(&self, conf: Log) {
        if let Err(e) = self.signal_rotate_send.send(conf).await {
            error!("add rotate task failed: {}", e)
        }
    }

    async fn run(&mut self) {
        let running_path = Arc::new(DashSet::<String>::new());

        loop {
            let received_log = match self.signal_rotate_recv.recv().await {
                Some(r) => r,
                None => return,
            };

            if running_path.contains(received_log.path.as_str()) {
                continue;
            }
            // rotate time
            let running_path = running_path.clone();
            tokio::spawn(async move {
                running_path.insert(received_log.path.as_str().to_string());
                if let Err(e) = Self::rotate(&received_log).await {
                    error!("rotate with conf {} failed: {}", received_log, e);
                };
                running_path.remove(received_log.path.as_str());
            });
        }
    }

    async fn rotate(conf: &Log) -> Result<()> {
        let path = &conf.path;
        let dir = Path::new(path.as_str()).parent().unwrap_or(Path::new("/"));
        let filepath = Self::format_path_by_time(path);
        let rotated_target = dir.join(&filepath);

        if let Err(e) = tokio::fs::rename(path, &rotated_target).await {
            return Err(anyhow!(PathInvalid {
                invalid_type: "rename file failed".to_string(),
                e: e.to_string()
            }));
        }

        info!(
            "rotated log {} to {}",
            filepath.to_str().unwrap_or("EMPTY"),
            rotated_target.to_str().unwrap_or("EMPTY")
        );

        if conf.compress {
            Self::gzip(rotated_target).await?
        }

        Ok(())
    }

    async fn gzip<P: AsRef<Path>>(path: P) -> Result<()> {
        let file_input = File::open(path.as_ref()).await?;
        let mut input = BufReader::new(file_input);

        let path_output = match path.as_ref().as_os_str().to_str() {
            Some(p) => p,
            None => {
                return Err(anyhow!("convert path {:?} to str empty", path.as_ref()));
            }
        };

        let path_output = format!("{}.gz", path_output);
        let file_output = File::create(&path_output).await?;
        let mut writer = async_compression::tokio::write::GzipEncoder::new(file_output);

        let mut line = String::new();
        if input.read_line(&mut line).await.is_ok() {
            writer.write(line.as_bytes()).await?;
        }

        info!("compress file {path_output}");
        Ok(())
    }

    async fn create_file_from_path<P: AsRef<Path>>(path: P) -> Result<File> {
        let dir = path.as_ref().parent().unwrap_or(Path::new("/"));
        if !dir.exists() {
            tokio::fs::create_dir_all(dir).await?;
        }

        Ok(tokio::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .append(true)
            .open(path)
            .await?)
    }

    fn format_path_by_time<P: AsRef<Path>>(origin_path: P) -> PathBuf {
        let now = chrono::Utc::now();
        let p = origin_path.as_ref().to_owned();
        let ext = p.extension().and_then(OsStr::to_str).unwrap_or_default();
        let stem = p.file_stem().and_then(OsStr::to_str).unwrap_or_default();

        Path::new(format!("{stem}-{}.{ext}", now.format("%Y%m%d%H%M%S")).as_str()).to_owned()
    }
}
