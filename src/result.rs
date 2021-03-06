// This file is part of rust-hox.
//
// rust-hox is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// rust-hox is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with rust-hox.  If not, see <https://www.gnu.org/licenses/>.

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
    #[allow(unused)]
    #[inline]
    pub fn new(error_type: ErrorType, path: Option<PathBuf>) -> Self {
        Error {
            path,
            error_type,
        }
    }

    #[allow(unused)]
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

    #[allow(unused)]
    #[inline]
    pub fn io_with_path(error: std::io::Error, path: impl AsRef<Path>) -> Self {
        Error {
            path:       Some(path.as_ref().to_path_buf()),
            error_type: ErrorType::IO(error),
        }
    }

    #[allow(unused)]
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

impl From<std::num::ParseIntError> for Error {
    fn from(error: std::num::ParseIntError) -> Self {
        Error {
            error_type: ErrorType::Message(format!("{}", error)),
            path: None,
        }
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Error {
            error_type: ErrorType::Message(format!("{}", error)),
            path: None,
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
