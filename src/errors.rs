/// A specialized [`Result`] type for this crate.
///
/// [`Result`]: ::std::result::Result
use std::fmt;

pub type Result<T> = ::std::result::Result<T, Error>;

/// Error categories from the `plthook` library.
#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    FileNotFound,
    InvalidFileFormat,
    FunctionNotFound,
    InvalidArgument,
    OutOfMemory,
    InternalError,
    NotImplemented,
    UnknownError(libc::c_int),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::FileNotFound => fmt.write_str("FileNotFound"),
            ErrorKind::InvalidFileFormat => fmt.write_str("InvalidFileFormat"),
            ErrorKind::FunctionNotFound => fmt.write_str("FunctionNotFound"),
            ErrorKind::InvalidArgument => fmt.write_str("InvalidArgument"),
            ErrorKind::OutOfMemory => fmt.write_str("OutOfMemory"),
            ErrorKind::InternalError => fmt.write_str("InternalError"),
            ErrorKind::NotImplemented => fmt.write_str("NotImplemented"),
            ErrorKind::UnknownError(x) => write!(fmt, "Error#{}", x),
        }
    }
}

impl From<libc::c_int> for ErrorKind {
    fn from(value: libc::c_int) -> Self {
        // These values are from the plthook.h file.
        match value {
            1 => ErrorKind::FileNotFound,
            2 => ErrorKind::InvalidFileFormat,
            3 => ErrorKind::FunctionNotFound,
            4 => ErrorKind::InvalidArgument,
            5 => ErrorKind::OutOfMemory,
            6 => ErrorKind::InternalError,
            7 => ErrorKind::NotImplemented,
            _ => ErrorKind::UnknownError(value),
        }
    }
}

/// Errors from the `plthook` library.
#[derive(Clone, Debug)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, message: String) -> Error {
        Error { kind, message }
    }

    /// Returns the kind of this error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Returns the message of this error.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "plthook error: {}: {}", self.kind, self.message)
    }
}

impl std::error::Error for Error {}
