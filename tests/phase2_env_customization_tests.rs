//! Comprehensive test suite for Phase 2: Environment Variable Customization
//!
//! Tests validate:
//! - Default environment variable conventions (prefix "APP", separator "__")
//! - Custom environment variable naming conventions
//! - Integration with LayerBuilder
//! - Backward compatibility with Phase 1 layering
//!
//! TDD RED PHASE - Tests currently fail until implementation is complete.

use serde::{Deserialize, Serialize};
use settings_loader::LoadingOptions;
use std::fs;

// ============================================================================
// Test Configuration Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
struct TestConfig {
    #[serde(default)]
    app_name: String,
    #[serde(default)]
    port: u16,
    #[serde(default)]
    debug: bool,
    #[serde(default)]
    database: DatabaseConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
struct DatabaseConfig {
    #[serde(default)]
    host: String,
    #[serde(default)]
    port: u16,
    #[serde(default)]
    user: String,
}

// ============================================================================
// LoadingOptions Implementations for Testing
// ============================================================================

/// Default LoadingOptions - uses APP prefix and __ separator
#[derive(Debug, Clone, Default)]
struct DefaultOptions;

impl LoadingOptions for DefaultOptions {
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

/// Custom Options - uses TURTLE prefix and __ separator
#[derive(Debug, Clone, Default)]
struct TurtleOptions;

impl LoadingOptions for TurtleOptions {
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

    /// Override prefix for Turtle naming convention
    fn env_prefix() -> &'static str {
        "TURTLE"
    }
}

/// Custom Options - uses CUSTOM prefix and ___ separator (triple underscore)
#[derive(Debug, Clone, Default)]
struct CustomSeparatorOptions;

impl LoadingOptions for CustomSeparatorOptions {
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

    /// Override separator to triple underscore
    fn env_separator() -> &'static str {
        "___"
    }
}

/// Custom Options - both prefix and separator overridden
#[derive(Debug, Clone, Default)]
struct FullyCustomOptions;

impl LoadingOptions for FullyCustomOptions {
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

    fn env_prefix() -> &'static str {
        "CUSTOM"
    }

    fn env_separator() -> &'static str {
        "_"
    }
}

// ============================================================================
// Test 1: Default Environment Variable Prefix
// ============================================================================

/// Test that default env prefix is "APP"
#[test]
fn test_default_env_prefix() {
    let prefix = DefaultOptions::env_prefix();
    assert_eq!(prefix, "APP", "Default prefix should be 'APP'");
}

// ============================================================================
// Test 2: Default Environment Variable Separator
// ============================================================================

/// Test that default env separator is "__"
#[test]
fn test_default_env_separator() {
    let separator = DefaultOptions::env_separator();
    assert_eq!(separator, "__", "Default separator should be '__' (double underscore)");
}

// ============================================================================
// Test 3: Custom Environment Variable Prefix
// ============================================================================

/// Test that custom prefix can be specified via trait override
#[test]
fn test_custom_env_prefix() {
    let prefix = TurtleOptions::env_prefix();
    assert_eq!(
        prefix, "TURTLE",
        "TurtleOptions should override prefix to 'TURTLE'"
    );
}

// ============================================================================
// Test 4: Custom Environment Variable Separator
// ============================================================================

/// Test that custom separator can be specified via trait override
#[test]
fn test_custom_env_separator() {
    let separator = CustomSeparatorOptions::env_separator();
    assert_eq!(
        separator, "___",
        "CustomSeparatorOptions should override separator to '___'"
    );
}

// ============================================================================
// Test 5: Custom Prefix AND Separator
// ============================================================================

/// Test that both prefix and separator can be customized simultaneously
#[test]
fn test_custom_prefix_and_separator() {
    assert_eq!(
        FullyCustomOptions::env_prefix(),
        "CUSTOM",
        "Prefix should be 'CUSTOM'"
    );
    assert_eq!(
        FullyCustomOptions::env_separator(),
        "_",
        "Separator should be '_' (single underscore)"
    );
}

// ============================================================================
// Test 6: LayerBuilder Respects Custom Prefix
// ============================================================================

