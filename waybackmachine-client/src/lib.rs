use reqwest::Response;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::future::Future;
use url::Url;

type HttpFuture = Result<Response, reqwest_middleware::Error>;

/// Maximum number of allowed request retries attempts.
const DEFAULT_MAX_REQUEST_RETRIES: u32 = 5;

/// User-agent to make requests from
const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Fedora; Linux x86_64; rv:40.0) Gecko/20100101 Firefox/40.0";

/// Endpoint for the Wayback Machine service
const WAYBACK_MACHINE_ENDPOINT: &str = "https://web.archive.org/save/";

/// Configuration for the Wayback Machine client
pub struct ClientConfig {
    retry_policy: ExponentialBackoff,
    user_agent: &'static str,
    endpoint: &'static str,
}

impl ClientConfig {
    /// Constructs a new `ClientConfig` with custom retry policy and user agent
    pub fn new(max_request_retries: u32, user_agent: &'static str) -> Self {
        ClientConfig {
            retry_policy: ExponentialBackoff::builder().build_with_max_retries(max_request_retries),
            user_agent,
            endpoint: WAYBACK_MACHINE_ENDPOINT,
        }
    }
}
impl Default for ClientConfig {
    /// Constructs a default `ClientConfig` with default retry policy and user agent
    fn default() -> Self {
        ClientConfig {
            retry_policy: ExponentialBackoff::builder()
                .build_with_max_retries(DEFAULT_MAX_REQUEST_RETRIES),
            user_agent: DEFAULT_USER_AGENT,
            endpoint: WAYBACK_MACHINE_ENDPOINT,
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
                .user_agent(client_config.user_agent)
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

    /// Archives a URL using the Wayback Machine service
    ///
    /// # Errors
    ///
    /// This method fails if the `url` provided is not well formatted
    /// of if there was an error while sending the request
    ///
    /// # Example
    /// ```
    /// use waybackmachine_client::{ClientConfig, WaybackMachineClient};
    ///
    /// # async fn main() -> Result<(), reqwest_middleware::Error> {
    /// let wayback_client = WaybackMachineClient::new(ClientConfig::default());
    /// wayback_client.archive_url("https://www.openbookpublishers.com/").await
    /// # }
    /// ```
    pub async fn archive_url(&self, url: &str) -> impl Future<Output = HttpFuture> {
        let to_archive = Url::parse(url).expect(&*format!("{} is not a valid URL", url));
        self.http_client
            .get(format!("{}{}", self.client_config.endpoint, to_archive))
            .send()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
