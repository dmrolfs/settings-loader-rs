//! Configuration scope definitions and utilities.
//!
//! This module provides the `ConfigScope` enum for representing different configuration
//! scopes (preferences, user-global, project-local, data storage, runtime) and utility
//! functions for discovering configuration files.

use std::path::{Path, PathBuf};

// ============================================================================
// ConfigScope Enum
// ============================================================================

/// Configuration scope representing different layers of configuration sources.
///
/// The scope system allows applications to organize configuration across multiple
/// layers with clear precedence: defaults → user prefs → global config → project config
/// → local data → persistent data → runtime overrides.
///
/// # Scopes
///
/// - **Preferences**: User application preferences (via `BaseDirs::preference_dir()`)
/// - **UserGlobal**: User configuration applying across all projects (via `ProjectDirs::config_dir()`)
/// - **ProjectLocal**: Project-specific configuration (current directory)
/// - **LocalData**: Machine-local data not synced across machines (via `BaseDirs::data_local_dir()`)
/// - **PersistentData**: Cross-machine persistent application state (via `BaseDirs::data_dir()`)
/// - **Runtime**: Dynamic configuration from environment variables and CLI (not file-based)
///
/// # Precedence Order
///
/// When using `MultiScopeConfig`, scopes are typically loaded in this order:
/// 1. Preferences
/// 2. UserGlobal
/// 3. ProjectLocal
/// 4. LocalData
/// 5. PersistentData
/// 6. Runtime (env vars + CLI, handled separately)
///
/// Later scopes override earlier ones during configuration merging.
///
/// # Example
///
/// ```ignore
/// use settings_loader::ConfigScope;
///
/// let scopes = vec![
///     ConfigScope::Preferences,
///     ConfigScope::UserGlobal,
///     ConfigScope::ProjectLocal,
/// ];
///
/// // Use in collections
/// let mut scope_map = std::collections::HashMap::new();
/// scope_map.insert(ConfigScope::Preferences, "user prefs");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigScope {
    /// User application preferences
    ///
    /// Platform-specific locations:
    /// - Linux: `~/.config/APP_NAME` (or XDG_CONFIG_HOME/APP_NAME)
    /// - macOS: `~/Library/Preferences/APP_NAME`
    /// - Windows: `%APPDATA%/APP_NAME`
    Preferences,

    /// User configuration applying across all projects
    ///
    /// Platform-specific locations:
    /// - Linux: `~/.config/APP_NAME` (or XDG_CONFIG_HOME/APP_NAME)
    /// - macOS: `~/Library/Application Support/ORG_NAME.APP_NAME`
    /// - Windows: `%APPDATA%/ORG_NAME/APP_NAME`
    UserGlobal,

    /// Project-specific configuration in current directory
    ///
    /// Location: `./settings.{ext}` (any supported format)
    /// Searchable with `find_config_in()`.
    ProjectLocal,

    /// Machine-local data not synced across machines
    ///
    /// Platform-specific locations:
    /// - Linux: `~/.cache/APP_NAME` (or XDG_CACHE_HOME/APP_NAME)
    /// - macOS: `~/Library/Caches/ORG_NAME.APP_NAME`
    /// - Windows: `%LOCALAPPDATA%/ORG_NAME/APP_NAME`
    ///
    /// Use for ephemeral data, caches, runtime state.
    LocalData,

    /// Cross-machine persistent application state
    ///
    /// Platform-specific locations:
    /// - Linux: `~/.local/share/APP_NAME` (or XDG_DATA_HOME/APP_NAME)
    /// - macOS: `~/Library/Application Support/ORG_NAME.APP_NAME`
    /// - Windows: `%APPDATA%/ORG_NAME/APP_NAME`
    ///
    /// Use for persistent data that should sync across machines.
    PersistentData,

    /// Dynamic configuration from environment variables and CLI
    ///
    /// Not file-based. Resolved separately via environment variables
    /// and command-line arguments. `resolve_path()` returns `None` for this scope.
    Runtime,
}

