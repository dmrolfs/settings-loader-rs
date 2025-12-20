use serde::Deserialize;
use settings_loader::{LayerBuilder, SourceType};
use std::fs;
use tempfile::TempDir;

#[derive(Debug, Deserialize, PartialEq)]
struct TestSettings {
    app_name: String,
    port: Option<u16>,
    database: Option<DatabaseSettings>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct DatabaseSettings {
    host: String,
}

#[test]
fn test_provenance_tracking_single_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("settings.toml");

    fs::write(
        &config_path,
        r#"
        app_name = "test_app"
        port = 8080
    "#,
    )
    .unwrap();

    let builder = LayerBuilder::new().with_path(&config_path);
    let (config, source_map) = builder.build_with_provenance().unwrap();

    let settings: TestSettings = config.build().unwrap().try_deserialize().unwrap();

    assert_eq!(settings.app_name, "test_app");

    // Check provenance
    let meta = source_map.source_of("app_name").expect("should have metadata");
    assert_eq!(meta.source_type, SourceType::File);
    assert_eq!(
        meta.path.as_ref().map(|p| p.canonicalize().unwrap()),
        Some(config_path.canonicalize().unwrap())
    );
}

#[test]
fn test_provenance_tracking_overrides() {
    let temp_dir = TempDir::new().unwrap();
    let layer1_path = temp_dir.path().join("layer1.toml");
    let layer2_path = temp_dir.path().join("layer2.toml");

    fs::write(
        &layer1_path,
        r#"
        app_name = "layer1"
        port = 8080
    "#,
    )
    .unwrap();

    fs::write(
        &layer2_path,
        r#"
        app_name = "layer2"
    "#,
    )
    .unwrap();

    let builder = LayerBuilder::new().with_path(&layer1_path).with_path(&layer2_path);

    let (config, source_map) = builder.build_with_provenance().unwrap();
    let settings: TestSettings = config.build().unwrap().try_deserialize().unwrap();

    assert_eq!(settings.app_name, "layer2");
    assert_eq!(settings.port, Some(8080));

    // Check provenance for app_name (should be layer2)
    let meta_app = source_map.source_of("app_name").unwrap();
    assert_eq!(
        meta_app.path.as_ref().map(|p| p.canonicalize().unwrap()),
        Some(layer2_path.canonicalize().unwrap())
    );

    // Check provenance for port (should be layer1)
    let meta_port = source_map.source_of("port").unwrap();
    assert_eq!(
        meta_port.path.as_ref().map(|p| p.canonicalize().unwrap()),
        Some(layer1_path.canonicalize().unwrap())
    );
}

#[test]
fn test_provenance_scoped_path_compilation() {
    // This test mainly verifies that ScopedPath handling compiles and runs without error.
    // Detailed scope resolution testing requires more complex setup or mocking.

    struct TestConfig;
    impl settings_loader::LoadingOptions for TestConfig {
        type Error = settings_loader::SettingsError;
        fn config_path(&self) -> Option<std::path::PathBuf> {
            None
        }
        fn secrets_path(&self) -> Option<std::path::PathBuf> {
            None
        }
        fn implicit_search_paths(&self) -> Vec<std::path::PathBuf> {
            vec![]
        }
    }
    impl settings_loader::MultiScopeConfig for TestConfig {
        const APP_NAME: &'static str = "test";
        fn find_config_in(_dir: &std::path::Path) -> Option<std::path::PathBuf> {
            None
        }
    }

    let builder = LayerBuilder::new().with_scopes::<TestConfig>(vec![]);
    let (_config, _source_map) = builder.build_with_provenance().unwrap();
}

#[test]
fn test_provenance_env_var() {
    // We must use strict environment variable naming convention
    // With prefix="TEST" and separator="__", "TEST__APP_NAME" maps to "app_name".
    std::env::set_var("TEST__APP_NAME", "env_app");

    // Use with_env_vars (plural) which maps system env vars to config keys
    let builder = LayerBuilder::new().with_env_vars("TEST", "__");
    let (config, source_map) = builder.build_with_provenance().unwrap();

    // config::Environment converts TEST__APP_NAME -> app_name
    let settings: TestSettings = config.build().unwrap().try_deserialize().unwrap();
    assert_eq!(settings.app_name, "env_app");

    let meta = source_map.source_of("app_name").unwrap();
    assert_eq!(meta.source_type, SourceType::Environment);
    assert!(meta.id.contains("TEST")); // "env_vars:TEST"

    std::env::remove_var("TEST__APP_NAME");
}
