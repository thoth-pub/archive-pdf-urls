use log::{error, info};
use std::io::{self, BufRead};
use waybackmachine_client::{ArchiveResult, ClientConfig, WaybackMachineClient};

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
            Ok(ArchiveResult::Archived(archive_url)) => {
                info!("Archived: {} – {}", url, archive_url)
            }
            Ok(ArchiveResult::RecentArchiveExists(archive_url)) => {
                info!("Skipped: {} – {}", url, archive_url)
            }
            Err(e) => error!("{}", e),
        }
    }
}
