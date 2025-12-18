//! Phase 5.2: Settings Introspection Trait
//!
//! This module provides runtime introspection capabilities for configuration schemas.
//! The SettingsIntrospection trait enables querying, filtering, and analyzing configuration
//! metadata programmatically.
//!
//! # Overview
//!
//! The SettingsIntrospection trait provides methods for:
//! - **Schema Retrieval**: Get complete or filtered configuration schemas
//! - **Setting Lookup**: Find settings by key, type, visibility, or constraints
//! - **Grouping & Organization**: Query settings by group
//! - **Type Analysis**: Introspect setting types and constraints
//! - **Search & Query**: Find settings by pattern or criteria
//! - **Statistics**: Analyze configuration distribution and metadata

use crate::metadata::{ConfigSchema, Constraint, SettingGroup, SettingMetadata, SettingType, Visibility};
use std::collections::HashMap;

/// Runtime introspection API for configuration schemas
///
/// This trait enables runtime discovery and analysis of configuration structure,
/// types, constraints, and metadata. It provides comprehensive querying capabilities
/// for building dynamic configuration UIs, documentation, and validation.
///
/// # Examples
///
/// ```ignore
/// use settings_loader::introspection::SettingsIntrospection;
/// use settings_loader::metadata::Visibility;
///
/// let introspector: Box<dyn SettingsIntrospection> = Box::new(my_config);
///
/// // Get all public settings
/// let public_settings = introspector.get_public_settings();
///
/// // Find settings of a specific type
/// let strings = introspector.get_settings_of_type(&SettingType::String {
///     pattern: None,
///     min_length: None,
///     max_length: None,
/// });
///
/// // Search settings by name/description
/// let results = introspector.search_settings("database");
/// ```
pub trait SettingsIntrospection {
    // ========================================================================
    // CORE SCHEMA RETRIEVAL
    // ========================================================================

    /// Get the complete configuration schema
    ///
    /// Returns the full schema including all settings and groups.
    fn get_schema(&self) -> ConfigSchema;

    /// Get a specific setting by its key
    ///
    /// Returns None if the setting key doesn't exist.
    fn get_setting(&self, key: &str) -> Option<SettingMetadata> {
        self.get_schema().settings.into_iter().find(|s| s.key == key)
    }

    /// Get all setting groups defined in the schema
    fn get_groups(&self) -> Vec<SettingGroup> {
        self.get_schema().groups
    }

    // ========================================================================
    // VISIBILITY FILTERS
    // ========================================================================

