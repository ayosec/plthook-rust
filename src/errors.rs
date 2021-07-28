/// A specialized [`Result`] type for this crate.
///
/// [`Result`]: ::std::result::Result

pub type Result<T> = ::std::result::Result<T, Error>;

/// Errors from the `plthook` library.
#[derive(Clone, Copy, Debug)]
pub enum Error {
    FileNotFound,
    InvalidFileFormat,
    FunctionNotFound,
    InvalidArgument,
    OutOfMemory,
    InternalError,
    NotImplemented,
    UnknownError(libc::c_int),
}

impl From<libc::c_int> for Error {
    fn from(value: libc::c_int) -> Self {
        // These values are from the plthook.h file.
        match value {
            1 => Error::FileNotFound,
            2 => Error::InvalidFileFormat,
            3 => Error::FunctionNotFound,
            4 => Error::InvalidArgument,
            5 => Error::OutOfMemory,
            6 => Error::InternalError,
            7 => Error::NotImplemented,
            _ => Error::UnknownError(value),
        }
    }
}
