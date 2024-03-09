use mockito::ServerGuard;
use waybackmachine_client::{ClientConfig, Error, WaybackMachineClient};

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