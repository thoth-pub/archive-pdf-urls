# Wayback Machine Client

This Rust crate provides a client for interacting with the Wayback Machine, allowing users to archive URLs.

[![Build status](https://github.com/thoth-pub/archive-pdf-urls/workflows/test-and-check/badge.svg)](https://github.com/thoth-pub/archive-pdf-urls/actions)
[![Crates.io](https://img.shields.io/crates/v/waybackmachine-client.svg)](https://crates.io/crates/waybackmachine-client)

## Installation

```bash
cargo add waybackmachine-client
```

## Usage

The WaybackMachineClient struct provides methods for archiving URLs using the Wayback Machine service. You can use the archive\_url method to archive a URL asynchronously.

Example:

```rust
use waybackmachine_client::{ClientConfig, Error, WaybackMachineClient};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let wayback_client = WaybackMachineClient::new(ClientConfig::default());
    wayback_client.archive_url("https://www.example.com").await?;
    Ok(())
}
```

## Features

- Automatic Retry: The client automatically retries failed requests with exponential backoff, configurable via the `ClientConfig.max_request_retries` setting.
- Recent Archive Check: The client checks if a URL has been archived within a specified threshold using the `ClientConfig.archive_threshold_days` setting.
- Customisable Configuration: You can customise the client's behavior using the `ClientConfig` struct.
- Asynchronous: Requests are sent asynchronously using the Tokio runtime

