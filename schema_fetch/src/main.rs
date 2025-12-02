use pale::{Client, ClientConfig, Result};
use serde::ser::Serialize;
use serde_json::ser::PrettyFormatter;
use serde_json::{Serializer, Value};
use std::fs::File;
use std::io::BufWriter;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().unwrap();

    let client = Client::new(
        dotenvy::var("management_server_url").unwrap(),
        ClientConfig::with_bearer(&dotenvy::var("management_server_secret").unwrap()),
    )
    .await?;
    let schema: Value = client.request("rpc.discover", None).await?;
    let formatter = PrettyFormatter::with_indent(b"    ");
    let mut serializer = Serializer::with_formatter(
        BufWriter::new(
            File::create(concat!(env!("CARGO_MANIFEST_DIR"), "/../schema.json")).unwrap(),
        ),
        formatter,
    );
    schema
        .serialize(&mut serializer)
        .expect("Failed to write new schema.json file");

    Ok(())
}
