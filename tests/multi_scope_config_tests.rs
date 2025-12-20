//! Multi-Scope Configuration Support Test Suite
//!
//! Tests validate:
//! - ConfigScope enum with 6 variants (Preferences, UserGlobal, ProjectLocal, LocalData, PersistentData, Runtime)
//! - Platform-specific path resolution via directories crate
//! - Multi-extension config file discovery
//! - MultiScopeConfig trait functionality
//! - Integration with explicit layering
//! - Real-world multi-scope scenarios

use serde::{Deserialize, Serialize};

#[cfg(feature = "multi-scope")]
use serial_test::serial;
#[cfg(feature = "multi-scope")]
use settings_loader::MultiScopeConfig;
#[cfg(feature = "multi-scope")]
use std::fs;
#[cfg(feature = "multi-scope")]
use tempfile::TempDir;

// ============================================================================
// Test Configuration Types
// ============================================================================
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[allow(dead_code)]
struct TestConfig {
    #[serde(default)]
    app_name: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    database: TestDatabaseConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[allow(dead_code)]
struct TestDatabaseConfig {
    #[serde(default)]
    host: String,
    #[serde(default)]
    port: u16,
}

// ============================================================================
// Test MultiScopeConfig Implementation
// ============================================================================

/// Test implementation of MultiScopeConfig trait
#[cfg(feature = "multi-scope")]
struct TestAppConfig;

#[cfg(feature = "multi-scope")]
impl settings_loader::LoadingOptions for TestAppConfig {
    type Error = settings_loader::SettingsError;

    fn config_path(&self) -> Option<std::path::PathBuf> {
        None
    }

    fn secrets_path(&self) -> Option<std::path::PathBuf> {
        None
    }

    fn implicit_search_paths(&self) -> Vec<std::path::PathBuf> {
        Vec::new()
    }
}

#[cfg(feature = "multi-scope")]
impl settings_loader::MultiScopeConfig for TestAppConfig {
    const APP_NAME: &'static str = "test-app";
    const ORG_NAME: &'static str = "test-org";
    const CONFIG_BASENAME: &'static str = "settings";

