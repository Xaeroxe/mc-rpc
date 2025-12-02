# mc-rpc

Fully generated rust bindings for **[Minecraft Server Management Protocol](https://minecraft.wiki/w/Minecraft_Server_Management_Protocol)**.  

All types, request methods and notification methods are fully generated.  
Built with **[pale](https://github.com/VilleOlof/pale)** to get a smooth websocket connection in the background that tries to reconnect when the connection drops.  


## Example
```rust
use mc_rpc::{Client, ClientConfig, Difficulty, Result, StreamExt};

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
    while let Some(_) = client.notification_server_saved().await?.next().await {
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

Some examples on how the crate converts the RPC schema to rust code.  

### Examples

#### Structs
```json
"operator": {
    "properties": {
        "bypassesPlayerLimit": {
            "type": "boolean"
        },
        "permissionLevel": {
            "type": "integer"
        },
        "player": {
            "$ref": "#/components/schemas/player"
        }
    },
    "type": "object"
},
```
```rust
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Operator {
    #[serde(rename = "bypassesPlayerLimit")]
    pub bypasses_player_limit: bool,
    #[serde(rename = "permissionLevel")]
    pub permission_level: i32,
    pub player: Player
}
```
#### Enums
```json
"game_type": {
    "enum": [
        "survival",
        "creative",
        "adventure",
        "spectator"
    ],
    "type": "string"
},
```
```rust
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub enum GameType {
    #[serde(rename = "survival")]
    Survival,
    #[serde(rename = "creative")]
    Creative,
    #[serde(rename = "adventure")]
    Adventure,
    #[serde(rename = "spectator")]
    Spectator
}
```
#### Request Functions
```json
"description": "Send a system message",
"name": "minecraft:server/system_message",
"params": [
    {
        "name": "message",
        "required": true,
        "schema": {
            "$ref": "#/components/schemas/system_message"
        }
    }
],
"result": {
    "name": "sent",
    "schema": {
        "type": "boolean"
    }
}
```
```rust
/// Send a system message
pub async fn server_system_message(&self, message: SystemMessage) -> Result<bool> {
    let mut map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    map.insert("message".to_string(), serde_json::to_value(&message)?);
    self.0.request("minecraft:server/system_message", Some(map)).await
}
```
#### Notification Functions
```json
"description": "Player joined",
"name": "minecraft:notification/players/joined",
"params": [
    {
        "name": "player",
        "required": true,
        "schema": {
            "$ref": "#/components/schemas/player"
        }
    }
]
```
```rust
/// Player joined
pub async fn notification_players_joined(&self) -> Result<impl Stream<Item = Option<std::result::Result<Vec<Player>, serde_json::Error>>>> {
    self.0.subscribe("minecraft:notification/players/joined").await
}
```