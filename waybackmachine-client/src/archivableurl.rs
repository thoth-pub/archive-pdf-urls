use crate::Error;
use std::fmt;
use url::{Host, Url};

/// Validator for archivable URLs
pub struct ArchivableUrl {
    pub url: Url,
}

/// List of domains that block wayback requests
const EXCLUDED_DOMAINS: &[&str] = &[
    "archive.org",
    "jstor.org",
    "diw.de",
    "youtube.com",
    "plato.stanford.edu",
];

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
            Host::Domain(domain) => {
                if domain.contains("localhost") {
                    return Err(Error::InvalidUrl(self.url.to_string()));
                }

                for &pattern in EXCLUDED_DOMAINS {
                    if domain.contains(pattern) {
                        return Err(Error::ExcludedUrl(self.url.to_string()));
                    }
                }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_http_url() {
        let url = "http://example.com/";
        let result = ArchivableUrl::parse(url);
        assert!(result.is_ok());
        let archivable_url = result.unwrap();
        assert_eq!(archivable_url.as_str(), url);
    }

    #[test]
    fn valid_https_url() {
        let url = "https://example.com/";
        let result = ArchivableUrl::parse(url);
        assert!(result.is_ok());
        let archivable_url = result.unwrap();
        assert_eq!(archivable_url.as_str(), url);
    }

    #[test]
    fn invalid_url() {
        let url = "invalid-url";
        let result = ArchivableUrl::parse(url);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::InvalidUrl(url.to_string())));
    }

    #[test]
    fn invalid_scheme() {
        let url = "ftp://example.com/";
        let result = ArchivableUrl::parse(url);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::InvalidUrl(url.to_string())));
    }

    #[test]
    fn localhost_url() {
        let url = "http://localhost/";
        let result = ArchivableUrl::parse(url);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::InvalidUrl(url.to_string())));
    }

    #[test]
    fn private_ip_url() {
        let url = "http://192.168.1.1/";
        let result = ArchivableUrl::parse(url);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::InvalidUrl(url.to_string())));
    }

    #[test]
    fn reserved_ip_url() {
        let url = "http://0.0.0.0/";
        let result = ArchivableUrl::parse(url);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::InvalidUrl(url.to_string())));
    }

    #[test]
    fn special_localhost_alias_url() {
        let url = "http://localhost.localdomain/";
        let result = ArchivableUrl::parse(url);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::InvalidUrl(url.to_string())));
    }

    #[test]
    fn wayback_url() {
        let url = "https://archive.org/some-book";
        let result = ArchivableUrl::parse(url);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::ExcludedUrl(url.to_string())));
    }

    #[test]
    fn jstor_url() {
        let url = "https://jstor.org/some-book";
        let result = ArchivableUrl::parse(url);
        assert!(result.is_err());
        assert_eq!(result.err(), Some(Error::ExcludedUrl(url.to_string())));
    }

    #[test]
    fn excluded_domains() {
        for &domain in EXCLUDED_DOMAINS {
            let url = format!("https://{}/some-path", domain);
            let result = ArchivableUrl::parse(&url);
            assert!(result.is_err());
            assert_eq!(result.err(), Some(Error::ExcludedUrl(url)));
        }
    }
}