    /// Get all settings with Public visibility
    fn get_public_settings(&self) -> Vec<SettingMetadata> {
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.visibility == Visibility::Public)
            .collect()
    }

    /// Get all settings with Hidden visibility
    fn get_hidden_settings(&self) -> Vec<SettingMetadata> {
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.visibility == Visibility::Hidden)
            .collect()
    }

    /// Get all settings with Secret visibility
    fn get_secret_settings(&self) -> Vec<SettingMetadata> {
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.visibility == Visibility::Secret)
            .collect()
    }

    /// Get all settings with Advanced visibility
    fn get_advanced_settings(&self) -> Vec<SettingMetadata> {
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.visibility == Visibility::Advanced)
            .collect()
    }

    /// Get all settings in a specific group by group name
    ///
    /// Returns empty vector if group doesn't exist.
    fn get_settings_in_group(&self, group_name: &str) -> Vec<SettingMetadata> {
        let schema = self.get_schema();
        let group_setting_keys: Vec<String> = schema
            .groups
            .iter()
            .filter(|g| g.name == group_name)
            .flat_map(|g| g.settings.clone())
            .collect();

        schema
            .settings
            .into_iter()
            .filter(|s| group_setting_keys.contains(&s.key))
            .collect()
    }

    // ========================================================================
    // TYPE INTROSPECTION
    // ========================================================================

    /// Get all settings of a specific type
    ///
    /// Performs type matching on the setting_type field.
    fn get_settings_of_type(&self, setting_type: &SettingType) -> Vec<SettingMetadata> {
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.setting_type == *setting_type)
            .collect()
    }

    /// Get all settings that have a specific constraint
    fn get_settings_with_constraint(&self, constraint: &Constraint) -> Vec<SettingMetadata> {
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.constraints.iter().any(|c| c == constraint))
            .collect()
    }

    /// Get all settings with no constraints
    fn get_unconstrained_settings(&self) -> Vec<SettingMetadata> {
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.constraints.is_empty())
            .collect()
    }

    /// Get all settings that have multiple constraints (2 or more)
    fn get_settings_with_multiple_constraints(&self) -> Vec<SettingMetadata> {
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.constraints.len() > 1)
            .collect()
    }

    // ========================================================================
    // DEFAULT VALUES
    // ========================================================================

    /// Get all settings that have a default value defined
    fn get_settings_with_defaults(&self) -> Vec<SettingMetadata> {
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.default.is_some())
            .collect()
    }

    /// Get all settings with no default value
    fn get_settings_without_defaults(&self) -> Vec<SettingMetadata> {
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.default.is_none())
            .collect()
    }

    /// Get the default value for a setting by key
    ///
    /// Returns None if setting doesn't exist or has no default.
    fn get_default_value(&self, key: &str) -> Option<serde_json::Value> {
        self.get_setting(key).and_then(|s| s.default)
    }

    // ========================================================================
    // VALIDATION & CONSTRAINTS
    // ========================================================================

    /// Validate that a setting with the given key exists
    ///
    /// Returns true if the setting exists, false otherwise.
    fn validate_setting(&self, key: &str) -> bool {
        self.get_setting(key).is_some()
    }

    /// Validate that a setting with the given key has the expected type
    ///
    /// Returns true if the setting exists and matches the type.
    fn validate_setting_type(&self, key: &str, expected_type: &SettingType) -> bool {
        self.get_setting(key)
            .map(|s| s.setting_type == *expected_type)
            .unwrap_or(false)
    }

    // ========================================================================
    // SEARCH & QUERY
    // ========================================================================

    /// Search for settings by key or label substring matching
    ///
    /// Performs case-insensitive substring matching on both key and label fields.
    fn search_settings(&self, query: &str) -> Vec<SettingMetadata> {
        if query.is_empty() {
            return vec![];
        }

        let query_lower = query.to_lowercase();
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.key.to_lowercase().contains(&query_lower) || s.label.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Search for settings by description
    ///
    /// Performs case-insensitive substring matching on the description field.
    fn search_settings_by_description(&self, query: &str) -> Vec<SettingMetadata> {
        if query.is_empty() {
            return vec![];
        }

        let query_lower = query.to_lowercase();
        self.get_schema()
            .settings
            .into_iter()
            .filter(|s| s.description.to_lowercase().contains(&query_lower))
            .collect()
    }

    // ========================================================================
    // STATISTICS & METRICS
    // ========================================================================

    /// Get the total count of all settings
    fn get_settings_count(&self) -> usize {
        self.get_schema().settings.len()
    }

    /// Get distribution of settings by visibility level
    ///
    /// Returns a map of visibility levels to setting counts.
    fn get_visibility_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for setting in &self.get_schema().settings {
            let visibility_str = match setting.visibility {
                Visibility::Public => "public",
                Visibility::Hidden => "hidden",
                Visibility::Secret => "secret",
                Visibility::Advanced => "advanced",
            };
            *distribution.entry(visibility_str.to_string()).or_insert(0) += 1;
        }
        distribution
    }

    /// Get distribution of settings by type
    ///
    /// Returns a map of type names to setting counts.
    fn get_type_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for setting in &self.get_schema().settings {
            let type_str = match &setting.setting_type {
                SettingType::String { .. } => "string",
                SettingType::Integer { .. } => "integer",
                SettingType::Float { .. } => "float",
                SettingType::Boolean => "boolean",
                SettingType::Duration { .. } => "duration",
                SettingType::Path { .. } => "path",
                SettingType::Url { .. } => "url",
                SettingType::Enum { .. } => "enum",
                SettingType::Secret => "secret",
                SettingType::Array { .. } => "array",
                SettingType::Object { .. } => "object",
                SettingType::Any => "any",
            };
            *distribution.entry(type_str.to_string()).or_insert(0) += 1;
        }
        distribution
    }

    /// Get statistics about constraints in the configuration
    ///
    /// Returns a map with constraint type names to counts.
    fn get_constraint_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for setting in &self.get_schema().settings {
            for constraint in &setting.constraints {
                let constraint_str = match constraint {
                    Constraint::Pattern(_) => "pattern",
                    Constraint::Range { .. } => "range",
                    Constraint::Length { .. } => "length",
                    Constraint::Required => "required",
                    Constraint::OneOf(_) => "oneof",
                    Constraint::Custom(_) => "custom",
                };
                *stats.entry(constraint_str.to_string()).or_insert(0) += 1;
            }
        }
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Test implementation for testing purposes
    struct TestSettings;

    impl SettingsIntrospection for TestSettings {
        fn get_schema(&self) -> ConfigSchema {
            ConfigSchema {
                name: "test-app".to_string(),
                version: "1.0.0".to_string(),
                settings: vec![
                    SettingMetadata {
                        key: "api_url".to_string(),
                        label: "API URL".to_string(),
                        description: "API endpoint URL".to_string(),
                        setting_type: SettingType::Url {
                            schemes: vec!["http".to_string(), "https".to_string()],
                        },
                        default: Some(json!("http://localhost:8080")),
                        constraints: vec![Constraint::Required],
                        visibility: Visibility::Public,
                        group: Some("api".to_string()),
                    },
                    SettingMetadata {
                        key: "api_key".to_string(),
                        label: "API Key".to_string(),
                        description: "Authentication key for API".to_string(),
                        setting_type: SettingType::Secret,
                        default: None,
                        constraints: vec![Constraint::Required],
                        visibility: Visibility::Secret,
                        group: Some("api".to_string()),
                    },
                ],
                groups: vec![SettingGroup {
                    name: "api".to_string(),
                    label: "API Settings".to_string(),
                    description: "Configuration for API".to_string(),
                    settings: vec!["api_url".to_string(), "api_key".to_string()],
                }],
            }
        }
    }

    #[test]
    fn test_get_schema_returns_full_schema() {
        let settings = TestSettings;
        let schema = settings.get_schema();

        assert_eq!(schema.name, "test-app");
        assert_eq!(schema.version, "1.0.0");
        assert_eq!(schema.settings.len(), 2);
    }

    #[test]
    fn test_get_setting_by_key() {
        let settings = TestSettings;
        let setting = settings.get_setting("api_url");

        assert!(setting.is_some());
        let setting = setting.unwrap();
        assert_eq!(setting.key, "api_url");
        assert_eq!(setting.label, "API URL");
    }

    #[test]
    fn test_get_public_settings() {
        let settings = TestSettings;
        let public = settings.get_public_settings();

        assert_eq!(public.len(), 1);
        assert_eq!(public[0].key, "api_url");
    }

    #[test]
    fn test_get_secret_settings() {
        let settings = TestSettings;
        let secret = settings.get_secret_settings();

        assert_eq!(secret.len(), 1);
        assert_eq!(secret[0].key, "api_key");
    }

    #[test]
    fn test_search_settings() {
        let settings = TestSettings;
        let results = settings.search_settings("api");

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_settings_empty_query() {
        let settings = TestSettings;
        let results = settings.search_settings("");

        assert!(results.is_empty());
    }

    #[test]
    fn test_get_settings_count() {
        let settings = TestSettings;
        assert_eq!(settings.get_settings_count(), 2);
    }
}
