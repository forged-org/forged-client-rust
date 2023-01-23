use cynic::MutationBuilder;
use serde_json::Value;

use crate::{
    queries::{CreateBlock, CreateBlockArguments},
    Result,
};

pub async fn block(
    client: &mut forged::Client,
    schema_name: String,
    data: Value,
) -> Result<()> {
    println!("📎  Creating block:");
    println!("{data:#}");

    client
        .run_query(CreateBlock::build(&CreateBlockArguments {
            schema_name,
            data,
        }))
        .await?;

    Ok(())
}
