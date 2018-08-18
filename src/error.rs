use failure::{Context, Fail};
use std::fmt::{self, Display};
use std::io::Error as IoError;

#[derive(Clone, Debug, Fail, PartialEq)]
pub enum ErrorKind {
    #[fail(display = "File '{}' is not a zip archive", _0)]
    NotZipArchive(String),

    #[fail(display = "Zip archive does not contain entry '{}'", _0)]
    ZipFileEntryNotFound(String),

    #[fail(display = "Zip entry '{}' is not a zip archive", _0)]
    ZipEntryNotZipArchive(String),

    #[fail(display = "No such file or directory '{}'", _0)]
    DoesNotExist(String),

    #[fail(display = "IO error: {}", _0)]
    IoError(String),

    #[fail(display = "No nested target file provided for unpack. \
    Please provide a nested file you want to unpack")]
    UnpackTargetMissing,

    #[fail(display = "Failed to create directory '{}'", _0)]
    CannotCreateDirectory(String),

    #[fail(display = "Cannot create file '{}'", _0)]
    CannotCreateFile(String),

    #[fail(display = "Cannot set permission '{}' for file '{}'", _0, _1)]
    CannotSetPermissions(u32, String),

    #[fail(display = "Unpack target '{}' already exists", _0)]
    TargetAlreadyExists(String)
}

#[derive(Debug, Fail)]
pub struct Error {
    inner: Context<ErrorKind>,
}

impl Error {
    #[cfg(test)]
    pub fn kind(&self) -> ErrorKind {
        self.inner.get_context().clone()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(error_kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(error_kind)
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error {
            inner
        }
    }
}

impl From<IoError> for Error {
    fn from(io_error: IoError) -> Error {
        let error_string = io_error.to_string();
        Error {
            inner: io_error.context(ErrorKind::IoError(error_string))
        }
    }
}
