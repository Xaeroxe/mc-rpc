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
    // Attempt to extract the version identifier
    let version = schema.get("info").and_then(|i| i.get("version"));
    let file_name = if let Some(version) = version.and_then(|v| v.as_str()) {
        format!("schema-{version}.json")
    } else {
        "schema.json".to_string()
    };
    let formatter = PrettyFormatter::with_indent(b"    ");
    let mut serializer = Serializer::with_formatter(
        BufWriter::new(
            File::create(format!("{}/../{file_name}", env!("CARGO_MANIFEST_DIR"))).unwrap(),
        ),
        formatter,
    );
    schema
        .serialize(&mut serializer)
        .expect("Failed to write new schema file");

    Ok(())
}
