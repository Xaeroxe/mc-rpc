use std::fs::write;

use pale::{Client, ClientConfig, Result};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().unwrap();

    let client = Client::new(
        dotenvy::var("management_server_url").unwrap(),
        ClientConfig::with_bearer(&dotenvy::var("management_server_secret").unwrap()),
    )
    .await?;
    let schema: Value = client.request("rpc.discover", None).await?;

    let json = serde_json::to_string_pretty(&schema)?;

    write(concat!(env!("CARGO_MANIFEST_DIR"), "/../schema.json"), json)
        .expect("Failed to write new schema.json file");

    Ok(())
}
