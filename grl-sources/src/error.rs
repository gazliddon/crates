#![deny(unused_imports)]
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum SourceErrorType {
    #[error(transparent)]
    FileError(#[from] grl_utils::FileError),
    #[error("Sort this out gaz")]
    Misc,
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Id not found: {0}")]
    IdNotFound(u64),

    #[error("IO Error : {0}")]
    Io(String)
}

pub type SResult<T> = Result<T,SourceErrorType>;

