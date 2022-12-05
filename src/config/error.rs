use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum Error {
    #[error("read file failed: [{0}]")]
    ReadFileError(String),
    #[error("config format invalid failed: [{0}]")]
    FormatCheckError(String),
}
