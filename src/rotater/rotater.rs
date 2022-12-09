use std::{
    ffi::OsStr,
    fs::File,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Result};
use log::info;

use crate::config::config::Log;

use super::error::RotaterErr::PathInvalid;

pub struct Rotater {
    file_path: String,

    need_compress: bool,
    need_merge_compress: bool,
    file: Arc<Mutex<File>>,
}

impl Rotater {
    pub fn new(conf: Log) -> Result<Self> {
        let f = match Self::create_file_from_path(&conf.path) {
            Err(e) => return Err(anyhow!(e)),
            Ok(f) => f,
        };

        Ok(Self {
            file_path: conf.path,
            file: Arc::new(Mutex::new(f)),
            need_compress: conf.compress,
            need_merge_compress: conf.merge_compressed,
        })
    }

    fn rotate(&mut self) -> Result<()> {
        let dir = Path::new(self.file_path.as_str())
            .parent()
            .unwrap_or(Path::new("/"));
        let filepath = Self::format_path_by_time(&self.file_path);
        let rotated_target = dir.join(&filepath);

        if let Err(e) = std::fs::rename(&self.file_path, &rotated_target) {
            return Err(anyhow!(PathInvalid {
                invalid_type: "rename file failed".to_string(),
                e: e.to_string()
            }));
        }

        self.file = match Self::create_file_from_path(&filepath) {
            Err(e) => {
                return Err(e);
            }
            Ok(f) => Arc::new(Mutex::new(f)),
        };

        info!(
            "rotated log {} to {}",
            filepath.to_str().unwrap_or("EMPTY"),
            rotated_target.to_str().unwrap_or("EMPTY")
        );

        thread::spawn(move || {});

        Ok(())
    }

    fn create_file_from_path<P: AsRef<Path>>(path: P) -> Result<File> {
        let dir = path.as_ref().parent().unwrap_or(Path::new("/"));
        if !dir.exists() {
            if let Err(e) = std::fs::create_dir_all(dir) {
                return Err(anyhow!(PathInvalid {
                    invalid_type: "create dir failed".to_string(),
                    e: e.to_string(),
                }));
            }
        }

        match std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .append(true)
            .open(path)
        {
            Ok(f) => Ok(f),
            Err(e) => Err(anyhow!(PathInvalid {
                invalid_type: "open file failed".to_string(),
                e: e.to_string(),
            })),
        }
    }

    fn format_path_by_time<P: AsRef<Path>>(origin_path: P) -> PathBuf {
        let now = chrono::Utc::now();
        let p = origin_path.as_ref().to_owned();
        let ext = p.extension().and_then(OsStr::to_str).unwrap_or_default();
        let stem = p.file_stem().and_then(OsStr::to_str).unwrap_or_default();

        Path::new(format!("{stem}-{}.{ext}", now.format("%Y%m%d%H%M%S")).as_str()).to_owned()
    }
}
