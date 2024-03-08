# Wayback Machine Client

This Rust crate provides a client for interacting with the Wayback Machine, allowing users to archive URLs.
## Installation

```bash
cargo add waybackmachine-client
```

## Usage

### Client Configuration

The ClientConfig struct allows you to configure the behavior of the Wayback Machine client. It provides options to set the maximum number of request retries and the user-agent string to be used for requests.

Example:

```rust
use waybackmachine_client::{ClientConfig, WaybackMachineClient};

fn main() {
    let client_config = ClientConfig::new(5, "MyCustomUserAgent");
    let client = WaybackMachineClient::new(client_config);
}
```

### Archiving URLs

The WaybackMachineClient struct provides methods for archiving URLs using the Wayback Machine service. You can use the archive_url method to archive a URL asynchronously.

Example:

```rust
use waybackmachine_client::{ClientConfig, WaybackMachineClient};

#[tokio::main]
async fn main() -> Result<(), reqwest_middleware::Error> {
    let wayback_client = WaybackMachineClient::new(ClientConfig::default());
    wayback_client.archive_url("https://www.example.com").await?;
    Ok(())
}
```

## Features

- Automatic Retry: The client automatically retries failed requests with exponential backoff.
- Customisable Configuration: You can customise the client's behavior using the `ClientConfig` struct.
- Asynchronous: Requests are sent asynchronously using the Tokio runtime