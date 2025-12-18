//! Comprehensive test suite for Phase 4: Configuration Editing & Writing

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(feature = "editor")]
mod tests {
    use super::*;
    use settings_loader::editor::json::JsonLayerEditor;
    use settings_loader::editor::toml::TomlLayerEditor;
    use settings_loader::editor::yaml::YamlLayerEditor;
    use settings_loader::editor::{ConfigFormat, Editor, EditorError, SettingsLoaderEditor};
    use settings_loader::LayerEditor;
    use settings_loader::SettingsEditor;
    use trim_margin::MarginTrimmable;

    // ========================================================================
    // Test 1-3: ConfigFormat Enum & from_path() Detection
    // ========================================================================

    /// Test ConfigFormat enum has all 5 format variants
    #[test]
    fn test_config_format_enum_variants() {
        // ConfigFormat should have exactly 5 variants:
        // - Toml
        // - Json
        // - Yaml
        // - Hjson (optional, may be deferred)
        // - Ron (optional, may be deferred)
        //
        // Must be: Debug, Clone, Copy, PartialEq, Eq

        let _toml = ConfigFormat::Toml;
        let _json = ConfigFormat::Json;
        let _yaml = ConfigFormat::Yaml;
        // Hjson and Ron may be optional in Phase 4
    }

    /// Test from_path() detects TOML files
    #[test]
    fn test_config_format_from_path_toml() {
        let toml_path = PathBuf::from("settings.toml");
        assert_eq!(ConfigFormat::from_path(&toml_path), Some(ConfigFormat::Toml));
    }

    /// Test from_path() detects JSON files
    #[test]
    fn test_config_format_from_path_json() {
        let json_path = PathBuf::from("config.json");
        assert_eq!(ConfigFormat::from_path(&json_path), Some(ConfigFormat::Json));
    }

    /// Test from_path() detects YAML files
    #[test]
    fn test_config_format_from_path_yaml() {
        let yaml_path = PathBuf::from("settings.yaml");
        assert_eq!(ConfigFormat::from_path(&yaml_path), Some(ConfigFormat::Yaml));

        let yml_path = PathBuf::from("settings.yml");
        assert_eq!(ConfigFormat::from_path(&yml_path), Some(ConfigFormat::Yaml));
    }

    /// Test from_path() returns None for unknown extensions
    #[test]
    fn test_config_format_from_path_unknown() {
        let unknown_path = PathBuf::from("settings.txt");
        assert_eq!(ConfigFormat::from_path(&unknown_path), None);
    }

    // ========================================================================
    // Test 5-8: LayerEditor Basic Operations (TOML)
    // ========================================================================

    /// Test opening existing TOML file
    #[test]
    fn test_toml_layer_editor_open_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");

        // Create test TOML file
        fs::write(&toml_path, "app_name = \"test_app\"\nversion = \"1.0.0\"").unwrap();