/// Test that LayerBuilder.with_env_vars() can use custom prefix from LoadingOptions
#[test]
fn test_env_vars_with_custom_prefix() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(
        &config_path,
        "app_name: BaseApp\nport: 8000\ndebug: false\ndatabase:\n  host: localhost\n  port: 5432\n  user: default",
    )
    .unwrap();

    // Set environment variables with custom prefix
    std::env::set_var("TURTLE_PORT", "9000");
    std::env::set_var("TURTLE_DATABASE__HOST", "prod.example.com");

    // Use custom prefix in LayerBuilder
    let builder = settings_loader::LayerBuilder::new()
        .with_path(&config_path)
        .with_env_vars(TurtleOptions::env_prefix(), TurtleOptions::env_separator());

    let config_builder = builder.build().unwrap();
    let config = config_builder.build().unwrap();
    let result: TestConfig = config.try_deserialize().unwrap();

    // Verify custom prefix values were loaded
    assert_eq!(
        result.port, 9000,
        "Port from TURTLE_PORT should override base config"
    );
    assert_eq!(
        result.database.host, "prod.example.com",
        "Database host from TURTLE_DATABASE__HOST should override"
    );

    std::env::remove_var("TURTLE_PORT");
    std::env::remove_var("TURTLE_DATABASE__HOST");
}

// ============================================================================
// Test 7: LayerBuilder Respects Custom Separator
// ============================================================================

/// Test that LayerBuilder.with_env_vars() can use custom separator from LoadingOptions
#[test]
fn test_env_vars_with_custom_separator() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(
        &config_path,
        "app_name: SeparatorApp\nport: 8000\ndebug: false\ndatabase:\n  host: localhost\n  port: 5432\n  user: default",
    )
    .unwrap();

    // Set environment variables with custom separator (single underscore)
    std::env::set_var("CUSTOM_PORT", "7000");
    std::env::set_var("CUSTOM_DATABASE_HOST", "sep.example.com");

    // Use custom separator in LayerBuilder (single underscore)
    let builder = settings_loader::LayerBuilder::new()
        .with_path(&config_path)
        .with_env_vars(FullyCustomOptions::env_prefix(), FullyCustomOptions::env_separator());

    let config_builder = builder.build().unwrap();
    let config = config_builder.build().unwrap();
    let result: TestConfig = config.try_deserialize().unwrap();

    // Verify custom separator values were loaded
    assert_eq!(
        result.port, 7000,
        "Port from CUSTOM_PORT should override with single separator"
    );
    assert_eq!(
        result.database.host, "sep.example.com",
        "Database host from CUSTOM_DATABASE_HOST should work with custom separator"
    );

    std::env::remove_var("CUSTOM_PORT");
    std::env::remove_var("CUSTOM_DATABASE_HOST");
}

// ============================================================================
// Test 8: Real-World Turtle Naming Convention
// ============================================================================

/// Test real-world Turtle application naming convention (TURTLE__LLM__*)
#[test]
fn test_turtle_style_naming_convention() {
    let prefix = TurtleOptions::env_prefix();
    let separator = TurtleOptions::env_separator();

    // Turtle would use TURTLE__LLM__MODEL, TURTLE__LLM__PROVIDER, etc.
    // Verify the methods return correct values for this convention
    assert_eq!(prefix, "TURTLE", "Turtle prefix incorrect");
    assert_eq!(separator, "__", "Turtle separator incorrect");

    // Example env vars that would work:
    // TURTLE__LLM__MODEL=ollama
    // TURTLE__LLM__PROVIDER=local
    // TURTLE__LLM__OLLAMA__BASE_URL=http://localhost:11434
}

// ============================================================================
// Test 9: Environment Variables Load with Custom Convention
// ============================================================================

