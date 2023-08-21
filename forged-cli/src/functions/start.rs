use cynic::MutationBuilder;

use crate::{queries::CreateDevice, Result};

pub async fn start(client: &mut forged::Client) -> Result<()> {
    eprintln!("🚀 Creating a new device ...");
    client.run_query(CreateDevice::build(())).await?;
    Ok(())
}
