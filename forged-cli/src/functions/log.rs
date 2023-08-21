use crate::{
    cli::LogOption,
    queries::{CreateLog, CreateLogArguments},
    Result,
};
use anyhow::anyhow;
use cynic::MutationBuilder;

use std::io::{prelude::*, BufReader, Read};

const SUPPORTED_LEVELS: [&str; 6] = ["CRITICAL", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];

/// Parse a line entry into a log entry.
///
/// # Note
/// Log entries generally have the form `<LEVEL> <MESSAGE>` and are parsed as such. Any data before
/// `<LEVEL>` is omitted.
///
/// # Returns
/// The parsed log entry. If no level was found, INFO is used.
fn parse_log_entry(line: &str) -> CreateLogArguments {
    for level in SUPPORTED_LEVELS {
        if let Some(index) = line.to_ascii_uppercase().find(level) {
            return CreateLogArguments {
                level: level.to_string(),
                message: line.split_at(index + level.len()).1.trim().to_string(),
            };
        }
    }

    CreateLogArguments {
        level: "INFO".to_string(),
        message: line.to_string(),
    }
}

async fn generate_log(client: &forged::Client, args: &CreateLogArguments) -> Result<()> {
    println!("🪵  Logging: [{}] {}", args.level, args.message);
    client.run_query(CreateLog::build(args)).await?;
    Ok(())
}

/// Generate log entries from an input file or STDIN.
///
/// # Args
/// * `token` - The provisioner authentication token.
/// * `instance_url` - The URL of the instance to generate logs on.
/// * `file` - An optional file to generate logs from. If `None`, STDIN will be used.
pub async fn log(client: &mut forged::Client, options: LogOption) -> Result<()> {
    match options {
        LogOption::Stream { filename } => {
            let input: Box<dyn Read> = if let Some(filename) = filename {
                let file = std::fs::File::open(filename)
                    .map_err(|e| anyhow!("Failed to open file: {e}"))?;
                Box::new(file)
            } else {
                Box::new(std::io::stdin())
            };

            for line in BufReader::new(input).lines() {
                let parsed_log =
                    parse_log_entry(&line.map_err(|e| anyhow!("Failed to read line: {e}"))?);
                generate_log(client, &parsed_log).await?;
            }
        }

        LogOption::Entry { level, message } => {
            generate_log(client, &CreateLogArguments { level, message }).await?;
        }
    }

    Ok(())
}
