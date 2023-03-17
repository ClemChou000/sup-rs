use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Context, Ok, Result};
use dashmap::DashSet;
use log::{error, info};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::mpsc,
};

use crate::config::config::Log;

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

        Ok(s)
    }

    // send log conf to backend rotater
    pub async fn add_rotate_task(&self, conf: Log) {
        if let Err(e) = self.signal_rotate_send.send(conf).await {
            error!("add rotate task failed: {}", e)
        }
    }

    pub async fn run(&mut self) {
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

        tokio::fs::File::create(path).await?;

        let dir = Path::new(path.as_str()).parent().unwrap_or(Path::new("/"));
        let rotated_filename = Self::format_path_by_time(path.as_ref());
        let rotated_target = dir.join(rotated_filename);

        tokio::fs::rename(path, &rotated_target).await?;

        info!(
            "rotated log {} to {}",
            path.as_str(),
            rotated_target.to_str().unwrap_or("EMPTY")
        );

        if conf.compress {
            Self::gzip_from_path(rotated_target).await?
        }

        Ok(())
    }

    /// gzip file by path to file.gz and delete raw file
    async fn gzip_from_path<P: AsRef<Path>>(path: P) -> Result<()> {
        let file_input = File::open(path.as_ref())
            .await
            .context("open input file failed")?;
        let mut input = BufReader::new(file_input);

        let path_output = match path.as_ref().as_os_str().to_str() {
            Some(p) => p,
            None => {
                return Err(anyhow!("convert path {:?} to str empty", path.as_ref()));
            }
        };

        let path_output = format!("{}.gz", path_output);
        let mut file_output = File::create(&path_output)
            .await
            .context("create output file failed")?;

        Self::gzip(&mut input, &mut file_output)
            .await
            .context("gzip file failed")?;

        tokio::fs::remove_file(path).await?;

        info!("compress file {path_output}");
        Ok(())
    }

    async fn gzip<W: AsyncWrite + Unpin, R: AsyncRead + Unpin>(
        input: &mut R,
        output: &mut W,
    ) -> Result<()> {
        let mut writer = async_compression::tokio::write::GzipEncoder::new(output);

        tokio::io::copy(input, &mut writer)
            .await
            .context("gzip rotated log failed")?;

        writer.shutdown().await?;
        writer.into_inner().flush().await?;
        Ok(())
    }

    async fn clean_extra_backups(dir: &Path) -> Result<()> {
        let mut entrys = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entrys.next_entry().await? {
            entry.file_name();
        }
        Ok(())
    }

    fn format_path_by_time<'a>(origin_path: &'a Path) -> &'a Path {
        origin_path.into()
    }
}

struct FormatPath<'a> {
    ext: &'a str,
    stem: &'a str,
}

impl<'a> From<&'a Path> for FormatPath<'a> {
    fn from(value: &'a Path) -> Self {
        let ext = value
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        let stem = value
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        Self { ext, stem }
    }
}

impl<'a> Into<PathBuf> for FormatPath<'a> {
    fn into(self) -> PathBuf {
        let now = chrono::Utc::now();
        format!("{}-{}.{}", self.stem, now.format("%Y%m%d%H%M%S"), self.ext)
            .to_owned()
            .into()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::Rotater;

    #[tokio::test]
    async fn async_gzip_test() {
        let mut input = Cursor::new(['1' as u8; 10]);
        let mut output = Cursor::new(Vec::with_capacity(10));
        Rotater::gzip(&mut input, &mut output).await.unwrap();

        assert_eq!(
            output.into_inner(),
            vec![
                31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 50, 52, 132, 1, 0, 0, 0, 0, 255, 255, 3, 0, 49,
                250, 88, 179, 10, 0, 0, 0
            ]
        );
    }
}
