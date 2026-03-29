use std::io;
use std::fmt;

/// Custom error type for the Blossom chat application
#[derive(Debug)]
pub enum BlossomError {
    /// I/O related errors
    Io(io::Error),
    /// Network related errors
    Network(String),
    /// Configuration errors
    Config(String),
    /// Yggdrasil process errors
    Yggdrasil(String),
    /// Encryption/Decryption errors
    Crypto(String),
    /// Invalid input or data format
    InvalidData(String),
    /// System command execution errors
    Command(String),
}

impl fmt::Display for BlossomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlossomError::Io(err) => write!(f, "I/O error: {}", err),
            BlossomError::Network(msg) => write!(f, "Network error: {}", msg),
            BlossomError::Config(msg) => write!(f, "Configuration error: {}", msg),
            BlossomError::Yggdrasil(msg) => write!(f, "Yggdrasil error: {}", msg),
            BlossomError::Crypto(msg) => write!(f, "Crypto error: {}", msg),
            BlossomError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            BlossomError::Command(msg) => write!(f, "Command error: {}", msg),
        }
    }
}

impl std::error::Error for BlossomError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BlossomError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for BlossomError {
    fn from(err: io::Error) -> Self {
        BlossomError::Io(err)
    }
}

impl From<base64::DecodeError> for BlossomError {
    fn from(err: base64::DecodeError) -> Self {
        BlossomError::InvalidData(format!("Base64 decode error: {}", err))
    }
}

impl From<std::string::FromUtf8Error> for BlossomError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        BlossomError::InvalidData(format!("UTF-8 conversion error: {}", err))
    }
}
impl From<rand::rngs::SysError> for BlossomError {
    fn from(err: rand::rngs::SysError) -> Self {
        BlossomError::Crypto(format!("Random generation error: {}", err))
    }
}

/// Result type alias for the Blossom application
pub type Result<T> = std::result::Result<T, BlossomError>;
