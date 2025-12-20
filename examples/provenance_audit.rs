use serde::Deserialize;
use settings_loader::{LayerBuilder, LoadingOptions, SettingsError, SettingsLoader};
use std::fs;
use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct MyAppSettings {
    app_name: String,
    port: u16,
    db: DatabaseSettings,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct DatabaseSettings {
    host: String,
    port: u16,
}

impl SettingsLoader for MyAppSettings {
    type Options = NoOptions;
}

#[derive(Debug)]
struct NoOptions;
impl LoadingOptions for NoOptions {
    type Error = SettingsError;

    fn config_path(&self) -> Option<PathBuf> {
        None
    }

    fn secrets_path(&self) -> Option<PathBuf> {
        None
    }

    fn implicit_search_paths(&self) -> Vec<PathBuf> {
        vec![]
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a temporary directory for config files
    let temp_dir = tempfile::tempdir()?;
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(
        &config_path,
        "app_name: \"AuditDemo\"\nport: 8080\ndb:\n  host: \"localhost\"\n  port: 5432",
    )?;

    // 2. Set some environment variables for overrides
    // DEMO_PORT overrides the 'port' field
    // DEMO_DB_HOST overrides 'db.host'
    std::env::set_var("DEMO_PORT", "9999");
    std::env::set_var("DEMO_DB_HOST", "prod-db.internal");

    // 3. Use LayerBuilder to construct the configuration stack
    let builder = LayerBuilder::new()
        // Layer 0: Default file
        .with_path(&config_path)
        // Layer 1: EnvVars prefix search for port, db.host, etc.
        .with_env_vars("DEMO", "_");

    // 4. Build with provenance to get the source map
    let (config_builder, source_map) = builder.build_with_provenance()?;
    let config = config_builder.build()?;
    let settings: MyAppSettings = config.try_deserialize()?;

    // 5. Print the settings (to avoid dead_code warnings) and the audit report
    println!("=== Loaded Settings ===\n");
    println!("{:#?}", settings);
    println!("\n=== Configuration Audit Report ===\n");
    println!("{}", source_map.audit_report());

    // Clean up env vars
    std::env::remove_var("DEMO_PORT");
    std::env::remove_var("DEMO_DB_HOST");

    Ok(())
}
