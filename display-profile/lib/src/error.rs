use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    WindowsDisplayConfigError(windows_display_config::Error),
    Custom(String),
}

impl From<windows_display_config::Error> for Error {
    fn from(value: windows_display_config::Error) -> Self {
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
