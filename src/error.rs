use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustyStorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("FFmpeg error: {0}")]
    FFmpeg(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Decoding error: {0}")]
    Decoding(String),
}

pub type Result<T> = std::result::Result<T, RustyStorageError>; 