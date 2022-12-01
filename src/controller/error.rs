use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ProcessErr {
    #[error("read from channel failed: [{0}]")]
    ReadFromChannelFail(String),

    #[error("read from stream failed: [{0}]")]
    ReadFromStreamFail(String),

    #[error("write to stream failed: [{0}]")]
    WriteToStreamFailed(String),

    #[error("channel [{0}] before inited")]
    ChannelUsedBeforeInited(String),

    #[error("stream [{0}] before inited")]
    StreamUsedBeforeInited(String),

    #[error("shutdown stream in [{0}] side failed: [{1}]")]
    ShutdownStreamFailed(String, String),

    #[error("crate server failed: [{0}]")]
    CreateServerFailed(String),
}
