pub mod errors;

pub use crate::errors::Error;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use url::Url;

/// Maximum number of allowed request retries attempts.
const DEFAULT_MAX_REQUEST_RETRIES: u32 = 5;

/// User-agent to make requests from
const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Fedora; Linux x86_64; rv:40.0) Gecko/20100101 Firefox/40.0";

/// Endpoint for the Wayback Machine service
const WAYBACK_MACHINE_ENDPOINT: &str = "https://web.archive.org/save/";

/// Configuration for the Wayback Machine client
pub struct ClientConfig {
    endpoint: String,
    retry_policy: ExponentialBackoff,
    user_agent: String,
}

impl ClientConfig {
    /// Constructs a new `ClientConfig` with custom retry policy and user agent
    pub fn new(endpoint: String, max_request_retries: u32, user_agent: String) -> Self {
        ClientConfig {
            endpoint: Url::parse(&endpoint)
                .expect(&format!("Invalid endpoint URL: {}", endpoint))
                .to_string(),
            retry_policy: ExponentialBackoff::builder().build_with_max_retries(max_request_retries),
            user_agent,
        }
    }
}
impl Default for ClientConfig {
    /// Constructs a default `ClientConfig` with default retry policy and user agent
    fn default() -> Self {
        ClientConfig {
            endpoint: WAYBACK_MACHINE_ENDPOINT.into(),
            retry_policy: ExponentialBackoff::builder()
                .build_with_max_retries(DEFAULT_MAX_REQUEST_RETRIES),
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
            client_config.retry_policy.clone(),
        ))
        .build();
        WaybackMachineClient {
            http_client,
            client_config,
        }
    }

    /// Archives a URL using the Wayback Machine service
    ///
    /// # Errors
    ///
    /// This method fails if the `url` provided is not well formatted
    /// of if there was an error while sending the request
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
        let to_archive = Url::parse(url).map_err(|_| Error::InvalidUrl(url.to_string()))?;
        let response = self
            .http_client
            .get(format!("{}{}", self.client_config.endpoint, to_archive))
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

    const ROOT_PATH: &str = "/save/";
    const MAX_REQUEST_RETRIES: u32 = 3;

    async fn mock_server() -> (ServerGuard, WaybackMachineClient) {
        let server = mockito::Server::new_async().await;
        let client_config = ClientConfig::new(
            format!("{}{}", server.url(), ROOT_PATH),
            MAX_REQUEST_RETRIES,
            "TestUserAgent".to_string(),
        );
        let wayback_client = WaybackMachineClient::new(client_config);
        (server, wayback_client)
    }

    #[tokio::test]
    async fn test_archive_url_success() {
        let to_archive = "https://example.com/";
        let (mut server, wayback_client) = mock_server().await;

        let mock = server
            .mock("GET", &format!("{}{}", ROOT_PATH, to_archive)[..])
            .with_status(200)
            .create_async()
            .await;

        assert!(wayback_client.archive_url(to_archive).await.is_ok());
        mock.assert_async().await;
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

    // #[tokio::test]
    // async fn test_archive_url_local_url() {
    //     let to_archive = "http://localhost/page";
    //     let wayback_client = WaybackMachineClient::new(ClientConfig::default());
    //
    //     assert_eq!(
    //         wayback_client.archive_url(to_archive).await.err().unwrap(),
    //         Error::InvalidHost("localhost".to_string(), to_archive.to_string())
    //     );
    // }

    #[tokio::test]
    async fn test_archive_url_failure() {
        let to_archive = "https://example.com/";
        let (mut server, wayback_client) = mock_server().await;

        let mock = server
            .mock("GET", &format!("{}{}", ROOT_PATH, to_archive)[..])
            .with_status(520)
            .expect_at_least(MAX_REQUEST_RETRIES as usize)
            .create_async()
            .await;

        assert!(wayback_client.archive_url(to_archive).await.is_err());
        mock.assert_async().await;
    }
}
