pub mod archivableurl;
pub mod errors;

pub use crate::archivableurl::ArchivableUrl;
pub use crate::errors::Error;
use chrono::{NaiveDateTime, TimeDelta, Utc};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::Deserialize;
use std::collections::HashMap;
use url::Url;

/// Maximum number of allowed request retries attempts.
const DEFAULT_MAX_REQUEST_RETRIES: u32 = 5;

/// Default threshold for considering an archive as recent, in days.
/// URLs with archives older than this threshold will be re-archived.
const DEFAULT_ARCHIVE_THRESHOLD_DAYS: i64 = 30;

/// User-agent to make requests from
const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Fedora; Linux x86_64; rv:40.0) Gecko/20100101 Firefox/40.0";

/// Endpoint for the Wayback Machine archiving service
const WAYBACK_MACHINE_ARCHIVE_ENDPOINT: &str = "https://web.archive.org/save/";
/// Endpoint to check if an archive is present in the Wayback Machine
const WAYBACK_MACHINE_CHECK_ENDPOINT: &str = "https://archive.org/wayback/available?url=";

#[derive(Debug, Deserialize)]
struct WaybackCheckResponse {
    archived_snapshots: Option<HashMap<String, ArchivedSnapshot>>,
}

#[derive(Debug, Deserialize)]
struct ArchivedSnapshot {
    status: String,
    available: bool,
    timestamp: String,
}

/// Configuration for the Wayback Machine client
pub struct ClientConfig {
    archive_endpoint: String,
    check_endpoint: String,
    retry_policy: ExponentialBackoff,
    archive_threshold_timestamp: NaiveDateTime,
    user_agent: String,
}

impl ClientConfig {
    /// Constructs a new `ClientConfig` with custom retry policy and user agent
    pub fn new(
        archive_endpoint: String,
        check_endpoint: String,
        max_request_retries: u32,
        archive_threshold_days: i64,
        user_agent: String,
    ) -> Self {
        ClientConfig {
            archive_endpoint: Url::parse(&archive_endpoint)
                .unwrap_or_else(|_| panic!("Invalid archive_endpoint URL: {}", archive_endpoint))
                .to_string(),
            check_endpoint: Url::parse(&check_endpoint)
                .unwrap_or_else(|_| panic!("Invalid check_endpoint URL: {}", check_endpoint))
                .to_string(),
            retry_policy: ExponentialBackoff::builder().build_with_max_retries(max_request_retries),
            archive_threshold_timestamp: (Utc::now()
                - TimeDelta::try_days(archive_threshold_days).unwrap())
            .naive_utc(),
            user_agent,
        }
    }
}
impl Default for ClientConfig {
    /// Constructs a default `ClientConfig` with default retry policy and user agent
    fn default() -> Self {
        ClientConfig {
            archive_endpoint: WAYBACK_MACHINE_ARCHIVE_ENDPOINT.into(),
            check_endpoint: WAYBACK_MACHINE_CHECK_ENDPOINT.into(),
            retry_policy: ExponentialBackoff::builder()
                .build_with_max_retries(DEFAULT_MAX_REQUEST_RETRIES),
            archive_threshold_timestamp: (Utc::now()
                - TimeDelta::try_days(DEFAULT_ARCHIVE_THRESHOLD_DAYS).unwrap())
            .naive_utc(),
            user_agent: DEFAULT_USER_AGENT.into(),
        }
    }
}

/// Wayback Machine client for archiving URLs
pub struct WaybackMachineClient {
    http_client: ClientWithMiddleware,
    client_config: ClientConfig,
}

impl WaybackMachineClient {
    /// Constructs a new `WaybackMachineClient` with the given configuration
    pub fn new(client_config: ClientConfig) -> Self {
        let http_client = ClientBuilder::new(
            reqwest::Client::builder()
                .user_agent(client_config.user_agent.clone())
                .build()
                .unwrap(),
        )
        .with(RetryTransientMiddleware::new_with_policy(
            client_config.retry_policy,
        ))
        .build();
        WaybackMachineClient {
            http_client,
            client_config,
        }
    }

