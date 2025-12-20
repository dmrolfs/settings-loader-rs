use serde_json::json;
use settings_loader::introspection::SettingsIntrospection;
use settings_loader::metadata::{ConfigSchema, Constraint, SettingMetadata, SettingType, Visibility};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_json_schema_generation() {
    let schema = ConfigSchema {
        name: "test-app".to_string(),
        version: "1.0.0".to_string(),
        settings: vec![
            SettingMetadata {
                key: "port".to_string(),
                label: "Server Port".to_string(),
                description: "The port to listen on".to_string(),
                setting_type: SettingType::Integer { min: Some(1), max: Some(65535) },
                default: Some(json!(8080)),
                constraints: vec![Constraint::Required],
                visibility: Visibility::Public,
                group: None,
            },
            SettingMetadata {
                key: "database.host".to_string(),
                label: "Database Host".to_string(),
                description: "Hostname of the database server".to_string(),
                setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                default: Some(json!("localhost")),
                constraints: vec![],
                visibility: Visibility::Public,
                group: Some("database".to_string()),
            },
        ],
        groups: vec![],
    };

    let json_schema = schema.to_json_schema();

    // Verify structure
    assert_eq!(json_schema["type"], "object");
    assert_eq!(json_schema["title"], "test-app");
    assert!(json_schema["properties"]["port"].is_object());
    assert_eq!(json_schema["properties"]["port"]["type"], "integer");
    assert_eq!(json_schema["properties"]["port"]["minimum"], 1);

    // Verify nesting
    assert!(json_schema["properties"]["database"].is_object());
    assert_eq!(json_schema["properties"]["database"]["type"], "object");
    assert!(json_schema["properties"]["database"]["properties"]["host"].is_object());
}

#[test]
fn test_html_generation() {
    let schema = ConfigSchema {
        name: "test-app".to_string(),
        version: "1.0.0".to_string(),
        settings: vec![SettingMetadata {
            key: "port".to_string(),
            label: "Server Port".to_string(),
            description: "The port to listen on".to_string(),
            setting_type: SettingType::Integer { min: Some(1), max: Some(65535) },
            default: Some(json!(8080)),
            constraints: vec![Constraint::Required],
            visibility: Visibility::Public,
            group: None,
        }],
        groups: vec![],
    };

    let html = schema.to_html();
    assert!(html.contains("test-app Configuration Reference"));
    assert!(html.contains("Server Port"));
    assert!(html.contains("The port to listen on"));
    assert!(html.contains("8080"));
}

#[test]
fn test_toml_example_generation() {
    let schema = ConfigSchema {
        name: "test-app".to_string(),
        version: "1.0.0".to_string(),
        settings: vec![SettingMetadata {
            key: "database.host".to_string(),
            label: "Database Host".to_string(),
            description: "Hostname of the database server".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: Some(json!("localhost")),
            constraints: vec![],
            visibility: Visibility::Public,
            group: Some("database".to_string()),
        }],
        groups: vec![],
    };

    let toml = schema.to_example_toml();
    assert!(toml.contains("[database]"));
    assert!(toml.contains("# Hostname of the database server"));
    assert!(toml.contains("host = \"localhost\""));
}

#[test]
fn test_export_methods() {
    let temp_dir = tempdir().unwrap();
    let schema = ConfigSchema {
        name: "test-app".to_string(),
        version: "1.0.0".to_string(),
        settings: vec![SettingMetadata {
            key: "port".to_string(),
            label: "Server Port".to_string(),
            description: "Port".to_string(),
            setting_type: SettingType::Integer { min: None, max: None },
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
        }],
        groups: vec![],
    };

    let json_path = temp_dir.path().join("schema.json");
    let html_path = temp_dir.path().join("docs.html");
    let toml_path = temp_dir.path().join("example.toml");

    schema.export_json_schema(&json_path).unwrap();
    schema.export_docs(&html_path).unwrap();
    schema.export_example_config(&toml_path).unwrap();

    assert!(json_path.exists());
    assert!(html_path.exists());
    assert!(toml_path.exists());

    let json_content = fs::read_to_string(json_path).unwrap();
    assert!(json_content.contains("\"type\": \"integer\""));
}
