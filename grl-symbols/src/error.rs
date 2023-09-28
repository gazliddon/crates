use thiserror::Error;

#[derive(PartialEq, Eq, Clone, Debug, Error)]
pub enum SymbolError {
    #[error("Symbol already define")]
    AlreadyDefined,
    #[error("Invalid scope")]
    InvalidScope,
    #[error("Symbol not found")]
    NotFound,
    #[error("Symbol has no value")]
    NoValue,
    #[error("Invalid symbol id")]
    InvalidId,
    #[error("Symbol not found : hit scope barrier")]
    HitScopeBarrier,
}
