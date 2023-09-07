#[derive(thiserror::Error, Debug, Clone)]
pub enum SourceErrorType {
    #[error("Sort this out gaz")]
    Misc,
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Id not found: {0}")]
    IdNotFound(u64),

    #[error("IO Error : {0}")]
    Io(String)

    // #[error(transparent)]
    // Json {
    //     #[from]
    //     source: serde_json::Error,
    // },
}

pub type SResult<T> = Result<T,SourceErrorType>;