impl ConfigScope {
    /// Returns a human-readable name for this scope
    ///
    /// # Example
    ///
    /// ```ignore
    /// assert_eq!(ConfigScope::Preferences.name(), "Preferences");
    /// assert_eq!(ConfigScope::UserGlobal.name(), "UserGlobal");
    /// ```
    pub fn name(self) -> &'static str {
        match self {
            ConfigScope::Preferences => "Preferences",
            ConfigScope::UserGlobal => "UserGlobal",
            ConfigScope::ProjectLocal => "ProjectLocal",
            ConfigScope::LocalData => "LocalData",
            ConfigScope::PersistentData => "PersistentData",
            ConfigScope::Runtime => "Runtime",
        }
    }

    /// Returns true if this scope is file-based
    ///
    /// Returns false for `Runtime` scope which is handled via env vars + CLI.
    ///
    /// # Example
    ///
    /// ```ignore
    /// assert!(ConfigScope::Preferences.is_file_based());
    /// assert!(!ConfigScope::Runtime.is_file_based());
    /// ```
    pub fn is_file_based(self) -> bool {
        matches!(
            self,
            ConfigScope::Preferences
                | ConfigScope::UserGlobal
                | ConfigScope::ProjectLocal
                | ConfigScope::LocalData
                | ConfigScope::PersistentData
        )
    }
}

// ============================================================================
// Configuration File Discovery
// ============================================================================

