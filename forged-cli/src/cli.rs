use clap::{Parser, Subcommand};
use serde_json::Value;

/// Doc comment
#[derive(Parser)]
#[clap(name = "provisioned")]
#[clap(bin_name = "provisioner")]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// The provisioner token that should be used for API access.
    #[clap(long)]
    pub api_token: Option<String>,

    /// The API instance URl which should be used to store the device data.
    #[clap(long)]
    pub api_instance_url: Option<String>,

    #[clap(subcommand)]
    pub command: Command,
}

/// Doc comment
#[derive(Subcommand)]
pub enum Command {
    /// Creates a new device.
    Start,

    /// Downloads the binary and device data to the target.
    Download,

    /// Creates a new log entry on the device in the current session.
    #[clap(subcommand)]
    Log(LogOption),

    /// Attaches a file to the current device event log.
    Attach {
        /// The path to the file to be attached to the test report.
        file_path: String,
    },

    /// Adds a new block to the current device.
    Block { schema_name: String, data: Value },

    /// Finishes the current device procurment procedure.
    End,
}

#[derive(Subcommand)]
pub enum LogOption {
    /// Generate logs from an input stream source.
    Stream {
        /// An optional filename to stream logs from. If unspecified, STDIN is used.
        filename: Option<String>,
    },

    /// Generate a single log entry.
    Entry {
        /// The level of the message to be logged to the cloud.
        level: String,
        /// The message to be logged to the cloud.
        message: String,
    },
}
