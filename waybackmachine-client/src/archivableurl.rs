use crate::Error;
use std::fmt;
use url::{Host, Url};

/// Validator for archivable URLs
pub struct ArchivableUrl {
    pub url: Url,
}

impl ArchivableUrl {
    /// Parses and validates the URL for archiving
    pub fn parse(url: &str) -> Result<Self, Error> {
        let parsed_url = Url::parse(url).map_err(|_| Error::InvalidUrl(url.to_string()))?;
        let archivable_url = Self { url: parsed_url };
        archivable_url.validate_url()
    }

    /// Validates the URL for archiving
    fn validate_url(self) -> Result<Self, Error> {
        let host = match self.url.host() {
            Some(host) => host,
            None => return Err(Error::InvalidUrl(self.url.to_string())),
        };

        // Check if the host is excluded
        match host {
            Host::Domain(domain) if domain.contains("localhost") => {
                return Err(Error::InvalidUrl(self.url.to_string()));
            }
            Host::Ipv4(ipv4)
                if ipv4.is_loopback()
                    || ipv4.is_private()
                    || ipv4.is_multicast()
                    || ipv4.is_unspecified() =>
            {
                return Err(Error::InvalidUrl(self.url.to_string()));
            }
            Host::Ipv6(ipv6) if ipv6.is_loopback() || ipv6.is_multicast() => {
                return Err(Error::InvalidUrl(self.url.to_string()));
            }
            _ => {}
        }

        // Check for non-HTTP(S) protocols
        if !["http", "https"].contains(&self.url.scheme()) {
            return Err(Error::InvalidUrl(self.url.to_string()));
        }

        // If none of the filters matched, the URL is valid for archiving
        Ok(self)
    }

    /// Returns the URL as a string
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }
}

impl fmt::Display for ArchivableUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}
