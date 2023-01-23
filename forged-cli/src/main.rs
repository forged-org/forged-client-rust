mod cli;
mod functions;
mod queries;

use std::{env, fmt::Display};

use clap::StructOpt;
use dotenv::dotenv;
use functions::block::block;

use crate::{
    cli::{Cli, Command},
    functions::{attach::attach, download::download, end::end, log::log, start::start},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    dotenv().ok();
    let token = cli
        .api_token
        .or_else(|| env::var("FORGED_TOKEN").ok())
        .expect("FORGED_TOKEN is not set");

    let mut client = forged::Client::new(token);

    if let Some(endpoint) = cli.api_instance_url.or_else(|| env::var("FORGED_INSTANCE_URL").ok()) {
        client = client.api(endpoint);
    }

    match cli.command {
        Command::Start => start(&mut client).await?,
        Command::Download => download(&mut client).await?,
        Command::Log(option) => log(&mut client, option).await?,
        Command::Attach { file_path } => attach(&mut client, file_path).await?,
        Command::Block { data, schema_name } => block(&mut client, schema_name, data).await?,
        Command::End => end(&mut client).await?,
    }

    Ok(())
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    Client(#[from] forged::Error),
    Probe(#[from] probe_rs::Error),
    FlashOperation(#[from] probe_rs_cli_util::common_options::OperationError),
    Other(#[from] anyhow::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Client(error) => writeln!(f, "forged: {error}"),
            Error::Probe(error) => {
                writeln!(f, "An error with the probe occured:")?;
                writeln!(f, "{error}")
            }
            Error::Other(error) => writeln!(f, "{error}"),
            Error::FlashOperation(error) => writeln!(f, "{error}"),
        }
    }
}
