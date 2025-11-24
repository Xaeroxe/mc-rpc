use std::{env, fs::write};

fn main() {
    let json_schema = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/schema.json"
    )))
    .expect("Failed to deserialize RPC Schema");

    let code = generate(json_schema).expect("Failed to generate json rpc bindings");

    write(
        format!("{}/json_rpc_bindings.rs", env::var("OUT_DIR").unwrap()),
        code,
    )
    .expect("Failed to write json_rpc_bindings.rs");
}

// absolute dogshit, panic full, horrible piss code that generates json rpc bindings from a schema file below
// you've been warned for horrible absolute dogshit code.

// TODO: Need to really fix everything but the most important one is gamerules
// Untyped should work, but typed needs fixing and both untyped & typed should be 100% dynamically generated
// but currently they are both half-constant in the code generation due to them being fucked up. fuck those.

fn generate(schema: serde_json::Value) -> anyhow::Result<String> {
    let mut code = String::new();

    // info
    code.push_str(&format!(
        "// Minecraft Server JSON-RPC Version: {}",
        schema["info"]["version"]
            .as_str()
            .ok_or(anyhow::format_err!("No info.title"))?
    ));

    add_dependencies(&mut code);
    add_base_lib(&mut code);
    generate_types(&schema, &mut code)?;
    generate_request_methods(&schema, &mut code)?;
    generate_notification_methods(&schema, &mut code)?;

    Ok(code)
}

