use std::fmt;

/// Errors that can occur during archiving.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidUrl(String),
    RequestFailed(String),
    CannotArchive(String, String),
    CannotCheckArchive(String),
    NoRecentArchive(String),
    ExcludedUrl(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidUrl(input) => write!(f, "Invalid URL: {}", input),
            Error::RequestFailed(err) => write!(f, "Request failed: {}", err),
            Error::CannotArchive(code, url) => write!(f, "Failed ({}): {}", code, url),
            Error::CannotCheckArchive(error) => write!(f, "Failed to get archive: {}", error),
            Error::NoRecentArchive(url) => write!(f, "No recent archive exists: {}", url),
            Error::ExcludedUrl(url) => write!(f, "Excluded URL: {}", url),
        }
    }
}

impl From<reqwest_middleware::Error> for Error {
    fn from(err: reqwest_middleware::Error) -> Self {
        Error::RequestFailed(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::RequestFailed(err.to_string())
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Self {
        Error::CannotCheckArchive(err.to_string())
    }
}
