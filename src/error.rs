//! Error types related to serialization

use thiserror::Error as ThisError;

/// Result alias
pub type Result<T> = std::result::Result<T, self::Error>;

/// Serialization/deserialization error
#[derive(Debug, ThisError)]
pub enum Error {
    /// A value using an unsupported Serde data type was supplied.
    ///
    /// Because the encoding is not self-describing, this error occurs for
    /// complex, variable-sized containers such as maps, sequences, and enums
    /// with struct or tuple variants.
    #[error("Unsupported data type")]
    UnsupportedType,

    /// Error decoding bytes as UTF-8 string.
    #[error("UTF-8 decoding error: {0}")]
    Utf8StringDecode(#[from] std::str::Utf8Error),

    /// Error decoding a component.
    ///
    /// This occurs when the encoded values do not match the given type, for
    /// example, attempting to decode an integer that isn't an integer.
    #[error("Component data error on component {0}")]
    Data(String),

    /// Error on the formatting of the strkey encoding.
    ///
    /// This occurs when the encoded values do not match the layout of the
    /// given types.
    #[error("Encoding syntax error")]
    Syntax,

    /// Standard IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Some other Serde error.
    #[error("Other error: {0}")]
    Other(String),
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Other(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Other(msg.to_string())
    }
}
