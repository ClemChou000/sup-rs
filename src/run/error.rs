use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum RunError {
    #[error("send task failed: [{0}]")]
    SendTaskFail(String),
}
