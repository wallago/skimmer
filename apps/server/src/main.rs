//! Command line entry point for `skimmer`.

use std::process::ExitCode;

use clap::Parser;
use skimmer::prelude::*;
use tracing::Level;

/// Runs the crate and maps the outcome to a process exit code.
#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    let level = match args.verbose {
        0 => Level::WARN, // default: warnings + errors only
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE, // -vvv and beyond
    };
    let under_journal = std::env::var_os("JOURNAL_STREAM").is_some();
    let fmt = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(std::io::stderr);
    if under_journal {
        fmt.without_time().with_ansi(false).init();
    } else {
        fmt.init();
    }
    match skimmer::run(&args).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}: {err}", version());
            ExitCode::FAILURE
        }
    }
}