    /// Checks if a recent archive exists for the given URL.
    ///
    /// If an archive exists, and it is newer than the configured archive threshold,
    /// the function returns Ok(()), indicating that the URL is considered recently archived.
    /// If no recent archive is found or the found archive is older than the threshold,
    /// it returns Err(Error::NoRecentArchive).
    ///
    /// https://archive.org/help/wayback_api.php
    ///
    async fn check_recent_archive_exists(&self, url: &str) -> Result<(), Error> {
        let to_check = ArchivableUrl::parse(url)?;
        let response = self
            .http_client
            .get(format!("{}{}", self.client_config.check_endpoint, to_check))
            .send()
            .await
            .map_err(|err| Error::CannotCheckArchive(err.to_string()))?
            .json::<WaybackCheckResponse>()
            .await
            .map_err(|e| Error::CannotCheckArchive(e.to_string()))?;

        if let Some(snapshots) = response.archived_snapshots {
            if let Some((_, snapshot)) = snapshots
                .iter()
                .max_by_key(|(_, snapshot)| &snapshot.timestamp)
            {
                let snapshot_timestamp =
                    NaiveDateTime::parse_from_str(&snapshot.timestamp, "%Y%m%d%H%M%S")?;
                if snapshot_timestamp > self.client_config.archive_threshold_timestamp
                    && snapshot.available
                    && snapshot.status.eq("200")
                {
                    return Ok(());
                }
            }
        }
        Err(Error::NoRecentArchive(url.to_string()))
    }

    /// Checks if a recent Wayback Machine archive exists for the given URL
    /// and archives it if necessary.
    ///
    /// This function first checks if a recent archive exists for the URL by calling
    /// `check_recent_archive_exists`. If an archive does not exist or is older than the
    /// configured archive threshold, it proceeds to archive the URL.
    ///
    /// # Errors
    ///
    /// This method fails if the `url` provided is not well formatted
    /// of if there was an error while sending the request. If a recent archive
    /// already exists, it returns an `Error::RecentArchiveExists`.
    ///
    /// # Example
    /// ```
    /// use waybackmachine_client::{ClientConfig, Error, WaybackMachineClient};
    ///
    /// # async fn run() -> Result<(), Error> {
    /// let wayback_client = WaybackMachineClient::new(ClientConfig::default());
    /// wayback_client.archive_url("https://www.openbookpublishers.com/").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn archive_url(&self, url: &str) -> Result<(), Error> {
        let to_archive = ArchivableUrl::parse(url)?;

        if self.check_recent_archive_exists(url).await.is_ok() {
            return Err(Error::RecentArchiveExists(url.to_string()));
        }

