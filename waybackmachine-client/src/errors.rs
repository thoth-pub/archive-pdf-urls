use std::fmt;

/// Errors that can occur during archiving.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidUrl(String),
    RequestFailed(String),
    CannotArchive(String, String),
    InvalidHost(String, String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidUrl(input) => write!(f, "Invalid URL: {}", input),
            Error::RequestFailed(err) => write!(f, "Request failed: {}", err),
            Error::CannotArchive(code, url) => write!(f, "Failed ({}): {}", code, url),
            Error::InvalidHost(url, host) => write!(f, "Invalid host \"{}\": {}", host, url),
        }
    }
}

impl From<reqwest_middleware::Error> for Error {
    fn from(err: reqwest_middleware::Error) -> Self {
        Error::RequestFailed(err.to_string())
    }
}
