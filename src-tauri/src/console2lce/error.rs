use std::fmt;
use std::io;
#[derive(Debug)]
pub enum ConversionError {
    Io(io::Error),
    InvalidFormat(String),
    UnsupportedVersion(String),
    NbtError(String),
    MissingData(String),
    DecompressionFailed(String),
    CompressionFailed(String),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConversionError::Io(e) => write!(f, "IO error: {}", e),
            ConversionError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            ConversionError::UnsupportedVersion(s) => write!(f, "Unsupported version: {}", s),
            ConversionError::NbtError(s) => write!(f, "NBT error: {}", s),
            ConversionError::MissingData(s) => write!(f, "Missing data: {}", s),
            ConversionError::DecompressionFailed(s) => write!(f, "Decompression failed: {}", s),
            ConversionError::CompressionFailed(s) => write!(f, "Compression failed: {}", s),
        }
    }
}

impl std::error::Error for ConversionError {}
impl From<io::Error> for ConversionError {
    fn from(e: io::Error) -> Self {
        ConversionError::Io(e)
    }
}
