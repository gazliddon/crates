use emucore::mem::MemErrorTypes;
use thiserror::Error;

#[derive(Clone, Error, Debug)]
pub enum CpuErrKind {
    #[error(transparent)]
    Memory(#[from] MemErrorTypes),
    #[error("instruction effect check failed: {0}")]
    Effects(String),
}

pub type CpuResult<T> = Result<T, CpuErrKind>;
