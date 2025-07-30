use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse DNS message: {0}")]
    ParseError(String),

    #[error("Invalid resolver address: {0}")]
    InvalidResolverAddress(String),

    #[error("DNS query failed: {0}")]
    QueryFailed(String),
}

// Helper to convert nom's complex error type into our simple one.
impl<'a> From<nom::Err<nom::error::Error<&'a [u8]>>> for Error {
    fn from(err: nom::Err<nom::error::Error<&'a [u8]>>) -> Self {
        Error::ParseError(err.to_string())
    }
}