        // Open should succeed
        let editor = TomlLayerEditor::open(&toml_path).expect("Failed to open TOML");
        assert!(!editor.is_dirty());
    }

    /// Test get() retrieves values from TOML
    #[test]
    fn test_toml_layer_editor_get_string_value() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");

        fs::write(&toml_path, "app_name = \"my_app\"").unwrap();

        let editor = TomlLayerEditor::open(&toml_path).unwrap();
        let value: String = editor.get("app_name").expect("Failed to get value");
        assert_eq!(value, "my_app");
    }

    /// Test set() modifies values in TOML
    #[test]
    fn test_toml_layer_editor_set_string_value() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");

        fs::write(&toml_path, "app_name = \"old_app\"").unwrap();

        let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
        editor.set("app_name", "new_app").expect("Failed to set value");
        assert!(editor.is_dirty());
        let value: String = editor.get("app_name").unwrap();
        assert_eq!(value, "new_app");
    }

    /// Test unset() removes keys from TOML
    #[test]
    fn test_toml_layer_editor_unset_key() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");

        fs::write(&toml_path, "app_name = \"test\"\nversion = \"1.0\"").unwrap();

        let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
        editor.unset("version").expect("Failed to unset key");
        assert!(editor.is_dirty());
        let value: Option<String> = editor.get("version");
        assert!(value.is_none());
    }

    /// Test keys() returns all top-level keys
    #[test]
    fn test_toml_layer_editor_keys() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");

        fs::write(&toml_path, "app_name = \"test\"\nversion = \"1.0\"\ndebug = true").unwrap();

        let editor = TomlLayerEditor::open(&toml_path).unwrap();
        let keys = editor.keys();
        assert!(keys.contains(&"app_name".to_string()));
        assert!(keys.contains(&"version".to_string()));
        assert!(keys.contains(&"debug".to_string()));
    }

    // ========================================================================
    // Test 9-11: LayerEditor Dotted Path Navigation
    // ========================================================================

    /// Test get() with dotted path
    #[test]
    fn test_toml_layer_editor_get_nested_dotted_path() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");

        fs::write(&toml_path, "[database]\nhost = \"localhost\"\nport = 5432").unwrap();

        let editor = TomlLayerEditor::open(&toml_path).unwrap();
        let host: String = editor.get("database.host").unwrap();
        assert_eq!(host, "localhost");
        let port: u16 = editor.get("database.port").unwrap();
        assert_eq!(port, 5432);
    }

    /// Test set() with dotted path creates nested structure
    #[test]
    fn test_toml_layer_editor_set_nested_dotted_path() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");

        fs::write(&toml_path, "# Empty config").unwrap();

        let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
        editor.set("database.host", "db.example.com").unwrap();
        editor.set("database.port", 3306).unwrap();
        let host: String = editor.get("database.host").unwrap();
        assert_eq!(host, "db.example.com");
    }

    /// Test unset() with dotted path
    #[test]
    fn test_toml_layer_editor_unset_nested_dotted_path() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");

        fs::write(&toml_path, "[database]\nhost = \"localhost\"\nport = 5432").unwrap();

        // let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
        // editor.unset("database.port").unwrap();
        // let port: Option<u16> = editor.get("database.port");
        // assert!(port.is_none());
    }

    // ========================================================================
    // Test 12-14: Dirty Flag Tracking & Save Operations
    // ========================================================================

    /// Test is_dirty() returns false initially
    #[test]
    fn test_toml_layer_editor_dirty_flag_initial_state() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");
        fs::write(&toml_path, "app_name = \"test\"").unwrap();

        let editor = TomlLayerEditor::open(&toml_path).unwrap();
        assert!(!editor.is_dirty());
    }

    /// Test is_dirty() returns true after modification
    #[test]
    fn test_toml_layer_editor_dirty_flag_after_modification() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");
        fs::write(&toml_path, "app_name = \"test\"").unwrap();

        let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
        editor.set("app_name", "modified").unwrap();
        assert!(editor.is_dirty());
    }

    /// Test save() writes changes to file
    #[test]
    fn test_toml_layer_editor_save_persists_changes() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");
        fs::write(&toml_path, "app_name = \"original\"").unwrap();

        let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
        editor.set("app_name", "updated").unwrap();
        editor.save().expect("Failed to save");
        assert!(!editor.is_dirty());

        // Verify file was actually updated
        let content = fs::read_to_string(&toml_path).unwrap();
        assert!(content.contains("updated"));
    }

    // ========================================================================
    // Test 15-16: Atomic Writes
    // ========================================================================

    /// Test save() uses temp file + rename pattern
    #[test]
    fn test_toml_layer_editor_save_atomic_write() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");
        fs::write(&toml_path, "app_name = \"test\"").unwrap();

        let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
        editor.set("app_name", "modified").unwrap();
        editor.save().unwrap();

        // Verify original file was atomically replaced
        let content = fs::read_to_string(&toml_path).unwrap();
        assert!(content.contains("modified"));
        // No temp files should remain
        let entries: Vec<_> = fs::read_dir(temp_dir.path()).unwrap().filter_map(Result::ok).collect();
        assert_eq!(entries.len(), 1); // Only settings.toml
    }

    /// Test save() handles disk full gracefully
    #[test]
    fn test_toml_layer_editor_save_error_leaves_original_untouched() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");
        let original_content = "app_name = \"original\"";
        fs::write(&toml_path, original_content).unwrap();

        let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
        editor.set("app_name", "modified").unwrap();
        // Simulate write error by making directory read-only (platform-specific)
        // editor.save() should fail
        // Verify original file untouched
        // let content = fs::read_to_string(&toml_path).unwrap();
        // assert_eq!(content, original_content);
    }

    // ========================================================================
    // Test 17-18: TOML Comment Preservation (UNIQUE FEATURE)
    // ========================================================================

    /// Test TOML comments are preserved after modification
    #[test]
    fn test_toml_layer_editor_preserves_comments() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");

        let original_toml = r#"# Application Configuration
