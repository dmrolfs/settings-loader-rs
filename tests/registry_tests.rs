#![cfg(feature = "metadata")]
use serde_json::json;
use settings_loader::metadata::{Constraint, SettingMetadata, SettingType, Visibility};
use settings_loader::{global_schema, init_global_registry, register_setting, SettingsRegistry};

#[test]
fn test_registry_registration() {
    let mut registry = SettingsRegistry::new("test-app", "1.0.0");

    let metadata = SettingMetadata {
        key: "server.port".to_string(),
        label: "Server Port".to_string(),
        description: "Port for the server to listen on".to_string(),
        setting_type: SettingType::Integer { min: Some(1024), max: Some(65535) },
        default: Some(json!(8080)),
        constraints: vec![Constraint::Required],
        visibility: Visibility::Public,
        group: Some("server".to_string()),
    };

    registry.register(metadata.clone());

    let retrieved = registry.get_metadata("server.port").expect("should find metadata");
    assert_eq!(retrieved.key, "server.port");
    assert_eq!(retrieved.label, "Server Port");
}

#[test]
fn test_global_registry() {
    init_global_registry("global-app", "2.0.0");

    let metadata = SettingMetadata {
        key: "app.name".to_string(),
        label: "App Name".to_string(),
        description: "Name of the application".to_string(),
        setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
        default: Some(json!("my-app")),
        constraints: vec![],
        visibility: Visibility::Public,
        group: None,
    };

    register_setting(metadata);

    let schema = global_schema();
    assert_eq!(schema.name, "global-app");
    assert!(schema.settings.iter().any(|s| s.key == "app.name"));
}

#[cfg(feature = "metadata")]
#[test]
fn test_registry_introspection() {
    use settings_loader::introspection::SettingsIntrospection;

    let mut registry = SettingsRegistry::new("intro-app", "1.0.0");

    registry.register(SettingMetadata {
        key: "api.key".to_string(),
        label: "API Key".to_string(),
        description: "Secrets".to_string(),
        setting_type: SettingType::Secret,
        default: None,
        constraints: vec![],
        visibility: Visibility::Secret,
        group: None,
    });

    assert_eq!(registry.settings_count(), 1);
    assert_eq!(registry.secret_settings().len(), 1);
    assert_eq!(registry.secret_settings()[0].key, "api.key");
}
