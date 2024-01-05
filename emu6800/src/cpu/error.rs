use emucore::mem::MemErrorTypes;
use thiserror::Error;

#[derive(Copy,Clone,Error,Debug)]
pub enum CpuErrKind {
    #[error(transparent)]
    Memory(#[from] MemErrorTypes)
}

pub type CpuResult<T> = Result<T,CpuErrKind>;
