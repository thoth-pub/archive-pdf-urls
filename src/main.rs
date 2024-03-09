use log::{error, info};
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
        match client.archive_url(&url).await {
            Ok(_) => info!("Archived: {}", url),
            Err(e) => error!("{}", e),
        }
    }
}
