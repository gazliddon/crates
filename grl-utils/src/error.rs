#![deny(unused_imports)]
use thiserror::Error;
use std::path::PathBuf;
use thin_vec::ThinVec;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum FileError {
    #[error("{0:?}")]
    Io(String),
    #[error("File not found {0:?} - searched {1:?}")]
    FileNotFound(PathBuf, ThinVec<PathBuf>),
    #[error("This shouldn't happen")]
    Placeholder,
    #[error( "Cannot read binary chunk. Range exceeds size of file {0}: file size is {1}, tried to grab up to {2}")]
    ReadingBinary(PathBuf, usize, usize),
    #[error("Trying to read size of {0:?}")]
    GettingSize(PathBuf),
}

pub type FResult<T> = Result<T,FileError>;

impl From<std::io::Error> for FileError {
    fn from(e: std::io::Error) -> Self {
        FileError::Io(e.to_string())
    }
}


