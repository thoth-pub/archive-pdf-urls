use log::{log, Level};
use std::io::{self, BufRead};
use waybackmachine_client::{ClientConfig, WaybackMachineClient};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_target(false)
        .init();

    let client = WaybackMachineClient::new(ClientConfig::default());

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let url = line.expect("Failed to read line from standard input");
        let response = client
            .archive_url(&url)
            .await
            .await
            .expect(&*format!("Error sending request to archive: {}", url));
        log!(
            if response.status().is_success() {
                Level::Info
            } else {
                Level::Error
            },
            "{} ({}): {}",
            if response.status().is_success() {
                "Archived"
            } else {
                "Failed"
            },
            response.status(),
            url
        );
    }
}
