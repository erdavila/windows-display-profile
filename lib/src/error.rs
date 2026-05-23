use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    WindowsDisplayConfigError(windows_ccd::Error),
    Custom(String),
}

impl From<windows_ccd::Error> for Error {
    fn from(value: windows_ccd::Error) -> Self {
        Error::WindowsDisplayConfigError(value)
    }
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WindowsDisplayConfigError(err) => write!(f, "{err}"),
            Error::Custom(message) => write!(f, "{message}"),
        }
    }
}
