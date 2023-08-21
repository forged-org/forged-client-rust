use cynic::MutationBuilder;

use crate::{queries::CreateDevice, Result};

pub async fn start(client: &mut forged::Client) -> Result<()> {
    eprintln!("🚀 Creating a new device ...");
    let start = std::time::Instant::now();
    client.run_query(CreateDevice::build(())).await?;
    println!("{:?}", start.elapsed());
    Ok(())
}
