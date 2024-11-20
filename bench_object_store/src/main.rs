#![allow(clippy::unwrap_used, clippy::disallowed_types)]
use std::collections::HashMap;
use std::str::FromStr;

use bench_object_store::s3::S3Scan;
use bench_object_store::{RunScan, Scan};
use clap::{Args, Parser, Subcommand, ValueEnum};
use log::{debug, info, trace, LevelFilter};
use object_store::aws::AmazonS3ConfigKey;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

#[derive(Parser, Debug)]
#[command(version, about)]
#[command(propagate_version = true)]
struct Cli {
    /// URL path of the Vortex file to scan.
    #[command(subcommand)]
    source: Source,

    /// Type of scan to perform.
    #[arg(long)]
    scan: ScanMode,

    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,
}

// Includes some sort of file scan behavior here instead.
#[derive(Subcommand, Debug)]
enum Source {
    /// Read from a storage engine that exposes the Amazon S3 protocol.
    S3(S3Args),
}

#[derive(Args, Debug)]
struct S3Args {
    /// Bucket to fetch the object from
    #[arg(long)]
    bucket: String,

    /// Path within the bucket that holds the Vortex file to scan
    #[arg(long)]
    file: String,
}

// Create different run variants.
#[derive(ValueEnum, Debug, Clone, Copy)]
enum ScanMode {
    /// Perform a full scan, reading all rows.
    Full,
    // TODO: add Filter, allow specifying a filter expression.
}

fn make_runner(source: Source) -> RunScan<Box<dyn Scan>> {
    match source {
        Source::S3(s3) => {
            // Collect all AWS env vars.
            debug!("Building s3 client from environment");
            let discovered_configs: HashMap<AmazonS3ConfigKey, String> = std::env::vars_os()
                .filter_map(|(os_key, os_value)| {
                    if let (Some(key), Some(value)) = (os_key.to_str(), os_value.to_str()) {
                        if key.starts_with("AWS_") {
                            let normalized_key = key.to_lowercase();
                            if let Ok(config) = AmazonS3ConfigKey::from_str(normalized_key.as_str())
                            {
                                return Some((config, value.to_string()));
                            }
                        }
                    }

                    None
                })
                .collect();
            debug!("S3 configs discovered from environment: {discovered_configs:?}");
            let scan = S3Scan::new(s3.bucket.as_str());
            RunScan::new(Box::new(scan), s3.file)
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Configure logging.
    let level = match cli.verbose {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    TermLogger::init(
        level,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .unwrap();

    // Find and load an available .env file
    if let Ok(path_buf) = dotenv::dotenv() {
        trace!(
            "loaded extra environment variables from {}",
            path_buf.display()
        );
    }

    // Return a thing that knows how to construct a scan this way instead.
    let runner = make_runner(cli.source);

    match cli.scan {
        ScanMode::Full => {
            // Run a full scan
            info!("Begin full scan test");
            runner.full_scan().await;
            info!("Full scan test complete");
        }
    }
}