fn format_struct_name(name: &str) -> String {
    name.split("_")
        .map(|f| {
            let mut chars = f.chars().collect::<Vec<char>>();
            chars[0] = chars[0].to_ascii_uppercase();
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<String>>()
        .join("")
}

fn format_field_name(name: &str) -> String {
    if name == "type" {
        return "pub _type".to_string();
    }

    let chars = name.chars().collect::<Vec<char>>();
    let mut new_name = String::new();
    let mut is_renamed = false;
    for char in chars {
        if char.is_ascii_uppercase() {
            new_name.push('_');
            is_renamed = true;
        }

        new_name.push(char.to_ascii_lowercase());
    }

    if is_renamed {
        return format!("#[serde(rename = \"{name}\")]\n    pub {new_name}");
    }

    format!("pub {new_name}")
}

fn get_rust_type(type_data: &serde_json::Value) -> anyhow::Result<String> {
    fn primitive_type(_type: &str) -> anyhow::Result<&'static str> {
        Ok(match _type {
            "string" => "String",
            "boolean" => "bool",
            "integer" => "i32",
            _ => anyhow::bail!("Invalid schema type"),
        })
    }
    if let Some(_ref) = type_data.get("$ref").map(|s| s.as_str().unwrap()) {
        let ref_name = _ref
            .split("/")
            .last()
            .ok_or(anyhow::format_err!("Invalid schema ref name"))?;
        return Ok(format_struct_name(ref_name));
    }

    let _type = type_data["type"].as_str().unwrap();
    if _type == "array" {
        if let Some(_ref) = type_data["items"].get("$ref").map(|s| s.as_str().unwrap()) {
            let ref_name = _ref
                .split("/")
                .last()
                .ok_or(anyhow::format_err!("Invalid schema ref name"))?;
            return Ok(format!("Vec<{}>", format_struct_name(ref_name)));
        }

        return Ok(format!(
            "Vec<{}>",
            primitive_type(type_data["items"]["type"].as_str().unwrap())?
        ));
    }

    Ok(primitive_type(_type)?.to_string())
}

fn add_dependencies(code: &mut String) {
    code.push_str(
        r#"
use serde::{Deserialize, Serialize};
pub use pale::{ClientConfig, Result, PaleError, RPCError, StreamExt, WebSocketConfig};
"#,
    );
}

fn add_base_lib(code: &mut String) {
    code.push_str(
            r#"
    #[derive(Debug, Clone)]
    pub struct Client(pub(crate) pale::Client);

    impl Client {
        pub async fn new(uri: impl AsRef<str>, config: ClientConfig) -> Result<Self> {
            Ok(Self(pale::Client::new(uri, config).await?))
        }
    }

    macro_rules! params {
        ( $( ($key:expr, $value:expr) ),* $(,)? ) => {{
            let mut map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
            $(
                map.insert(
                    $key.to_string(),
                    serde_json::to_value(&$value)?,
                );
            )*
            Some(map)
        }};
    }

    #[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
    #[serde(untagged)]
    pub enum IntegerOrBoolean {
        Boolean(bool),
        Integer(i32)
    }
    "#,
        );
}

fn generate_types(schema: &serde_json::Value, code: &mut String) -> anyhow::Result<()> {
    let schemas = schema["components"]["schemas"]
        .as_object()
        .ok_or(anyhow::format_err!("components.schemas is not an object"))?;

    for (name, data) in schemas {
        println!("{name}, {data:?}");

        // TODO: move this into a dynamic one and where the enum used here is generated and not constant
        // i dont wanna fucking deal with this "oooh a value can be either one of 2 types bullshit"
        if name == "typed_game_rule" || name == "untyped_game_rule" {
            code.push_str(&format!(
                r#"
pub type {} = std::collections::HashMap<String, IntegerOrBoolean>;
"#,
                format_struct_name(name)
            ));

            continue;
        }

        // handle enum types
        if let Some(enum_data) = data.get("enum").map(|e| e.as_array().unwrap()) {
            code.push_str(&format!(
                r#"
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub enum {} {{
{}
}}
"#,
                format_struct_name(name),
                enum_data
                    .iter()
                    .map(|v| {
                        let name = v.as_str().unwrap();
                        let variant_name = format_struct_name(name);
                        format!("    #[serde(rename = \"{name}\")]\n    {variant_name}")
                    })
                    .collect::<Vec<String>>()
                    .join(",\n")
            ));

            continue;
        }
        // only structs past here
        if data["type"].as_str().ok_or(anyhow::format_err!(
            "components.schemas.type is not a string"
        ))? != "object"
        {
            continue;
        }

        let props = data["properties"].as_object().ok_or(anyhow::format_err!(
            "components.schemas[x].properties is not an object"
        ))?;
        let mut fields = String::new();

        for (i, (field_name, type_data)) in props.iter().enumerate() {
            let _type = get_rust_type(&type_data)?;
            let name = format_field_name(&field_name);
            fields.push_str(&format!(
                "    {name}: {_type}{}",
                if i != (props.len() - 1) { ",\n" } else { "" }
            ));
        }

        code.push_str(&format!(
            r#"
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct {} {{
{fields}
}}
"#,
            format_struct_name(name)
        ));
    }

    Ok(())
}

fn get_function_name(name: &str) -> String {
    name.replace("minecraft:", "")
        .replace("notification:", "")
        .replace("minecraft:notification/", "")
        .replace("/", "_")
}

fn generate_request_methods(schema: &serde_json::Value, code: &mut String) -> anyhow::Result<()> {
    let methods: Vec<&serde_json::Map<String, serde_json::Value>> = schema["methods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_object().unwrap())
        .collect();

    let mut rust_methods = String::new();

    for method in methods {
        let docs = format!(
            "/// {}",
            method["description"]
                .as_str()
                .ok_or(anyhow::format_err!("methods.description is not a string"))?
        );
        let endpoint = method["name"]
            .as_str()
            .ok_or(anyhow::format_err!("methods.name is not a string"))?;
        if endpoint.starts_with("minecraft:notification") {
            continue;
        }

        let fn_name = get_function_name(endpoint);
        let arg_names = method["params"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| {
                let name = n["name"].as_str().unwrap();
                format!(
                    "(\"{name}\", {})",
                    if name == "use" {
                        format!("_{name}")
                    } else {
                        name.to_string()
                    }
                )
            })
            .collect::<Vec<String>>();
        let arg_params = method["params"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                let param_type = get_rust_type(&p["schema"]).unwrap();
                let name = p["name"].as_str().unwrap();
                format!(
                    "{}: {}",
                    if name == "use" {
                        format!("_{name}")
                    } else {
                        name.to_string()
                    },
                    param_type
                )
            })
            .collect::<Vec<String>>();
        let return_type = get_rust_type(&method["result"]["schema"])?;

        rust_methods.push_str(&format!(
            r#"    {docs}
    pub async fn {fn_name}(&self{}) -> Result<{return_type}> {{
        self.0.request("{endpoint}", {}).await
    }}
"#,
            if arg_params.len() > 0 {
                &format!(", {}", arg_params.join(", "))
            } else {
                ""
            },
            if arg_names.len() > 0 {
                &format!("params![{}]", arg_names.join(","))
            } else {
                "None"
            }
        ));
    }

    code.push_str(&format!(
        r#"
impl Client {{
{rust_methods}
}}
"#
    ));

    Ok(())
}

fn generate_notification_methods(
    schema: &serde_json::Value,
    code: &mut String,
) -> anyhow::Result<()> {
    let methods: Vec<&serde_json::Map<String, serde_json::Value>> = schema["methods"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_object().unwrap())
        .collect();

    let mut rust_methods = String::new();

    for method in methods {
        let endpoint = method["name"]
            .as_str()
            .ok_or(anyhow::format_err!("methods.name is not a string"))?;
        if !endpoint.starts_with("minecraft:notification") {
            continue;
        }

        let docs = method["description"]
            .as_str()
            .ok_or(anyhow::format_err!("methods.description is not a string"))?;
        let fn_name = get_function_name(endpoint);

        let return_type = if let Some(return_elem) = method["params"]
            .as_array()
            .ok_or(anyhow::format_err!("methods.params is not a string"))?
            .first()
        {
            get_rust_type(&return_elem["schema"])?
        } else {
            "()".to_string()
        };

        rust_methods.push_str(&format!(
            r#"    /// {docs}
    pub async fn sub_{fn_name}(&self) -> Result<impl tokio_stream::Stream<Item = Option<{return_type}>>> {{
        self.0.subscribe("{endpoint}").await
    }}
"#));
    }

    code.push_str(&format!(
        r#"
impl Client {{
{rust_methods}
}}
"#
    ));

    Ok(())
}
