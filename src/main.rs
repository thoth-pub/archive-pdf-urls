use clap::{crate_authors, crate_version, Arg, ArgAction, Command};
use log::{error, info};
use lopdf::{Dictionary, Document, Object};
use regex::Regex;
use std::collections::HashSet;
use waybackmachine_client::{ArchiveResult, ClientConfig, Error, WaybackMachineClient};

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("file")
                .value_name("FILE")
                .help("Sets the input PDF file to use")
                .required(true),
        )
        .arg(
            Arg::new("exclude")
                .long("exclude")
                .value_name("PATTERN")
                .help("Excludes URLs matching the pattern")
                .required(false)
                .action(ArgAction::Append),
        )
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_target(false)
        .init();

    let args = cli().get_matches();
    let pdf_file = args.get_one::<String>("file").unwrap();
    let doc = match Document::load(pdf_file) {
        Ok(doc) => doc,
        Err(err) => {
            error!("Error loading PDF file: {}", err);
            std::process::exit(1);
        }
    };
    let regex_patterns: Vec<Regex> = args
        .get_many::<String>("exclude")
        .unwrap_or_default()
        .map(|pattern| Regex::new(pattern).expect("Invalid regex pattern"))
        .collect();

    let links_set = extract_links(doc);
    let client = WaybackMachineClient::new(ClientConfig::default());

    let mut exit_code = 0;
    for url in links_set.into_iter() {
        if regex_patterns.iter().any(|regex| regex.is_match(&url)) {
            info!("Skipped: {}", url);
            continue;
        }

        match client.archive_url(&url).await {
            Ok(ArchiveResult::Archived(archive_url)) => {
                info!("Archived: {} – {}", url, archive_url)
            }
            Ok(ArchiveResult::RecentArchiveExists) => {
                info!("Skipped: {}", url)
            }
            Err(Error::ExcludedUrl(url)) => {
                info!("Skipped: {}", url)
            }
            Err(e) => {
                error!("{}", e);
                // Set exit code to failure (1) if any URL fails to archive
                exit_code = 1;
            }
        }
    }
    std::process::exit(exit_code);
}

// Extract all Links from a PDF
fn extract_links(doc: Document) -> HashSet<String> {
    let mut links_set = HashSet::new();

    for page_id in doc.page_iter() {
        for annotation in doc.get_page_annotations(page_id) {
            if is_link_annotation(annotation) {
                if let Some(dest) = extract_link_dest(annotation, &doc) {
                    links_set.insert(dest);
                }
            }
        }
    }
    links_set
}

// Check if the annotation is a link
fn is_link_annotation(annotation: &Dictionary) -> bool {
    annotation
        .get(b"Subtype")
        .ok()
        .and_then(|subtype| subtype.as_name().ok())
        .map_or(false, |subtype| subtype == b"Link")
}

// Extract the destination URI from a link annotation
fn extract_link_dest(annotation: &Dictionary, document: &Document) -> Option<String> {
    if let Ok(Object::Reference(dest_ref)) = annotation.get(b"A") {
        if let Ok(Object::Dictionary(dest_dict)) = document.get_object(*dest_ref) {
            // Assuming the destination dictionary has a URI
            if let Ok(Object::String(uri, _)) = dest_dict.get(b"URI") {
                return Some(String::from_utf8_lossy(uri).into_owned());
            }
        }
    }
    None
}
