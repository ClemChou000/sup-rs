use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum RotaterErr {
    #[error("crate rotater failed: [{0}]")]
    CreateRotaterFailed(String),

    #[error("path invalid {invalid_type} :{e}")]
    PathInvalid { invalid_type: String, e: String },
}
