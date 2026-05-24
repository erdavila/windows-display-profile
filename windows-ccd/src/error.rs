// TODO: Improve the public interface.
//   It's odd for an Error type to have an `is_ok()` method!
//   Should the fields be public?
//   Should the `Win32Error` type be public?
//   Should `Error` and `Win32Error` be merged?

use std::fmt::Display;

use windows::Win32::Foundation::{
    ERROR_ACCESS_DENIED, ERROR_GEN_FAILURE, ERROR_INSUFFICIENT_BUFFER, ERROR_INVALID_PARAMETER,
    ERROR_NOT_SUPPORTED, ERROR_SUCCESS, WIN32_ERROR,
};

use crate::Result;

macro_rules! codes {
    (
        $(
            ( $code:ident, $text:literal ),
        )*
    ) => {
        &[
            $(
                ($code, stringify!($code), $text),
            )*
        ]
    };
}

static CODES: &[(WIN32_ERROR, &str, &str)] = codes![
    (ERROR_SUCCESS, "The function succeeded"),
    (
        ERROR_INVALID_PARAMETER,
        "The combination of parameters and flags that are specified is invalid."
    ),
    (
        ERROR_NOT_SUPPORTED,
        "The system is not running a graphics driver that was written according to the Windows Display Driver Model (WDDM). The function is only supported on a system with a WDDM driver running."
    ),
    (
        ERROR_ACCESS_DENIED,
        "The caller does not have access to the console session. This error occurs if the calling process does not have access to the current desktop or is running on a remote session."
    ),
    (ERROR_GEN_FAILURE, "An unspecified error occurred."),
    (
        ERROR_INSUFFICIENT_BUFFER,
        "The supplied path and mode buffer are too small."
    ),
];

/// Error type for this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
    pub(crate) win32_error: Win32Error,
    pub(crate) function: &'static str,
}

impl Error {
    pub(crate) fn new(win32_error: impl Into<Win32Error>, function: &'static str) -> Self {
        win32_error.into().to_error(function)
    }

    /// Tells whether this is not really an error.
    #[must_use]
    pub fn is_ok(self) -> bool {
        self.win32_error.is_ok()
    }

    /// Tells whether this is really an error.
    #[must_use]
    pub fn is_err(&self) -> bool {
        self.win32_error.is_err()
    }

    pub(crate) fn to_result<T>(self, value: T) -> Result<T> {
        self.to_result_with(|| value)
    }

    pub(crate) fn to_result_with<T>(self, f: impl FnOnce() -> T) -> Result<T> {
        if self.is_ok() { Ok(f()) } else { Err(self) }
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.function, self.win32_error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Win32Error(WIN32_ERROR);

impl Win32Error {
    fn is_ok(self) -> bool {
        self.0.is_ok()
    }

    fn is_err(self) -> bool {
        self.0.is_err()
    }

    fn code_number(self) -> u32 {
        self.0.0
    }

    fn code_str_and_text(self) -> Option<(&'static str, &'static str)> {
        CODES
            .iter()
            .find_map(|(code, code_str, text)| (self.0 == *code).then_some((*code_str, *text)))
    }
}

impl Win32Error {
    fn to_error(self, function: &'static str) -> Error {
        Error {
            win32_error: self,
            function,
        }
    }
}

impl From<WIN32_ERROR> for Win32Error {
    fn from(value: WIN32_ERROR) -> Self {
        Win32Error(value)
    }
}

impl From<u32> for Win32Error {
    fn from(value: u32) -> Self {
        Win32Error::from(WIN32_ERROR(value))
    }
}

impl From<i32> for Win32Error {
    fn from(value: i32) -> Self {
        Win32Error::from(value.cast_unsigned())
    }
}

impl Display for Win32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.code_str_and_text() {
            Some((code_str, text)) => write!(f, "{code_str} - {text}"),
            None => write!(f, "{} - Unknown error", self.code_number()),
        }
    }
}
