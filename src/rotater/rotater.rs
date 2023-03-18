use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{anyhow, Context, Ok, Result};
use chrono::{DateTime, Days, TimeZone, Utc};
use dashmap::DashSet;
use log::{error, info};
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::mpsc,
};

use crate::config::config::Log;

const TIME_FORMAT: &str = "%Y%m%d%H%M%S";

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
        let rotated_filename = Self::format_path_by_time(path, Utc::now());
        let rotated_target = dir.join(rotated_filename);

        tokio::fs::rename(path, &rotated_target).await?;

        info!(
            "rotated log {} to {}",
            path.as_str(),
            rotated_target.to_str().unwrap_or("EMPTY")
        );

        if conf.compress {
            Self::gzip_from_path(rotated_target).await?;
        }

        Self::clean_extra_backups(
            dir,
            Path::new(path).file_stem().unwrap(),
            Utc::now()
                .checked_sub_days(Days::new(conf.max_days))
                .unwrap(),
            conf.max_backups,
        )
        .await
        .context("clean extra backups failed")?;

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

    /// deadline = current time - roatate duration
    /// origin_filename = {test}-20230317200700.log.gz
    async fn clean_extra_backups(
        dir: &Path,
        origin_filename: &OsStr,
        deadline: DateTime<Utc>,
        max_backups: usize,
    ) -> Result<()> {
        let mut entrys = tokio::fs::read_dir(dir).await?;
        let mut filename_vec = Vec::new();
        while let Some(entry) = entrys.next_entry().await? {
            if entry.file_type().await?.is_dir()
                || entry
                    .file_name()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .starts_with(origin_filename.to_str().unwrap())
            {
                continue;
            }
            let t =
                Self::parse_path_to_time(entry.file_name()).context("parse path to time failed")?;
            if t < deadline {
                // remove file
                tokio::fs::remove_file(dir.join(entry.file_name()))
                    .await
                    .context("remove backup file failed")?;
                continue;
            }
            filename_vec.push(entry.file_name());
        }

        if filename_vec.len() <= max_backups {
            return Ok(());
        }

        let topk_filename = top_k(&mut filename_vec, max_backups);
        let topk_time = Self::parse_path_to_time(topk_filename)?;
        for n in &filename_vec {
            if Self::parse_path_to_time(n).context("parse topk time failed")? < topk_time {
                tokio::fs::remove_file(dir.join(n))
                    .await
                    .context("remove topk files failed")?;
            }
        }
        Ok(())
    }

    fn parse_path_to_time<P: AsRef<Path>>(path: P) -> Result<DateTime<Utc>> {
        let ts = path
            .as_ref()
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap()
            .split('-')
            .last()
            .unwrap();
        Utc.datetime_from_str(ts, TIME_FORMAT)
            .context("convert str to time failed")
    }

    fn format_path_by_time<P: AsRef<Path>>(origin_path: P, t: DateTime<Utc>) -> PathBuf {
        let ext = origin_path
            .as_ref()
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        let stem = origin_path
            .as_ref()
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or_default();
        Path::new(format!("{}-{}.{}", stem, t.format(TIME_FORMAT), ext).as_str()).to_owned()
    }
}

// k in range [1, len(v)]
fn top_k<T: PartialOrd>(v: &mut Vec<T>, k: usize) -> &T {
    quick_select(v, 0, v.len() - 1, k)
}

fn quick_select<T: PartialOrd>(v: &mut Vec<T>, left: usize, right: usize, k: usize) -> &T {
    if left == right {
        return &v[left];
    }

    let mut mid_index = left;

    let (mut i, mut j) = (left, right + 1);
    while i < j {
        if i != left {
            i += 1;
        }
        while &v[i] < &v[mid_index] {
            i += 1;
        }
        j -= 1;
        while &v[j] > &v[mid_index] {
            j -= 1;
        }
        if i < j {
            v.swap(i, j);
            if i == mid_index {
                mid_index = j
            }
        }
    }

    let sl = j - left + 1;
    if k <= sl {
        quick_select(v, left, j, k)
    } else {
        quick_select(v, j + 1, right, k - sl)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use chrono::{TimeZone, Utc};

    use super::*;

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

    #[tokio::test]
    async fn async_parse_and_format_time_test() {
        let t = Utc.with_ymd_and_hms(2023, 3, 17, 20, 07, 00).unwrap();
        let path = Rotater::format_path_by_time("test.log", t);

        assert_eq!("test-20230317200700.log", path.to_str().unwrap());
        assert_eq!(Rotater::parse_path_to_time(path.as_path()).unwrap(), t);
    }

    #[test]
    fn quick_select_test() {
        let mut v = vec![1, 4, 8, 3, 2, 5];
        let lenv = v.len() - 1;
        assert_eq!(quick_select(&mut v, 0, lenv, 1), &1);
        assert_eq!(quick_select(&mut v, 0, lenv, 2), &2);
        assert_eq!(quick_select(&mut v, 0, lenv, 3), &3);
        assert_eq!(quick_select(&mut v, 0, lenv, 4), &4);
        assert_eq!(quick_select(&mut v, 0, lenv, 5), &5);
        assert_eq!(quick_select(&mut v, 0, lenv, 6), &8);
        assert_eq!(quick_select(&mut v, 0, lenv, 7), &8);
    }
}
