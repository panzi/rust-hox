use std::path::{PathBuf, Path};

#[derive(Debug)]
pub enum ErrorType {
    IO(std::io::Error),
    Message(String),
}

#[derive(Debug)]
pub struct Error {
    pub(crate) error_type: ErrorType,
    pub(crate) path:       Option<PathBuf>,
}

impl Error {
    #[inline]
    pub fn new(error_type: ErrorType, path: Option<PathBuf>) -> Self {
        Error {
            path,
            error_type,
        }
    }

    #[inline]
    pub fn error_type(&self) -> &ErrorType {
        &self.error_type
    }

    #[inline]
    pub fn path(&self) -> &Option<PathBuf> {
        &self.path
    }

    #[inline]
    pub fn with_path(self, path: impl AsRef<Path>) -> Self {
        Error {
            path:       Some(path.as_ref().to_path_buf()),
            error_type: self.error_type,
        }
    }

    #[inline]
    pub fn io_with_path(error: std::io::Error, path: impl AsRef<Path>) -> Self {
        Error {
            path:       Some(path.as_ref().to_path_buf()),
            error_type: ErrorType::IO(error),
        }
    }

    #[inline]
    pub fn io(error: std::io::Error) -> Self {
        Error {
            path:       None,
            error_type: ErrorType::IO(error),
        }
    }

    #[inline]
    pub fn message(message: impl AsRef<str>) -> Self {
        Error {
            path:       None,
            error_type: ErrorType::Message(message.as_ref().to_owned()),
        }
    }
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::IO(err)      => err.fmt(f),
            ErrorType::Message(msg) => msg.fmt(f),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{:?}: {}", path, self.error_type)
        } else {
            self.error_type.fmt(f)
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error {
            error_type: ErrorType::IO(error),
            path: None,
        }
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error {
            error_type: ErrorType::Message("ncurses error".to_owned()),
            path: None,
        }
    }
}

impl From<std::fmt::Error> for Error {
    fn from(error: std::fmt::Error) -> Self {
        Error {
            error_type: ErrorType::Message(format!("{}", error)),
            path: None,
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
