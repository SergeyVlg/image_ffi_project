use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("Library error: {0}")]
    LibLoading(#[from] libloading::Error),
    #[error("validation error: {0}")]
    Validation(String)
}