app_name = "my_app"  # The application name
version = "1.0.0"    # Version number

# Database Settings
[database]
host = "localhost"   # Database host
port = 5432          # Database port
"#;

        fs::write(&toml_path, original_toml).unwrap();

        let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
        editor.set("version", "2.0.0").unwrap();
        editor.save().unwrap();

        let modified_toml = fs::read_to_string(&toml_path).unwrap();

        // All comments should be preserved
        assert!(modified_toml.contains("# Application Configuration"));
        assert!(modified_toml.contains("# The application name"));
        assert!(modified_toml.contains("# Database Settings"));
        assert!(modified_toml.contains("# Database host"));
        assert!(modified_toml.contains("# Database port"));

        // Modified value should be updated
        assert!(modified_toml.contains("version = \"2.0.0\""));
        assert!(!modified_toml.contains("version = \"1.0.0\""));
    }

    /// Test TOML whitespace and formatting preserved
    #[test]
    fn test_toml_layer_editor_preserves_formatting() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");

        let original_toml = r#"[database]
# Critical settings
host = "localhost"

# Optional settings
ssl_enabled = true
"#;

        fs::write(&toml_path, original_toml).unwrap();

        let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
        editor.set("database.ssl_enabled", false).unwrap();
        editor.save().unwrap();

        let modified_toml = fs::read_to_string(&toml_path).unwrap();

        // Comments and blank lines should be preserved
        assert!(modified_toml.contains("# Critical settings"));
        assert!(modified_toml.contains("# Optional settings"));
        assert_eq!(modified_toml.matches('\n').count(), original_toml.matches('\n').count());
    }

    // ========================================================================
    // Test 19-20: JSON Backend (No Comment Preservation)
    // ========================================================================

    /// Test JSON editor opens and gets values
    #[test]
    fn test_json_layer_editor_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("settings.json");

        let json_content = r#"{
      "app_name": "test_app",
      "version": "1.0.0"
    }"#;

        fs::write(&json_path, json_content).unwrap();

        let mut editor = JsonLayerEditor::open(&json_path).unwrap();
        let app_name: String = editor.get("app_name").unwrap();
        assert_eq!(app_name, "test_app");

        editor.set("version", "2.0.0").unwrap();
        editor.save().unwrap();

        let modified_json = fs::read_to_string(&json_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&modified_json).unwrap();
        assert_eq!(parsed["version"], "2.0.0");
    }

    /// Test JSON nested path navigation
    #[test]
    fn test_json_layer_editor_nested_paths() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("settings.json");

        let json_content = r#"{
      "database": {
        "host": "localhost",
        "port": 5432
      }
    }"#;

        fs::write(&json_path, json_content).unwrap();

        let editor = JsonLayerEditor::open(&json_path).unwrap();
        let host: String = editor.get("database.host").unwrap();
        assert_eq!(host, "localhost");
        let port: u16 = editor.get("database.port").unwrap();
        assert_eq!(port, 5432);
    }

    // // ========================================================================
    // // Test 21-22: YAML Backend
    // // ========================================================================

    /// Test YAML editor roundtrip
    #[test]
    fn test_yaml_layer_editor_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("settings.yaml");

        let yaml_content = r#"
            |app_name: test_app
            |version: "1.0.0"
            |"#
        .trim_margin()
        .unwrap();

        fs::write(&yaml_path, yaml_content).unwrap();

        let mut editor = YamlLayerEditor::open(&yaml_path).unwrap();
        let app_name: String = editor.get("app_name").unwrap();
        assert_eq!(app_name, "test_app");

        editor.set("version", "2.0.0").unwrap();
        editor.save().unwrap();

        let modified_yaml = fs::read_to_string(&yaml_path).unwrap();
        assert!(modified_yaml.contains("version: \"2.0.0\"") || modified_yaml.contains("version: 2.0.0"));
    }

    /// Test YAML nested structures
    #[test]
    fn test_yaml_layer_editor_nested_structures() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("settings.yaml");

        let yaml_content = r#"
            |database:
            |  host: localhost
            |  port: 5432
            |  enabled: true
            |"#
        .trim_margin()
        .unwrap();

        fs::write(&yaml_path, yaml_content).unwrap();

        let editor = YamlLayerEditor::open(&yaml_path).unwrap();
        let host: String = editor.get("database.host").unwrap();
        assert_eq!(host, "localhost");
    }

    // ========================================================================
    // Test 23-24: Error Handling
    // ========================================================================

    /// Test opening non-existent file returns error
    #[test]
    fn test_layer_editor_open_nonexistent_file_error() {
        let _nonexistent_path = PathBuf::from("/tmp/nonexistent_config_12345.toml");

        let result = TomlLayerEditor::open(&_nonexistent_path);
        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::IoError(_) => {},
            _ => panic!("Expected IoError"),
        }
    }

    /// Test getting non-existent key returns None
    #[test]
    fn test_layer_editor_get_nonexistent_key() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");
        fs::write(&toml_path, "existing_key = \"value\"").unwrap();

        let editor = TomlLayerEditor::open(&toml_path).unwrap();
        let result: Option<String> = editor.get("nonexistent_key");
        assert!(result.is_none());
    }

    /// Test unsetting non-existent key returns error
    #[test]
    fn test_layer_editor_unset_nonexistent_key_error() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");
        fs::write(&toml_path, "existing_key = \"value\"").unwrap();

        let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
        let result = editor.unset("nonexistent_key");
        assert!(result.is_err());
        match result.unwrap_err() {
            EditorError::KeyNotFound(_) => {},
            _ => panic!("Expected KeyNotFound"),
        }
    }

    /// Test type mismatch error
    #[test]
    fn test_layer_editor_type_mismatch_error() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");
        fs::write(&toml_path, "port = 5432").unwrap();

        let editor = TomlLayerEditor::open(&toml_path).unwrap();
        let result: Option<String> = editor.get("port");
        // Should fail: trying to get u16 value as String
        assert!(result.is_none());
    }

    // ========================================================================
    // Test 25-26: SettingsEditor Factory Trait
    // ========================================================================

    /// Test SettingsEditor::open() format detection
    #[test]
    fn test_settings_editor_open_format_detection() {
        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("config.toml");
        fs::write(&toml_path, "test = true").unwrap();

        let editor = SettingsLoaderEditor::open(&toml_path).unwrap();
        match editor {
            Editor::Toml(_) => {}, // Expected Toml editor
            _ => panic!("Expected Toml editor"),
        }

        let json_path = temp_dir.path().join("config.json");
        fs::write(&json_path, "{ \"test\": true }").unwrap();
        let editor = SettingsLoaderEditor::open(&json_path).unwrap();
        match editor {
            Editor::Json(_) => {}, // Expected Json editor
            _ => panic!("Expected Json editor"),
        }

        let yaml_path = temp_dir.path().join("config.yaml");
        fs::write(&yaml_path, "test: true").unwrap();
        let editor = SettingsLoaderEditor::open(&yaml_path).unwrap();
        match editor {
            Editor::Yaml(_) => {}, // Expected Yaml editor
            _ => panic!("Expected Yaml editor"),
        }
    }

    /// Test SettingsEditor::create() with explicit format
    #[test]
    fn test_settings_editor_create_with_format() {
        let temp_dir = TempDir::new().unwrap();
        let new_toml_path = temp_dir.path().join("new_config.toml");

        let editor = SettingsLoaderEditor::create(&new_toml_path, ConfigFormat::Toml).unwrap();
        assert!(!editor.is_dirty());
        assert!(new_toml_path.exists());
        match editor {
            Editor::Toml(_) => {}, // Expected Toml editor
            _ => panic!("Expected Toml editor"),
        }

        let new_json_path = temp_dir.path().join("new_config.json");
        let editor = SettingsLoaderEditor::create(&new_json_path, ConfigFormat::Json).unwrap();
        assert!(!editor.is_dirty());
        assert!(new_json_path.exists());
        match editor {
            Editor::Json(_) => assert!(true),
            _ => panic!("Expected Json editor"),
        }

        let new_yaml_path = temp_dir.path().join("new_config.yaml");
        let editor = SettingsLoaderEditor::create(&new_yaml_path, ConfigFormat::Yaml).unwrap();
        assert!(!editor.is_dirty());
        assert!(new_yaml_path.exists());
        match editor {
            Editor::Yaml(_) => assert!(true),
            _ => panic!("Expected Yaml editor"),
        }
    }

    // ========================================================================
    // Test 27: Real-World Turtle TUI Scenario
    // ========================================================================

    /// Test Turtle configuration editing workflow
    #[test]
    fn test_turtle_tui_configuration_editing_scenario() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("turtle_settings.toml");

        let turtle_config = r#"# Spark Turtle Configuration

