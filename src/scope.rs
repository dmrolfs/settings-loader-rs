//! Configuration scope support for multi-scope configuration management.
//!
//! Provides `ConfigScope` enum for representing different configuration scopes
//! (System, UserGlobal, ProjectLocal, Runtime) and utilities for discovering
//! configuration files across multiple formats.

use std::path::{Path, PathBuf};

/// Represents a configuration scope/layer for multi-scope configuration support.
///
/// Scopes allow applications to load configuration from multiple sources with
/// clear precedence. Later scopes override earlier scopes.
///
/// # Scopes
///
/// - `System` - Immutable system-wide defaults (read-only, typically in /etc)
/// - `UserGlobal` - User preferences that apply everywhere (e.g., ~/.config)
/// - `ProjectLocal` - Project-specific overrides (e.g., ./settings.toml)
/// - `Runtime` - Dynamic configuration from environment variables and CLI flags
///
/// # Examples
///
/// ```ignore
/// use settings_loader::ConfigScope;
///
/// // Create scope instances
/// let system = ConfigScope::System;
/// let user = ConfigScope::UserGlobal;
/// let project = ConfigScope::ProjectLocal;
/// let runtime = ConfigScope::Runtime;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigScope {
    /// System-wide immutable defaults
    System,

    /// User global configuration (platform-specific path)
    UserGlobal,

    /// Project-local configuration (searched in current directory)
    ProjectLocal,

    /// Runtime configuration (environment variables, CLI flags)
    Runtime,
}

impl ConfigScope {
    /// Returns a human-readable name for this scope.
    pub fn name(&self) -> &'static str {
        match self {
            ConfigScope::System => "System",
            ConfigScope::UserGlobal => "UserGlobal",
            ConfigScope::ProjectLocal => "ProjectLocal",
            ConfigScope::Runtime => "Runtime",
        }
    }
}

impl std::fmt::Display for ConfigScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Search for a configuration file in a directory with multiple format extensions.
///
/// This function searches for configuration files in order of preference:
/// 1. `settings.toml`
/// 2. `settings.yaml`
/// 3. `settings.yml`
/// 4. `settings.json`
/// 5. `settings.hjson`
/// 6. `settings.ron`
///
/// Returns the first matching file found, or `None` if no matching file exists.
///
/// # Arguments
///
/// * `dir` - The directory to search
///
/// # Returns
///
/// `Some(PathBuf)` if a config file is found, `None` otherwise
///
/// # Examples
///
/// ```ignore
/// use std::path::Path;
/// use settings_loader::scope::find_config_in;
///
/// if let Some(config_path) = find_config_in(Path::new(".")) {
///     println!("Found config at: {}", config_path.display());
/// }
/// ```
pub fn find_config_in(dir: &Path) -> Option<PathBuf> {
    // Search order: prefer TOML > YAML > JSON > other formats
    let extensions = ["toml", "yaml", "yml", "json", "hjson", "ron"];

    for ext in &extensions {
        let path = dir.join(format!("settings.{}", ext));
        if path.exists() {
            return Some(path);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_name() {
        assert_eq!(ConfigScope::System.name(), "System");
        assert_eq!(ConfigScope::UserGlobal.name(), "UserGlobal");
        assert_eq!(ConfigScope::ProjectLocal.name(), "ProjectLocal");
        assert_eq!(ConfigScope::Runtime.name(), "Runtime");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(ConfigScope::System.to_string(), "System");
        assert_eq!(ConfigScope::UserGlobal.to_string(), "UserGlobal");
    }

    #[test]
    fn test_scope_equality() {
        assert_eq!(ConfigScope::System, ConfigScope::System);
        assert_ne!(ConfigScope::System, ConfigScope::UserGlobal);
    }

    #[test]
    fn test_scope_hash() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(ConfigScope::System, "system");
        map.insert(ConfigScope::UserGlobal, "user");

        assert_eq!(map.get(&ConfigScope::System), Some(&"system"));
        assert_eq!(map.get(&ConfigScope::UserGlobal), Some(&"user"));
    }

    #[test]
    fn test_find_config_in_toml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let toml_path = temp_dir.path().join("settings.toml");
        std::fs::write(&toml_path, "test = true").unwrap();

        let found = find_config_in(temp_dir.path());
        assert_eq!(found, Some(toml_path));
    }

    #[test]
    fn test_find_config_in_yaml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let yaml_path = temp_dir.path().join("settings.yaml");
        std::fs::write(&yaml_path, "test: true").unwrap();

        let found = find_config_in(temp_dir.path());
        assert_eq!(found, Some(yaml_path));
    }

    #[test]
    fn test_find_config_in_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        let json_path = temp_dir.path().join("settings.json");
        std::fs::write(&json_path, r#"{"test": true}"#).unwrap();

        let found = find_config_in(temp_dir.path());
        assert_eq!(found, Some(json_path));
    }

    #[test]
    fn test_find_config_in_priority() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create both TOML and YAML
        let toml_path = temp_dir.path().join("settings.toml");
        std::fs::write(&toml_path, "test = true").unwrap();

        let yaml_path = temp_dir.path().join("settings.yaml");
        std::fs::write(&yaml_path, "test: true").unwrap();

        // Should prefer TOML
        let found = find_config_in(temp_dir.path());
        assert_eq!(found, Some(toml_path));
    }

    #[test]
    fn test_find_config_in_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let found = find_config_in(temp_dir.path());
        assert_eq!(found, None);
    }
}