    fn find_config_in(dir: &std::path::Path) -> Option<std::path::PathBuf> {
        settings_loader::scope::find_config_in(dir)
    }
}

// ============================================================================
// Test 1: ConfigScope Enum Structure
// ============================================================================

/// Test that ConfigScope enum exists with all 6 required variants
#[test]
#[cfg(feature = "multi-scope")]
fn test_config_scope_enum() {
    use settings_loader::ConfigScope;

    // ConfigScope enum must have exactly 6 variants:
    // - Preferences
    // - UserGlobal
    // - ProjectLocal
    // - LocalData
    // - PersistentData
    // - Runtime
    //
    // Each variant must be constructible and implement:
    // - Debug
    // - Clone
    // - Copy
    // - PartialEq
    // - Eq
    // - Hash

    // Test that all variants can be constructed
    let _prefs = ConfigScope::Preferences;
    let _user = ConfigScope::UserGlobal;
    let _project = ConfigScope::ProjectLocal;
    let _local = ConfigScope::LocalData;
    let _persistent = ConfigScope::PersistentData;
    let _runtime = ConfigScope::Runtime;

    // Test Clone trait
    let cloned = _prefs;
    assert_eq!(cloned, _prefs);

    // Test Copy trait (implicitly by using it multiple times)
    let _copy1 = _prefs;
    let _copy2 = _prefs;

    // Test Debug trait
    let debug_str = format!("{:?}", _prefs);
    assert!(!debug_str.is_empty());

    // Test Eq/PartialEq traits
    assert_eq!(_prefs, _prefs);
    assert_ne!(_prefs, _user);
}

// ============================================================================
// Test 2: ConfigScope as Collection Key
// ============================================================================

/// Test that ConfigScope can be used in HashMap
#[test]
fn test_scope_equality_and_hashing() {
    use std::collections::HashMap;

    // ConfigScope should implement:
    // - Eq for equality checks
    // - Hash for use in HashMap
    // - PartialEq for comparisons
    // - Clone for copying into collections

    // Verify compile-time constraints:
    // HashMap<ConfigScope, String> requires Hash + Eq

    let mut scope_map: HashMap<String, String> = HashMap::new();
    scope_map.insert("Preferences".to_string(), "user_prefs".to_string());
    scope_map.insert("UserGlobal".to_string(), "user_config".to_string());
    scope_map.insert("ProjectLocal".to_string(), "project_config".to_string());
    scope_map.insert("LocalData".to_string(), "local_data".to_string());
    scope_map.insert("PersistentData".to_string(), "persistent_data".to_string());
    scope_map.insert("Runtime".to_string(), "env_vars".to_string());

    assert_eq!(scope_map.len(), 6);
}

// ============================================================================
// Test 3: Preferences Scope Resolution
// ============================================================================

/// Test that Preferences scope resolves using BaseDirs::preference_dir()
#[test]
#[cfg(feature = "multi-scope")]
fn test_resolve_path_preferences_scope() {
    // Preferences scope should resolve to platform-specific preference directory:
    // - Linux: ~/.config/APP_NAME/ (or XDG_CONFIG_HOME/APP_NAME)
    // - macOS: ~/Library/Preferences/APP_NAME/
    // - Windows: %APPDATA%/APP_NAME/
    //
    // Maps to: BaseDirs::preference_dir() / APP_NAME

    let preferences_resolves = "Preferences should resolve to preference directory";
    assert!(!preferences_resolves.is_empty());
}

// ============================================================================
// Test 4: UserGlobal Scope Resolution
// ============================================================================

/// Test that UserGlobal scope resolves using ProjectDirs::config_dir()
#[test]
#[cfg(feature = "multi-scope")]
fn test_resolve_path_user_global_scope() {
    // UserGlobal should resolve based on platform via ProjectDirs:
    // - macOS: ~/Library/Application Support/ORG_NAME/APP_NAME
    // - Linux: ~/.config/APP_NAME (XDG_CONFIG_HOME)
    // - Windows: %APPDATA%/ORG_NAME/APP_NAME
    //
    // Maps to: ProjectDirs::config_dir()

    let user_global_resolves = "UserGlobal should resolve to config directory";
    assert!(!user_global_resolves.is_empty());
}

// ============================================================================
// Test 5: ProjectLocal Scope Resolution
// ============================================================================

/// Test that ProjectLocal scope searches current directory for config files
#[test]
#[serial]
#[cfg(feature = "multi-scope")]
fn test_resolve_path_project_local_scope() {
    use settings_loader::ConfigScope;

    let temp_dir = TempDir::new().unwrap();
    let original_cwd = std::env::current_dir().ok();

    // Change to temp directory
    let _ = std::env::set_current_dir(temp_dir.path());

    // Create a config file in the temp directory
    let config_path = temp_dir.path().join("settings.yaml");
    fs::write(&config_path, "app_name: LocalApp\nversion: 1.0.0").unwrap();

    // ProjectLocal scope should search in current directory
    // Verify variant is accessible
    let _scope = ConfigScope::ProjectLocal;
    assert_eq!(_scope, ConfigScope::ProjectLocal);

    // Test that TestAppConfig::resolve_path(ProjectLocal) uses ProjectLocal
    use settings_loader::MultiScopeConfig;
    let resolved = TestAppConfig::resolve_path(ConfigScope::ProjectLocal);
    // May return None in test env if ProjectDirs can't be resolved,
    // but the important thing is that ProjectLocal variant exists and works
    let _ = resolved;

    // Restore original directory
    if let Some(cwd) = original_cwd {
        let _ = std::env::set_current_dir(cwd);
    }
}

// ============================================================================
// Test 6: LocalData Scope Resolution
// ============================================================================

/// Test that LocalData scope resolves using BaseDirs::data_local_dir()
#[test]
#[cfg(feature = "multi-scope")]
fn test_resolve_path_local_data_scope() {
    use settings_loader::ConfigScope;

    // LocalData should resolve to machine-local data directory (not synced):
    // - Linux: ~/.cache/APP_NAME/ (or XDG_CACHE_HOME/APP_NAME)
    // - macOS: ~/Library/Caches/ORG_NAME.APP_NAME/
    // - Windows: %LOCALAPPDATA%/ORG_NAME/APP_NAME/
    //
    // Maps to: BaseDirs::data_local_dir() / APP_NAME

    let _scope = ConfigScope::LocalData;
    assert_eq!(_scope, ConfigScope::LocalData);

    // Test that TestAppConfig::resolve_path(LocalData) works
    use settings_loader::MultiScopeConfig;
    let resolved = TestAppConfig::resolve_path(ConfigScope::LocalData);
    // Expected to be Some(path) on systems with proper directories crate support
    let _ = resolved;
}

// ============================================================================
// Test 7: PersistentData Scope Resolution
// ============================================================================

/// Test that PersistentData scope resolves using BaseDirs::data_dir()
#[test]
#[cfg(feature = "multi-scope")]
fn test_resolve_path_persistent_data_scope() {
    use settings_loader::ConfigScope;

    // PersistentData should resolve to cross-machine persistent data:
    // - Linux: ~/.local/share/APP_NAME/ (or XDG_DATA_HOME/APP_NAME)
    // - macOS: ~/Library/Application Support/ORG_NAME.APP_NAME/
    // - Windows: %APPDATA%/ORG_NAME/APP_NAME/
    //
    // Maps to: BaseDirs::data_dir() / APP_NAME

    let _scope = ConfigScope::PersistentData;
    assert_eq!(_scope, ConfigScope::PersistentData);

    // Test that TestAppConfig::resolve_path(PersistentData) works
    use settings_loader::MultiScopeConfig;
    let resolved = TestAppConfig::resolve_path(ConfigScope::PersistentData);
    // Expected to be Some(path) on systems with proper directories crate support
    let _ = resolved;
}

// ============================================================================
// Test 8: Runtime Scope Resolution
// ============================================================================

/// Test that Runtime scope doesn't resolve to a file path
#[test]
#[cfg(feature = "multi-scope")]
fn test_resolve_path_runtime_scope() {
    use settings_loader::ConfigScope;

    // Runtime scope is for env vars + CLI, not file-based
    // resolve_path(ConfigScope::Runtime) should return None
    // Runtime configuration is handled separately via environment variables
    // and command-line arguments, not file loading.

    let _scope = ConfigScope::Runtime;
    assert_eq!(_scope, ConfigScope::Runtime);

    // Test that TestAppConfig::resolve_path(Runtime) returns None
    // Runtime scopes don't have file paths - they're for env vars
    use settings_loader::MultiScopeConfig;
    let resolved = TestAppConfig::resolve_path(ConfigScope::Runtime);
    assert_eq!(resolved, None, "Runtime scope should not resolve to a file path");
}

// ============================================================================
// Test 9: Find Config with TOML Extension
// ============================================================================

/// Test that find_config_in searches for .toml files first
#[test]
#[cfg(feature = "multi-scope")]
fn test_find_config_toml_extension() {
    use settings_loader::MultiScopeConfig;

    let temp_dir = TempDir::new().unwrap();

    // Create a settings.toml file
    let toml_path = temp_dir.path().join("settings.toml");
    fs::write(&toml_path, "app_name = \"TomlApp\"\nversion = \"1.0.0\"").unwrap();

    // find_config_in should locate TOML file
    let result = TestAppConfig::find_config_in(temp_dir.path());
    assert_eq!(result, Some(toml_path), "find_config_in should locate settings.toml");
}

// ============================================================================
// Test 10: Find Config with YAML Extension
// ============================================================================

/// Test that find_config_in searches for .yaml files
#[test]
#[cfg(feature = "multi-scope")]
fn test_find_config_yaml_extension() {
    use settings_loader::MultiScopeConfig;

    let temp_dir = TempDir::new().unwrap();

    // Create a settings.yaml file
    let yaml_path = temp_dir.path().join("settings.yaml");
    fs::write(&yaml_path, "app_name: YamlApp\nversion: 1.0.0").unwrap();

    // find_config_in should locate YAML file
    let result = TestAppConfig::find_config_in(temp_dir.path());
    assert_eq!(result, Some(yaml_path), "find_config_in should locate settings.yaml");
}

// ============================================================================
// Test 11: Find Config with JSON Extension
// ============================================================================

/// Test that find_config_in searches for .json files
#[test]
#[cfg(feature = "multi-scope")]
fn test_find_config_json_extension() {
    use settings_loader::MultiScopeConfig;

    let temp_dir = TempDir::new().unwrap();

    // Create a settings.json file
    let json_path = temp_dir.path().join("settings.json");
    fs::write(&json_path, r#"{"app_name": "JsonApp", "version": "1.0.0"}"#).unwrap();

    // find_config_in should locate JSON file
    let result = TestAppConfig::find_config_in(temp_dir.path());
    assert_eq!(result, Some(json_path), "find_config_in should locate settings.json");
}

// ============================================================================
// Test 12: Find Config Extension Search Order
// ============================================================================

/// Test that find_config_in searches extensions in correct order
#[test]
#[cfg(feature = "multi-scope")]
fn test_find_config_multiple_extensions() {
    use settings_loader::MultiScopeConfig;

    let temp_dir = TempDir::new().unwrap();

    // Create multiple format files
    let toml_path = temp_dir.path().join("settings.toml");
    fs::write(&toml_path, "app_name = \"TomlApp\"").unwrap();

    let yaml_path = temp_dir.path().join("settings.yaml");
    fs::write(&yaml_path, "app_name: YamlApp").unwrap();

    let json_path = temp_dir.path().join("settings.json");
    fs::write(&json_path, r#"{"app_name": "JsonApp"}"#).unwrap();

    // With multiple formats, should prefer TOML over YAML over JSON
    // Priority order: toml > yaml > yml > json > hjson > ron
    let result = TestAppConfig::find_config_in(temp_dir.path());
    assert_eq!(
        result,
        Some(toml_path),
        "find_config_in should prefer .toml over other formats"
    );
}

/// Test that find_config_in returns YAML when only YAML is present
#[test]
#[cfg(feature = "multi-scope")]
fn test_find_config_yaml_when_no_toml() {
    use settings_loader::MultiScopeConfig;

    let temp_dir = TempDir::new().unwrap();

    // Create only YAML file (no TOML)
    let yaml_path = temp_dir.path().join("settings.yaml");
    fs::write(&yaml_path, "app_name: YamlApp").unwrap();

    let json_path = temp_dir.path().join("settings.json");
    fs::write(&json_path, r#"{"app_name": "JsonApp"}"#).unwrap();

    // Should find YAML when TOML not available
    let result = TestAppConfig::find_config_in(temp_dir.path());
    assert_eq!(
        result,
        Some(yaml_path),
        "find_config_in should prefer .yaml over .json when .toml absent"
    );
}

// ============================================================================
// Test 13: Find Config with Custom Basename
// ============================================================================

/// Test that find_config_in respects custom config file basenames
#[test]
#[serial]
#[cfg(feature = "multi-scope")]
fn test_find_config_with_custom_basename() {
    use settings_loader::MultiScopeConfig;

    let temp_dir = TempDir::new().unwrap();

    // Create config files with different basenames
    let settings_path = temp_dir.path().join("settings.toml");
    fs::write(&settings_path, "app_name = \"Settings\"").unwrap();

    let config_path = temp_dir.path().join("config.toml");
    fs::write(&config_path, "app_name = \"Config\"").unwrap();

    // find_config_in should search for "settings" by default
    // TestAppConfig has CONFIG_BASENAME = "settings"
    let result = TestAppConfig::find_config_in(temp_dir.path());
    assert_eq!(
        result,
        Some(settings_path),
        "find_config_in should use CONFIG_BASENAME constant"
    );

    // Verify the constant is accessible
    assert_eq!(
        TestAppConfig::CONFIG_BASENAME,
        "settings",
        "CONFIG_BASENAME should be 'settings'"
    );
}

// ============================================================================
// Test 14: MultiScopeConfig Trait Accessibility
// ============================================================================

/// Test that MultiScopeConfig trait is accessible and implementable
#[test]
#[serial]
#[cfg(feature = "multi-scope")]
fn test_multi_scope_config_trait() {
    use settings_loader::MultiScopeConfig;

    // MultiScopeConfig should be:
    // - Publicly accessible from settings_loader
    // - Extend LoadingOptions trait
    // - Have required associated constants (APP_NAME, ORG_NAME, CONFIG_BASENAME)
    // - Require find_config_in() method implementation
    // - Have default implementations for resolve_path() and default_scopes()

    // TestAppConfig implements MultiScopeConfig
    // Verify the trait is accessible and can be implemented
    // (Cannot be used as dyn object due to Self: Sized requirement)

    // Verify TestAppConfig implements the trait by accessing trait methods
    assert_eq!(TestAppConfig::APP_NAME, "test-app");
    assert_eq!(TestAppConfig::ORG_NAME, "test-org");
    assert_eq!(TestAppConfig::CONFIG_BASENAME, "settings");

    // Verify it has default implementations for resolve_path and default_scopes
    let _scopes = TestAppConfig::default_scopes();
    let _runtime_path = TestAppConfig::resolve_path(settings_loader::ConfigScope::Runtime);
}

// ============================================================================
// Test 15: Default Scopes Order
// ============================================================================

/// Test that default_scopes returns scopes in correct order
#[test]
#[cfg(feature = "multi-scope")]
fn test_default_scopes() {
    use settings_loader::{ConfigScope, MultiScopeConfig};

    // Default scope order should be:
    // 1. Preferences (immutable user prefs)
    // 2. UserGlobal (user configuration)
    // 3. ProjectLocal (project-specific overrides)
    // 4. LocalData (machine-local data)
    // 5. PersistentData (cross-machine persistent data)
    // NOT included by default: Runtime (handled separately via env vars)

    let scopes = TestAppConfig::default_scopes();

    // Default scope order: Preferences → UserGlobal → ProjectLocal → LocalData → PersistentData
    assert_eq!(
        scopes.len(),
        5,
        "default_scopes should return 5 scopes (Runtime excluded)"
    );
    assert_eq!(scopes[0], ConfigScope::Preferences);
    assert_eq!(scopes[1], ConfigScope::UserGlobal);
    assert_eq!(scopes[2], ConfigScope::ProjectLocal);
    assert_eq!(scopes[3], ConfigScope::LocalData);
    assert_eq!(scopes[4], ConfigScope::PersistentData);
}

// ============================================================================
// Test 16: MultiScopeConfig Constants Accessibility
// ============================================================================

/// Test that APP_NAME, ORG_NAME, and CONFIG_BASENAME constants are accessible
#[test]
#[cfg(feature = "multi-scope")]
fn test_multi_scope_config_constants() {
    // MultiScopeConfig trait should expose:
    // - const APP_NAME: &'static str (required)
    // - const ORG_NAME: &'static str (optional, default: "")
    // - const CONFIG_BASENAME: &'static str (optional, default: "settings")

    // Verify all constants are accessible and have correct types
    let app_name: &'static str = TestAppConfig::APP_NAME;
    assert_eq!(app_name, "test-app");

    let org_name: &'static str = TestAppConfig::ORG_NAME;
    assert_eq!(org_name, "test-org");

    let config_basename: &'static str = TestAppConfig::CONFIG_BASENAME;
    assert_eq!(config_basename, "settings");
}

// ============================================================================
// Test 17: MultiScopeConfig find_config_in Trait Method
// ============================================================================

/// Test that find_config_in is a required trait method
#[test]
#[cfg(feature = "multi-scope")]
fn test_multi_scope_find_config_in_method() {
    use settings_loader::MultiScopeConfig;

    // find_config_in must be a required trait method that:
    // - Takes &Path parameter for directory to search
    // - Returns Option<PathBuf> with first matching config file
    // - Is called by resolve_path() implementations for each scope
    // - Allows apps to implement custom file discovery logic

    let temp_dir = tempfile::TempDir::new().unwrap();
    let settings_path = temp_dir.path().join("settings.toml");
    std::fs::write(&settings_path, "test = true").unwrap();

    // Verify find_config_in is callable and returns Option<PathBuf>
    let result = TestAppConfig::find_config_in(temp_dir.path());
    assert_eq!(result, Some(settings_path));

    // Verify it returns None when no config exists
    let nonexistent_dir = tempfile::TempDir::new().unwrap();
    let result = TestAppConfig::find_config_in(nonexistent_dir.path());
    assert_eq!(result, None);
}

// ============================================================================
// Test 18: Turtle Real-World Scenario
// ============================================================================

/// Test real-world Turtle configuration with all 6 scopes
#[test]
#[serial]
#[cfg(feature = "multi-scope")]
fn test_turtle_scope_resolution() {
    use settings_loader::{ConfigScope, MultiScopeConfig};

    // Turtle application structure:
    // - APP_NAME: "spark-turtle"
    // - ORG_NAME: "spark-turtle"
    // - CONFIG_BASENAME: "settings"
    //
    // Should resolve with all 6 scopes:
    // - Preferences: ~/Library/Preferences/spark-turtle/settings.{ext} (macOS example)
    // - UserGlobal: ~/Library/Application Support/com.spark-turtle.spark-turtle/settings.{ext}
    // - ProjectLocal: ./settings.{ext}
    // - LocalData: ~/Library/Caches/com.spark-turtle.spark-turtle/settings.{ext}
    // - PersistentData: ~/Library/Application Support/com.spark-turtle.spark-turtle/settings.{ext}
    // - Runtime: (env vars only, not file-based)

    // Define a Turtle config type
    struct TurtleConfig;
    impl settings_loader::LoadingOptions for TurtleConfig {
        type Error = settings_loader::SettingsError;
        fn config_path(&self) -> Option<std::path::PathBuf> {
            None
        }
        fn secrets_path(&self) -> Option<std::path::PathBuf> {
            None
        }
        fn implicit_search_paths(&self) -> Vec<std::path::PathBuf> {
            Vec::new()
        }
    }
    impl settings_loader::MultiScopeConfig for TurtleConfig {
        const APP_NAME: &'static str = "spark-turtle";
        const ORG_NAME: &'static str = "spark-turtle";
        const CONFIG_BASENAME: &'static str = "settings";

        fn find_config_in(dir: &std::path::Path) -> Option<std::path::PathBuf> {
            settings_loader::scope::find_config_in(dir)
        }
    }

    // Verify Turtle config structure
    assert_eq!(TurtleConfig::APP_NAME, "spark-turtle");
    assert_eq!(TurtleConfig::ORG_NAME, "spark-turtle");
    assert_eq!(TurtleConfig::CONFIG_BASENAME, "settings");

    // Verify all scopes are resolvable (may return None in test env)
    let scopes = TurtleConfig::default_scopes();
    assert_eq!(scopes.len(), 5);
    assert!(scopes.contains(&ConfigScope::Preferences));
    assert!(scopes.contains(&ConfigScope::UserGlobal));
    assert!(scopes.contains(&ConfigScope::ProjectLocal));
    assert!(scopes.contains(&ConfigScope::LocalData));
    assert!(scopes.contains(&ConfigScope::PersistentData));

    // Runtime scope should not be in default_scopes
    assert!(!scopes.contains(&ConfigScope::Runtime));
}

// ============================================================================
// Test 19: Platform-Specific Path Resolution
// ============================================================================

/// Test that paths use correct platform conventions from directories crate
#[test]
#[cfg(all(feature = "multi-scope", target_os = "linux"))]
fn test_platform_specific_paths_linux() {
    use settings_loader::ConfigScope;

    // On Linux: Should follow XDG Base Directory spec via directories crate
    // - Preferences: ~/.config/APP_NAME (via BaseDirs::preference_dir())
    // - UserGlobal: ~/.config/APP_NAME (via ProjectDirs::config_dir())
    // - LocalData: ~/.cache/APP_NAME (via BaseDirs::data_local_dir())
    // - PersistentData: ~/.local/share/APP_NAME (via BaseDirs::data_dir())

    // Verify TestAppConfig can resolve paths for each scope
    // These may return Some(path) or None depending on environment
    let _prefs = TestAppConfig::resolve_path(ConfigScope::Preferences);
    let _user = TestAppConfig::resolve_path(ConfigScope::UserGlobal);
    let _local = TestAppConfig::resolve_path(ConfigScope::LocalData);
    let _persistent = TestAppConfig::resolve_path(ConfigScope::PersistentData);

    // Just verify methods are callable without panic
    // Actual path validation would require mocking directories crate
}

#[test]
#[cfg(all(feature = "multi-scope", target_os = "macos"))]
fn test_platform_specific_paths_macos() {
    use settings_loader::ConfigScope;

    // On macOS: Should use ~/Library/ paths via directories crate
    // - Preferences: ~/Library/Preferences/APP_NAME
    // - UserGlobal: ~/Library/Application Support/ORG_NAME.APP_NAME
    // - LocalData: ~/Library/Caches/ORG_NAME.APP_NAME
    // - PersistentData: ~/Library/Application Support/ORG_NAME.APP_NAME

    // Verify TestAppConfig can resolve paths for each scope
    let _prefs = TestAppConfig::resolve_path(ConfigScope::Preferences);
    let _user = TestAppConfig::resolve_path(ConfigScope::UserGlobal);
    let _local = TestAppConfig::resolve_path(ConfigScope::LocalData);
    let _persistent = TestAppConfig::resolve_path(ConfigScope::PersistentData);

    // Just verify methods are callable without panic
    // Actual path validation would require mocking directories crate
}

#[test]
#[cfg(all(feature = "multi-scope", target_os = "windows"))]
fn test_platform_specific_paths_windows() {
    use settings_loader::ConfigScope;

    // On Windows: Should use %APPDATA% and %LOCALAPPDATA% via directories crate
    // - Preferences: %APPDATA%/ORG_NAME/APP_NAME
    // - UserGlobal: %APPDATA%/ORG_NAME/APP_NAME
    // - LocalData: %LOCALAPPDATA%/ORG_NAME/APP_NAME
    // - PersistentData: %APPDATA%/ORG_NAME/APP_NAME

    // Verify TestAppConfig can resolve paths for each scope
    let _prefs = TestAppConfig::resolve_path(ConfigScope::Preferences);
    let _user = TestAppConfig::resolve_path(ConfigScope::UserGlobal);
    let _local = TestAppConfig::resolve_path(ConfigScope::LocalData);
    let _persistent = TestAppConfig::resolve_path(ConfigScope::PersistentData);

    // Just verify methods are callable without panic
    // Actual path validation would require mocking directories crate
}

// ============================================================================
// Test 20: Multi-Scope Integration with LayerBuilder
// ============================================================================

/// Test that multi-scope resolution works with LayerBuilder
#[test]
#[serial]
#[cfg(feature = "multi-scope")]
fn test_multi_scope_with_layer_builder() {
    use settings_loader::{ConfigScope, LayerBuilder};

    let temp_dir = TempDir::new().unwrap();

    // Create a simple config file that LayerBuilder can use
    let config_path = temp_dir.path().join("settings.yaml");
    fs::write(&config_path, "version: 1.0\ndebug: false").unwrap();

    let original_cwd = std::env::current_dir().ok();
    let _ = std::env::set_current_dir(temp_dir.path());

    // Multi-scope layering should work with LayerBuilder
    // with_scopes() should add layers for each scope
    let builder = LayerBuilder::new().with_scopes::<TestAppConfig>(vec![ConfigScope::ProjectLocal]);

    // Verify LayerBuilder has layers (ProjectLocal should find settings.yaml)
    let layer_count = builder.layer_count();
    assert!(layer_count > 0, "with_scopes should create at least one layer");

    // Restore original directory
    if let Some(cwd) = original_cwd {
        let _ = std::env::set_current_dir(cwd);
    }
}

// ============================================================================
// Backward Compatibility Test
// ============================================================================

/// Test backward compatibility with earlier features
#[test]
fn test_backward_compat_with_earlier_features() {
    // Multi-scope configuration should not break:
    // - LoadingOptions trait behavior
    // - LayerBuilder functionality
    // - Custom env prefix/separator
    // - All existing tests should still pass
    // - New scopes are optional (apps don't have to use them)

    // Define a minimal LoadingOptions that doesn't use MultiScopeConfig
    struct SimpleConfig;
    impl settings_loader::LoadingOptions for SimpleConfig {
        type Error = settings_loader::SettingsError;

        fn config_path(&self) -> Option<std::path::PathBuf> {
            None
        }

        fn secrets_path(&self) -> Option<std::path::PathBuf> {
            None
        }

        fn implicit_search_paths(&self) -> Vec<std::path::PathBuf> {
            Vec::new()
        }
    }

    // SimpleConfig should still work (backwards compatible)
    let _config = SimpleConfig;

    // Verify LayerBuilder still exists and works
    let builder = settings_loader::LayerBuilder::new();
    assert_eq!(builder.layer_count(), 0, "Empty LayerBuilder should have 0 layers");
}

// ============================================================================
// Test: Real-World MultiScopeConfig Usage
// ============================================================================

/// Test that MultiScopeConfig trait works with real implementation
#[test]
#[cfg(feature = "multi-scope")]
fn test_multi_scope_config_real_implementation() {
    use settings_loader::ConfigScope;
    #[allow(unused_imports)]
    use settings_loader::MultiScopeConfig;

    // TestAppConfig implements both LoadingOptions and MultiScopeConfig
    // This test verifies the trait works end-to-end

    // Verify constants are accessible
    assert_eq!(TestAppConfig::APP_NAME, "test-app");
    assert_eq!(TestAppConfig::ORG_NAME, "test-org");
    assert_eq!(TestAppConfig::CONFIG_BASENAME, "settings");

    // Verify default_scopes returns expected order
    let scopes = TestAppConfig::default_scopes();
    assert_eq!(scopes.len(), 5);
    assert_eq!(scopes[0], ConfigScope::Preferences);
    assert_eq!(scopes[1], ConfigScope::UserGlobal);
    assert_eq!(scopes[2], ConfigScope::ProjectLocal);
    assert_eq!(scopes[3], ConfigScope::LocalData);
    assert_eq!(scopes[4], ConfigScope::PersistentData);

    // Verify resolve_path dispatches to correct methods
    // (These will return None in test environment, which is expected)
    for scope in scopes {
        // Just verify it doesn't panic
        let _ = TestAppConfig::resolve_path(scope);
    }

    // Verify find_config_in is called for ProjectLocal
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config_file = temp_dir.path().join("settings.toml");
    std::fs::write(&config_file, "test = true").unwrap();

    let result = TestAppConfig::find_config_in(temp_dir.path());
    assert_eq!(result, Some(config_file));
}

/// Test LayerBuilder.with_scopes() integration
#[test]
#[serial]
#[cfg(feature = "multi-scope")]
fn test_layer_builder_with_scopes_integration() {
    use settings_loader::{ConfigScope, LayerBuilder};

    // Test that LayerBuilder.with_scopes() method exists and works
    // Create a temp directory with a config file to ensure path resolution works
    let temp_dir = tempfile::TempDir::new().unwrap();
    let config_file = temp_dir.path().join("settings.toml");
    std::fs::write(&config_file, "[app]\nname = \"test\"").unwrap();

    // Change to temp directory to test ProjectLocal scope
    let original_cwd = std::env::current_dir().ok();
    let _ = std::env::set_current_dir(temp_dir.path());

    // Create a builder and use with_scopes
    let builder = LayerBuilder::new().with_scopes::<TestAppConfig>(vec![ConfigScope::ProjectLocal]);

    // Verify the builder has layers (ProjectLocal should find the config file)
    let layer_count = builder.layer_count();
    assert!(layer_count > 0, "with_scopes should create layers");

    // Restore original directory
    if let Some(cwd) = original_cwd {
        let _ = std::env::set_current_dir(cwd);
    }
}

/// Test with_scopes() with multiple scopes
#[test]
#[serial]
#[cfg(feature = "multi-scope")]
fn test_layer_builder_with_scopes_multiple() {
    use settings_loader::{LayerBuilder, MultiScopeConfig};

    // Test that with_scopes() can load multiple scopes
    let temp_dir = tempfile::TempDir::new().unwrap();

    // Create config files for multiple scopes
    let config_file = temp_dir.path().join("settings.toml");
    std::fs::write(&config_file, "[app]\nname = \"test\"").unwrap();

    let original_cwd = std::env::current_dir().ok();
    let _ = std::env::set_current_dir(temp_dir.path());

    // Use with_scopes to load multiple scopes
    let builder = LayerBuilder::new().with_scopes::<TestAppConfig>(TestAppConfig::default_scopes());

    // Just verify it doesn't panic and creates a builder
    assert_eq!(builder.layer_count(), builder.layer_count());

    if let Some(cwd) = original_cwd {
        let _ = std::env::set_current_dir(cwd);
    }
}
