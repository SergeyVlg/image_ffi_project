use std::ffi::NulError;
use thiserror::Error;

#[derive(Debug, Error)]
pub(super) enum ProcessError {
    #[error("Plugin library loading error: {0}")]
    LibLoading(#[from] libloading::Error),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("params error: contains nul byte")]
    ParamsFormatError(#[from] NulError),
    #[error("image processing error, code {0}")]
    ImageProcessing(i32),
    #[error(transparent)]
    ParseError(#[from] clap_builder::error::Error),
    #[error("error: {0}")]
    StandardError(#[from] std::io::Error),
    #[error("image error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("corrupted image")]
    CorruptedImage,
}