/// Test full cycle: load config with custom env var naming convention
#[test]
fn test_env_var_loading_with_custom_convention() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(
        &config_path,
        "app_name: TurtleApp\nport: 8000\ndebug: false\ndatabase:\n  host: localhost\n  port: 5432\n  user: turtle",
    )
    .unwrap();

    // Set environment variables using Turtle convention
    std::env::set_var("TURTLE_APP_NAME", "TurtleCustom");
    std::env::set_var("TURTLE_PORT", "9999");
    std::env::set_var("TURTLE_DEBUG", "true");
    std::env::set_var("TURTLE_DATABASE__HOST", "turtle.db.local");
    std::env::set_var("TURTLE_DATABASE__USER", "turtle_admin");

    let builder = settings_loader::LayerBuilder::new()
        .with_path(&config_path)
        .with_env_vars("TURTLE", "__");

    let config_builder = builder.build().unwrap();
    let config = config_builder.build().unwrap();
    let result: TestConfig = config.try_deserialize().unwrap();

    // Verify Turtle convention worked
    assert_eq!(result.app_name, "TurtleCustom");
    assert_eq!(result.port, 9999);
    assert!(result.debug);
    assert_eq!(result.database.host, "turtle.db.local");
    assert_eq!(result.database.user, "turtle_admin");

    std::env::remove_var("TURTLE_APP_NAME");
    std::env::remove_var("TURTLE_PORT");
    std::env::remove_var("TURTLE_DEBUG");
    std::env::remove_var("TURTLE_DATABASE__HOST");
    std::env::remove_var("TURTLE_DATABASE__USER");
}

// ============================================================================
// Test 10: Backward Compatibility - Default Prefix Still Works
// ============================================================================

/// Test that existing code using default "APP" prefix continues to work
#[test]
fn test_backward_compatibility_default_prefix() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(
        &config_path,
        "app_name: LegacyApp\nport: 8000\ndebug: false\ndatabase:\n  host: localhost\n  port: 5432\n  user: legacy",
    )
    .unwrap();

    // Set environment variables using original APP convention
    std::env::set_var("APP_PORT", "8888");
    std::env::set_var("APP_DATABASE__HOST", "legacy.db.local");

    // Use default prefix (should still be "APP")
    let builder = settings_loader::LayerBuilder::new()
        .with_path(&config_path)
        .with_env_vars("APP", "__");

    let config_builder = builder.build().unwrap();
    let config = config_builder.build().unwrap();
    let result: TestConfig = config.try_deserialize().unwrap();

    assert_eq!(result.port, 8888);
    assert_eq!(result.database.host, "legacy.db.local");

    std::env::remove_var("APP_PORT");
    std::env::remove_var("APP_DATABASE__HOST");
}

// ============================================================================
// Test 11: Backward Compatibility - Default Separator Still Works
// ============================================================================

/// Test that existing code using default "__" separator continues to work
#[test]
fn test_backward_compatibility_default_separator() {
    assert_eq!(
        DefaultOptions::env_separator(),
        "__",
        "Default separator must remain '__' for backward compatibility"
    );

    // Verify the separator works with nested keys
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    fs::write(
        &config_path,
        "app_name: LegacyApp\ndatabase:\n  host: localhost\n  port: 5432\n  user: default",
    )
    .unwrap();

    std::env::set_var("APP_DATABASE__PORT", "3306");

    let builder = settings_loader::LayerBuilder::new()
        .with_path(&config_path)
        .with_env_vars("APP", "__");

    let config_builder = builder.build().unwrap();
    let config = config_builder.build().unwrap();
    let result: TestConfig = config.try_deserialize().unwrap();

    assert_eq!(result.database.port, 3306);

    std::env::remove_var("APP_DATABASE__PORT");
}

// ============================================================================
// Test 12: Multiple Custom Implementations Can Coexist
// ============================================================================

/// Test that different LoadingOptions implementations can have different conventions
#[test]
fn test_multiple_custom_implementations() {
    // TurtleOptions uses TURTLE prefix
    let turtle_prefix = TurtleOptions::env_prefix();
    assert_eq!(turtle_prefix, "TURTLE");

    // FullyCustomOptions uses CUSTOM prefix
    let custom_prefix = FullyCustomOptions::env_prefix();
    assert_eq!(custom_prefix, "CUSTOM");

    // Different separators too
    let default_sep = DefaultOptions::env_separator();
    let custom_sep = FullyCustomOptions::env_separator();

    assert_eq!(default_sep, "__");
    assert_eq!(custom_sep, "_");

    // Multiple implementations can coexist in same application
    // e.g., one LoadingOptions for Turtle app, another for default app
}
