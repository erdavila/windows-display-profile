use std::fmt::Display;

use crate::Result;
use crate::windows::{
    ERROR_ACCESS_DENIED, ERROR_GEN_FAILURE, ERROR_INSUFFICIENT_BUFFER, ERROR_INVALID_PARAMETER,
    ERROR_NOT_SUPPORTED, ERROR_SUCCESS, WIN32_ERROR,
};

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

    /// The Windows API error code wrapper.
    #[must_use]
    pub fn win32_error(self) -> Win32Error {
        self.win32_error
    }

    /// The name of the Windows API function that failed.
    #[must_use]
    pub fn function(self) -> &'static str {
        self.function
    }

    #[cfg(feature = "dump")]
    #[must_use]
    pub(crate) fn is_ok(self) -> bool {
        self.win32_error.is_ok()
    }

    #[cfg(feature = "dump")]
    #[must_use]
    pub(crate) fn is_err(&self) -> bool {
        self.win32_error.is_err()
    }

    pub(crate) fn to_result<T>(self, value: T) -> Result<T> {
        self.to_result_with(|| value)
    }

    pub(crate) fn to_result_with<T>(self, f: impl FnOnce() -> T) -> Result<T> {
        if self.win32_error.is_ok() {
            Ok(f())
        } else {
            Err(self)
        }
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.function, self.win32_error)
    }
}

/// A [`WIN32_ERROR`] wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Win32Error(WIN32_ERROR);

impl Win32Error {
    /// Tells whether the error is [`ERROR_SUCCESS`].
    #[must_use]
    pub fn is_ok(self) -> bool {
        self.0.is_ok()
    }

    /// Tells whether the error is *not* [`ERROR_SUCCESS`].
    #[must_use]
    pub fn is_err(self) -> bool {
        self.0.is_err()
    }

    /// The numeric error code.
    ///
    /// # Example
    ///
    /// ```
    /// use windows_ccd::Win32Error;
    /// use windows_ccd::windows::ERROR_INVALID_PARAMETER;
    ///
    /// let e = Win32Error::from(ERROR_INVALID_PARAMETER);
    /// assert_eq!(e.code_number(), 87);
    /// ```
    #[must_use]
    pub fn code_number(self) -> u32 {
        self.0.0
    }

    /// The name and description of the error.
    ///
    /// # Example
    /// ```
    /// use windows_ccd::Win32Error;
    /// use windows_ccd::windows::ERROR_INVALID_PARAMETER;
    ///
    /// let e = Win32Error::from(ERROR_INVALID_PARAMETER);
    /// let (error_code_str, error_text) = e.code_str_and_text().unwrap();
    /// assert_eq!(error_code_str, "ERROR_INVALID_PARAMETER");
    /// assert_eq!(error_text, "The combination of parameters and flags that are specified is invalid.");
    /// ```
    #[must_use]
    pub fn code_str_and_text(self) -> Option<(&'static str, &'static str)> {
        CODES
            .iter()
            .find_map(|(code, code_str, text)| (self.0 == *code).then_some((*code_str, *text)))
    }

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