/// Search for a configuration file in the given directory.
///
/// Searches for configuration files with multiple format extensions in order of preference:
/// 1. `.toml` (TOML format - preferred)
/// 2. `.yaml` (YAML format)
/// 3. `.yml` (YAML short form)
/// 4. `.json` (JSON format)
/// 5. `.hjson` (HJSON format)
/// 6. `.ron` (RON format)
///
/// Returns the path to the first matching file, or `None` if no configuration file is found.
///
/// # Arguments
///
/// * `dir` - Directory to search for configuration files
///
/// # Example
///
/// ```ignore
/// use std::path::PathBuf;
/// use settings_loader::scope::find_config_in;
///
/// let config_dir = PathBuf::from("./config");
/// if let Some(path) = find_config_in(&config_dir) {
///     println!("Found config at: {}", path.display());
/// }
/// ```
pub fn find_config_in(dir: &Path) -> Option<PathBuf> {
    // Search in order: toml > yaml > yml > json > hjson > ron
    for ext in &["toml", "yaml", "yml", "json", "hjson", "ron"] {
        let path = dir.join(format!("settings.{}", ext));
        if path.exists() && path.is_file() {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_scope_names() {
        assert_eq!(ConfigScope::Preferences.name(), "Preferences");
        assert_eq!(ConfigScope::UserGlobal.name(), "UserGlobal");
        assert_eq!(ConfigScope::ProjectLocal.name(), "ProjectLocal");
        assert_eq!(ConfigScope::LocalData.name(), "LocalData");
        assert_eq!(ConfigScope::PersistentData.name(), "PersistentData");
        assert_eq!(ConfigScope::Runtime.name(), "Runtime");
    }

    #[test]
    fn test_config_scope_is_file_based() {
        assert!(ConfigScope::Preferences.is_file_based());
        assert!(ConfigScope::UserGlobal.is_file_based());
        assert!(ConfigScope::ProjectLocal.is_file_based());
        assert!(ConfigScope::LocalData.is_file_based());
        assert!(ConfigScope::PersistentData.is_file_based());
        assert!(!ConfigScope::Runtime.is_file_based());
    }

    #[test]
    fn test_config_scope_equality() {
        let scope1 = ConfigScope::Preferences;
        let scope2 = ConfigScope::Preferences;
        let scope3 = ConfigScope::UserGlobal;

        assert_eq!(scope1, scope2);
        assert_ne!(scope1, scope3);
    }

    #[test]
    fn test_config_scope_in_collections() {
        use std::collections::{HashMap, HashSet};

        let mut scope_map: HashMap<ConfigScope, &str> = HashMap::new();
        scope_map.insert(ConfigScope::Preferences, "user_prefs");
        scope_map.insert(ConfigScope::UserGlobal, "user_config");

        assert_eq!(scope_map.get(&ConfigScope::Preferences), Some(&"user_prefs"));

        let mut scope_set: HashSet<ConfigScope> = HashSet::new();
        scope_set.insert(ConfigScope::ProjectLocal);
        scope_set.insert(ConfigScope::LocalData);

        assert!(scope_set.contains(&ConfigScope::ProjectLocal));
        assert!(!scope_set.contains(&ConfigScope::Preferences));
    }

    #[test]
    fn test_find_config_in_empty_directory() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let result = find_config_in(temp_dir.path());

        assert_eq!(result, None);
    }

    #[test]
    fn test_find_config_in_toml() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");
        fs::write(&toml_path, "key = \"value\"").unwrap();

        let result = find_config_in(temp_dir.path());
        assert_eq!(result, Some(toml_path));
    }

    #[test]
    fn test_find_config_in_yaml() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("settings.yaml");
        fs::write(&yaml_path, "key: value").unwrap();

        let result = find_config_in(temp_dir.path());
        assert_eq!(result, Some(yaml_path));
    }

    #[test]
    fn test_find_config_in_json() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("settings.json");
        fs::write(&json_path, r#"{"key": "value"}"#).unwrap();

        let result = find_config_in(temp_dir.path());
        assert_eq!(result, Some(json_path));
    }

    #[test]
    fn test_find_config_in_prefers_toml_over_yaml() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");
        let yaml_path = temp_dir.path().join("settings.yaml");

        fs::write(&toml_path, "key = \"toml\"").unwrap();
        fs::write(&yaml_path, "key: yaml").unwrap();

        let result = find_config_in(temp_dir.path());
        assert_eq!(result, Some(toml_path));
    }

    #[test]
    fn test_find_config_in_prefers_yaml_over_json() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let yaml_path = temp_dir.path().join("settings.yaml");
        let json_path = temp_dir.path().join("settings.json");

        fs::write(&yaml_path, "key: yaml").unwrap();
        fs::write(&json_path, r#"{"key": "json"}"#).unwrap();

        let result = find_config_in(temp_dir.path());
        assert_eq!(result, Some(yaml_path));
    }

    #[test]
    fn test_find_config_in_all_extensions() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create all format files
        let toml_path = temp_dir.path().join("settings.toml");
        let yaml_path = temp_dir.path().join("settings.yaml");
        let yml_path = temp_dir.path().join("settings.yml");
        let json_path = temp_dir.path().join("settings.json");
        let hjson_path = temp_dir.path().join("settings.hjson");
        let ron_path = temp_dir.path().join("settings.ron");

        fs::write(&toml_path, "").unwrap();
        fs::write(&yaml_path, "").unwrap();
        fs::write(&yml_path, "").unwrap();
        fs::write(&json_path, "").unwrap();
        fs::write(&hjson_path, "").unwrap();
        fs::write(&ron_path, "").unwrap();

        // Should prefer TOML
        let result = find_config_in(temp_dir.path());
        assert_eq!(result, Some(toml_path));
    }

    #[test]
    fn test_find_config_in_with_yml_short_form() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create only .yml (not .yaml)
        let yml_path = temp_dir.path().join("settings.yml");
        fs::write(&yml_path, "key: value").unwrap();

        let result = find_config_in(temp_dir.path());
        assert_eq!(result, Some(yml_path));
    }

    #[test]
    fn test_find_config_in_nonexistent_directory() {
        let nonexistent = PathBuf::from("/tmp/nonexistent_config_dir_12345");
        let result = find_config_in(&nonexistent);

        // Should return None gracefully (path doesn't exist)
        assert_eq!(result, None);
    }
}
