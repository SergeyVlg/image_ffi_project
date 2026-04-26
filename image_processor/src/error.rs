use thiserror::Error;

#[derive(Debug, Error)]
pub(super) enum ProcessError {
    #[error("Library error: {0}")]
    LibLoading(#[from] libloading::Error),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("image processing error, code {0}")]
    ImageProcessing(i32),
    #[error("command line parsing error: {0}")]
    ParseError(#[from] clap_builder::error::Error),
    #[error("error: {0}")]
    StandardError(#[from] std::io::Error),
    #[error("image error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("corrupted image")]
    CorruptedImage,
}