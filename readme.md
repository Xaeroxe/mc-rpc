# mc-rpc

Fully generated rust bindings for **[Minecraft Server Management Protocol](https://minecraft.wiki/w/Minecraft_Server_Management_Protocol)**.  

All types, request methods and notification methods are fully generated.  
Built with **[pale](https://github.com/VilleOlof/pale)** to get a smooth websocket connection in the background that tries to reconnect when the connection drops.  


## Example
```rust
use mc_rpc::{Client, Difficulty, pale::{ClientConfig, Result}};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new client
    let client = Client::new(
            "wss://example.com",
            ClientConfig::with_bearer("<secret>")
        ).await?;

    // Fetch all players online
    let players = client.players().await?;

    // Get notified when the server is saved
    while let Some(_) = client.sub_server_saved().await?.next().await {
        println!("Server just got saved");
    }

    // Change the difficulty
    client.serversettings_difficulty_set(Difficulty::Peaceful).await?;

    // Stop the server
    client.server_stop().await?;

    Ok(())
}
```

## Version

This crate is currently built on `Minecraft Server JSON-RPC Version: 2.0.0 (25w44a)`.  

## build.rs

Just also gonna warn you that the code in `build.rs` for generating the bindings is abysmal dogshit & the worst rust code i've written but if it works it works.  
And before this crate ever gets published or anything it needs HEAVY refactoring because like, dont look at the coooode plsss.  