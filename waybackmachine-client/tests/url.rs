use waybackmachine_client::{ArchivableUrl, Error};

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