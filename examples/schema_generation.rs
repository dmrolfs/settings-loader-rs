use serde::Deserialize;
use settings_loader::metadata::{Constraint, SettingMetadata, SettingType};
use settings_loader::registry;
use settings_loader::SettingsLoader;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MySettings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
    pub logging: LoggingSettings,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ServerSettings {
    pub port: u16,
    pub host: String,
    pub timeout: String, // Represented as Duration string
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DatabaseSettings {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LoggingSettings {
    pub level: String,
    pub format: String,
}

impl SettingsLoader for MySettings {
    type Options = settings_loader::NoOptions;

    fn app_config_basename() -> &'static str {
        "example_app"
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize the registry
    registry::init_global_registry("Demo App", "1.2.3");

    // 2. Register some metadata
    registry::register_setting(SettingMetadata {
        key: "server.port".to_string(),
        label: "Server Port".to_string(),
        description: "The port the HTTP server will listen on.".to_string(),
        setting_type: SettingType::Integer { min: Some(1024), max: Some(65535) },
        default: Some(serde_json::json!(8080)),
        constraints: vec![Constraint::Required, Constraint::Range { min: 1024.0, max: 65535.0 }],
        visibility: settings_loader::metadata::Visibility::Public,
        group: Some("Server".to_string()),
    });

    registry::register_setting(SettingMetadata {
        key: "server.host".to_string(),
        label: "Listen Address".to_string(),
        description: "The network interface to bind to.".to_string(),
        setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
        default: Some(serde_json::json!("127.0.0.1")),
        constraints: vec![],
        visibility: settings_loader::metadata::Visibility::Public,
        group: Some("Server".to_string()),
    });

    registry::register_setting(SettingMetadata {
        key: "server.timeout".to_string(),
        label: "Request Timeout".to_string(),
        description: "Maximum time to wait for a request to complete.".to_string(),
        setting_type: SettingType::Duration { min: None, max: None },
        default: Some(serde_json::json!("30s")),
        constraints: vec![],
        visibility: settings_loader::metadata::Visibility::Public,
        group: Some("Server".to_string()),
    });

    registry::register_setting(SettingMetadata {
        key: "database.url".to_string(),
        label: "Database Connection String".to_string(),
        description: "PostgreSQL connection URL.".to_string(),
        setting_type: SettingType::Url {
            schemes: vec!["postgres".to_string(), "postgresql".to_string()],
        },
        default: Some(serde_json::json!("postgres://localhost:5432/demo")),
        constraints: vec![Constraint::Required],
        visibility: settings_loader::metadata::Visibility::Secret,
        group: Some("Database".to_string()),
    });

    registry::register_setting(SettingMetadata {
        key: "logging.level".to_string(),
        label: "Log Level".to_string(),
        description: "The verbosity of the application logs.".to_string(),
        setting_type: SettingType::Enum {
            variants: vec![
                "trace".to_string(),
                "debug".to_string(),
                "info".to_string(),
                "warn".to_string(),
                "error".to_string(),
            ],
        },
        default: Some(serde_json::json!("info")),
        constraints: vec![],
        visibility: settings_loader::metadata::Visibility::Public,
        group: Some("Logging".to_string()),
    });

    // 3. Export everything!
    let out_dir = Path::new("target/demo_export");
    std::fs::create_dir_all(out_dir)?;

    println!("Exporting metadata to {:?}...", out_dir);

    // Export JSON Schema
    let schema_path = out_dir.join("schema.json");
    MySettings::export_json_schema(&schema_path)?;
    println!("  - JSON Schema: {:?}", schema_path);

    // Export HTML Documentation
    let docs_path = out_dir.join("documentation.html");
    MySettings::export_docs(&docs_path)?;
    println!("  - HTML Docs:   {:?}", docs_path);

    // Export Example Config
    let config_path = out_dir.join("application.example.toml");
    MySettings::export_example_config(&config_path)?;
    println!("  - Example TOML: {:?}", config_path);

    println!("\nExport complete! Take a look at the generated files.");

    Ok(())
}