        let response = self
            .http_client
            .get(format!(
                "{}{}",
                self.client_config.archive_endpoint, to_archive
            ))
            .send()
            .await?;
        if !response.status().is_success() {
            return Err(Error::CannotArchive(
                response.status().to_string(),
                url.to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::ServerGuard;
    use serde_json::{json, Value};

    const ARCHIVE_ROOT_PATH: &str = "/save/";
    const CHECK_ROOT_PATH: &str = "/wayback/available?url=";
    const MAX_REQUEST_RETRIES: u32 = 3;

    async fn mock_server() -> (ServerGuard, WaybackMachineClient) {
        let server = mockito::Server::new_async().await;
        let client_config = ClientConfig::new(
            format!("{}{}", server.url(), ARCHIVE_ROOT_PATH),
            format!("{}{}", server.url(), CHECK_ROOT_PATH),
            MAX_REQUEST_RETRIES,
            30,
            "TestUserAgent".to_string(),
        );
        let wayback_client = WaybackMachineClient::new(client_config);
        (server, wayback_client)
    }

    #[tokio::test]
    async fn test_archive_url_success() {
        let to_archive = "https://example.com/";
        let snapshot_timestamp = "20230227054528";
        let (mut server, wayback_client) = mock_server().await;

        let snapshot: Value = json!({
            "url": to_archive,
            "archived_snapshots": {
                "closest": {
                    "status": "200",
                    "available": true,
                    "url": format!("http://web.archive.org/web/{}/{}", snapshot_timestamp, to_archive),
                    "timestamp": snapshot_timestamp
                }
            }
        });
        let mock1 = server
            .mock("GET", &format!("{}{}", CHECK_ROOT_PATH, to_archive)[..])
            .with_status(200)
            .with_body(snapshot.to_string())
            .create_async()
            .await;
        let mock2 = server
            .mock("GET", &format!("{}{}", ARCHIVE_ROOT_PATH, to_archive)[..])
            .with_status(200)
            .create_async()
            .await;

        assert!(wayback_client.archive_url(to_archive).await.is_ok());
        mock1.assert_async().await;
        mock2.assert_async().await;
    }

    #[tokio::test]
    async fn test_archive_url_no_scheme() {
        let to_archive = "example.com";
        let wayback_client = WaybackMachineClient::new(ClientConfig::default());

        assert_eq!(
            wayback_client.archive_url(to_archive).await.err().unwrap(),
            Error::InvalidUrl(to_archive.to_string())
        );
    }

    #[tokio::test]
    async fn test_archive_url_local_url() {
        let to_archive = "http://localhost/page";
        let wayback_client = WaybackMachineClient::new(ClientConfig::default());

        assert_eq!(
            wayback_client.archive_url(to_archive).await.err().unwrap(),
            Error::InvalidUrl(to_archive.to_string())
        );
    }

    #[tokio::test]
    async fn test_archive_url_failure() {
        let to_archive = "https://example.com/";
        let snapshot_timestamp = "20230227054528";
        let (mut server, wayback_client) = mock_server().await;

        let snapshot: Value = json!({
            "url": to_archive,
            "archived_snapshots": {
                "closest": {
                    "status": "200",
                    "available": true,
                    "url": format!("http://web.archive.org/web/{}/{}", snapshot_timestamp, to_archive),
                    "timestamp": snapshot_timestamp
                }
            }
        });
        let mock1 = server
            .mock("GET", &format!("{}{}", CHECK_ROOT_PATH, to_archive)[..])
            .with_status(200)
            .with_body(snapshot.to_string())
            .create_async()
            .await;
        let mock2 = server
            .mock("GET", &format!("{}{}", ARCHIVE_ROOT_PATH, to_archive)[..])
            .with_status(520)
            .expect_at_least(MAX_REQUEST_RETRIES as usize)
            .create_async()
            .await;

        assert!(wayback_client.archive_url(to_archive).await.is_err());
        mock1.assert_async().await;
        mock2.assert_async().await;
    }

    #[tokio::test]
    async fn test_check_recent_archive_exists_success() {
        let to_archive = "https://example.com/";
        let snapshot_timestamp = (Utc::now() - TimeDelta::try_days(1).unwrap())
            .format("%Y%m%d%H%M%S")
            .to_string();
        let (mut server, wayback_client) = mock_server().await;

        let snapshot: Value = json!({
            "url": to_archive,
            "archived_snapshots": {
                "closest": {
                    "status": "200",
                    "available": true,
                    "url": format!("http://web.archive.org/web/{}/{}", snapshot_timestamp, to_archive),
                    "timestamp": snapshot_timestamp
                }
            }
        });
        let mock = server
            .mock("GET", &format!("{}{}", CHECK_ROOT_PATH, to_archive)[..])
            .with_status(200)
            .with_body(snapshot.to_string())
            .create_async()
            .await;

        assert!(wayback_client
            .check_recent_archive_exists(to_archive)
            .await
            .is_ok());
        mock.assert_async().await;
    }
}