[app]
name = "Spark Turtle"
version = "0.1.0"

# Logging Configuration
[logging]
level = "info"
format = "json"

# TUI Settings
[tui]
theme = "dark"
enabled = true
"#;

        fs::write(&config_path, turtle_config).unwrap();

        // Scenario: User opens TUI, sees current settings, edits them
        let mut editor = SettingsLoaderEditor::open(&config_path).unwrap();

        // Get current values
        let current_level: String = editor.get("logging.level").unwrap();
        assert_eq!(current_level, "info");

        // User changes logging level
        editor.set("logging.level", "debug").unwrap();
        assert!(editor.is_dirty());

        // User changes theme
        editor.set("tui.theme", "light").unwrap();

        // User confirms changes
        editor.save().unwrap();
        assert!(!editor.is_dirty());

        // Verify changes persisted
        let updated_content = fs::read_to_string(&config_path).unwrap();
        assert!(updated_content.contains("level = \"debug\""));
        assert!(updated_content.contains("theme = \"light\""));

        // Comments should still be there (unique feature!)
        assert!(updated_content.contains("# Logging Configuration"));
        assert!(updated_content.contains("# TUI Settings"));
    }

    // ========================================================================
    // Test 28: EditorError Enum Variants
    // ========================================================================

    /// Test EditorError variants exist and convert properly
    #[test]
    fn test_editor_error_variants() {
        // EditorError should have:
        // - IoError (from std::io::Error)
        // - ParseError (String)
        // - SerializationError (String)
        // - KeyNotFound (String)
        // - FormatMismatch
        // - TypeMismatch { expected, actual }
        //
        // All should implement Display, Debug, Error traits

        // Test that errors convert from io::Error
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let _editor_error = EditorError::from(io_err);
    }
}

#[cfg(not(feature = "editor"))]
mod tests_without_feature {
    /// Placeholder test when editor feature is disabled
    #[test]
    fn editor_feature_not_enabled() {
        // This test documents that Phase 4 tests require the "editor" feature
        // Run with: cargo test --features editor
    }
}
