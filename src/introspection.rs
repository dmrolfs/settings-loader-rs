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
use crate::validation::{ValidationError, ValidationResult};
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
/// let public_settings = introspector.public_settings();
///
/// // Find settings of a specific type
/// let strings = introspector.settings_of_type(&SettingType::String {
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
    fn schema(&self) -> ConfigSchema;

    /// Get a specific setting metadata by its key
    ///
    /// Returns None if the setting key doesn't exist.
    /// Note: This returns metadata about a setting, not its current value.
    fn get_setting_metadata(&self, key: &str) -> Option<SettingMetadata> {
        self.schema().settings.into_iter().find(|s| s.key == key)
    }

    /// Get all setting groups defined in the schema
    fn groups(&self) -> Vec<SettingGroup> {
        self.schema().groups
    }

    // ========================================================================
    // VISIBILITY FILTERS
    // ========================================================================

    /// Get all settings with Public visibility
    fn public_settings(&self) -> Vec<SettingMetadata> {
        self.schema()
            .settings
            .into_iter()
            .filter(|s| s.visibility == Visibility::Public)
            .collect()
    }

    /// Get all settings with Hidden visibility
    fn hidden_settings(&self) -> Vec<SettingMetadata> {
        self.schema()
            .settings
            .into_iter()
            .filter(|s| s.visibility == Visibility::Hidden)
            .collect()
    }

    /// Get all settings with Secret visibility
    fn secret_settings(&self) -> Vec<SettingMetadata> {
        self.schema()
            .settings
            .into_iter()
            .filter(|s| s.visibility == Visibility::Secret)
            .collect()
    }

    /// Get all settings with Advanced visibility
    fn advanced_settings(&self) -> Vec<SettingMetadata> {
        self.schema()
            .settings
            .into_iter()
            .filter(|s| s.visibility == Visibility::Advanced)
            .collect()
    }

    /// Get all settings in a specific group by group name
    ///
    /// Returns empty vector if group doesn't exist.
    fn settings_in_group(&self, group_name: &str) -> Vec<SettingMetadata> {
        let schema = self.schema();
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
    fn settings_of_type(&self, setting_type: &SettingType) -> Vec<SettingMetadata> {
        self.schema()
            .settings
            .into_iter()
            .filter(|s| s.setting_type == *setting_type)
            .collect()
    }

    /// Get all settings that have a specific constraint
    fn settings_with_constraint(&self, constraint: &Constraint) -> Vec<SettingMetadata> {
        self.schema()
            .settings
            .into_iter()
            .filter(|s| s.constraints.iter().any(|c| c == constraint))
            .collect()
    }

    /// Get all settings with no constraints
    fn unconstrained_settings(&self) -> Vec<SettingMetadata> {
        self.schema()
            .settings
            .into_iter()
            .filter(|s| s.constraints.is_empty())
            .collect()
    }

    /// Get all settings that have multiple constraints (2 or more)
    fn settings_with_multiple_constraints(&self) -> Vec<SettingMetadata> {
        self.schema()
            .settings
            .into_iter()
            .filter(|s| s.constraints.len() > 1)
            .collect()
    }

    // ========================================================================
    // DEFAULT VALUES
    // ========================================================================

    /// Get all settings that have a default value defined
    fn settings_with_defaults(&self) -> Vec<SettingMetadata> {
        self.schema()
            .settings
            .into_iter()
            .filter(|s| s.default.is_some())
            .collect()
    }

    /// Get all settings with no default value
    fn settings_without_defaults(&self) -> Vec<SettingMetadata> {
        self.schema()
            .settings
            .into_iter()
            .filter(|s| s.default.is_none())
            .collect()
    }

    /// Get the default value for a setting by key
    ///
    /// Returns None if setting doesn't exist or has no default.
    fn get_default_value(&self, key: &str) -> Option<serde_json::Value> {
        self.get_setting_metadata(key).and_then(|s| s.default)
    }

    // ========================================================================
    // VALIDATION & CONSTRAINTS (SCHEMA CHECKS)
    // ========================================================================

    /// Validate that a setting with the given key exists
    ///
    /// Returns true if the setting exists, false otherwise.
    fn validate_setting(&self, key: &str) -> bool {
        self.get_setting_metadata(key).is_some()
    }

    /// Validate that a setting with the given key has the expected type
    ///
    /// Returns true if the setting exists and matches the type.
    fn validate_setting_type(&self, key: &str, expected_type: &SettingType) -> bool {
        self.get_setting_metadata(key)
            .map(|s| s.setting_type == *expected_type)
            .unwrap_or(false)
    }

    // ========================================================================
    // VALUE VALIDATION (CONSTRAINT CHECKING)
    // ========================================================================

    /// Validate a single setting value against its metadata constraints
    ///
    /// Looks up the setting by key and validates the value against its type
    /// and constraint requirements.
    ///
    /// # Arguments
    /// * `key` - Setting key to validate
    /// * `value` - JSON value to validate
    ///
    /// # Returns
    /// * `Ok(ValidationResult)` with accumulated errors/warnings
    /// * `Err(ValidationError)` if setting key doesn't exist
    ///
    /// # Example
    ///
    /// ```ignore
    /// use settings_loader::introspection::SettingsIntrospection;
    /// use serde_json::json;
    ///
    /// let config: Box<dyn SettingsIntrospection> = Box::new(my_config);
    /// let result = config.validate_setting_value("port", &json!(8080))?;
    ///
    /// if result.is_valid() {
    ///     println!("Port is valid");
    /// } else {
    ///     for error in result.errors() {
    ///         eprintln!("Validation error: {}", error);
    ///     }
    /// }
    /// ```
    fn validate_setting_value(
        &self, key: &str, value: &serde_json::Value,
    ) -> Result<ValidationResult, ValidationError> {
        match self.get_setting_metadata(key) {
            Some(metadata) => Ok(metadata.validate(value)),
            None => Err(ValidationError::ConstraintViolation {
                key: key.to_string(),
                reason: format!("Setting '{}' not found in schema", key),
            }),
        }
    }

    /// Validate an entire configuration object against all settings
    ///
    /// Iterates through all settings in the schema and validates corresponding
    /// values in the configuration object. Accumulates all errors and warnings.
    ///
    /// # Arguments
    /// * `config` - JSON object with setting values to validate
    ///
    /// # Returns
    /// * `Ok(ValidationResult)` with all validation errors/warnings
    /// * `Err(ValidationError)` on internal validation failures (unexpected)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use settings_loader::introspection::SettingsIntrospection;
    /// use serde_json::json;
    ///
    /// let config: Box<dyn SettingsIntrospection> = Box::new(my_config);
    /// let values = json!({
    ///     "port": 8080,
    ///     "host": "0.0.0.0",
    ///     "timeout_secs": 30
    /// });
    ///
    /// let result = config.validate_config(&values)?;
    /// if !result.is_valid() {
    ///     eprintln!("Configuration has {} errors", result.errors().len());
    /// }
    /// ```
    fn validate_config(&self, config: &serde_json::Value) -> Result<ValidationResult, ValidationError> {
        let mut result = ValidationResult::new();
        let schema = self.schema();

        // Validate each setting
        for setting in schema.settings {
            if let Some(value) = config.get(&setting.key) {
                let validation_result = setting.validate(value);
                result.merge(validation_result);
            }
        }

        Ok(result)
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
        self.schema()
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
        self.schema()
            .settings
            .into_iter()
            .filter(|s| s.description.to_lowercase().contains(&query_lower))
            .collect()
    }

    // ========================================================================
    // STATISTICS & METRICS
    // ========================================================================

    /// Get the total count of all settings
    fn settings_count(&self) -> usize {
        self.schema().settings.len()
    }

    /// Get distribution of settings by visibility level
    ///
    /// Returns a map of visibility levels to setting counts.
    fn visibility_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for setting in &self.schema().settings {
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
    fn type_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        for setting in &self.schema().settings {
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
    fn constraint_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for setting in &self.schema().settings {
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
    use std::time::Instant;

    // ========================================================================
    // HELPER FUNCTIONS - Reusable test utilities
    // ========================================================================

    /// Create a basic test metadata for use in multiple tests
    fn create_simple_metadata() -> SettingMetadata {
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
        }
    }

    /// Create a complex nested settings metadata
    fn create_nested_object_metadata() -> SettingMetadata {
        SettingMetadata {
            key: "database".to_string(),
            label: "Database Config".to_string(),
            description: "Database connection configuration".to_string(),
            setting_type: SettingType::Object {
                fields: vec![
                    SettingMetadata {
                        key: "database.host".to_string(),
                        label: "Database Host".to_string(),
                        description: "Database server hostname".to_string(),
                        setting_type: SettingType::String {
                            pattern: None,
                            min_length: Some(1),
                            max_length: Some(255),
                        },
                        default: Some(json!("localhost")),
                        constraints: vec![Constraint::Required],
                        visibility: Visibility::Public,
                        group: Some("database".to_string()),
                    },
                    SettingMetadata {
                        key: "database.port".to_string(),
                        label: "Database Port".to_string(),
                        description: "Database server port".to_string(),
                        setting_type: SettingType::Integer { min: Some(1), max: Some(65535) },
                        default: Some(json!(5432)),
                        constraints: vec![Constraint::Range { min: 1.0, max: 65535.0 }],
                        visibility: Visibility::Public,
                        group: Some("database".to_string()),
                    },
                    SettingMetadata {
                        key: "database.password".to_string(),
                        label: "Database Password".to_string(),
                        description: "Database authentication password".to_string(),
                        setting_type: SettingType::Secret,
                        default: None,
                        constraints: vec![Constraint::Required],
                        visibility: Visibility::Secret,
                        group: Some("database".to_string()),
                    },
                ],
            },
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: Some("database".to_string()),
        }
    }

    /// Create a test schema with many settings for performance testing
    fn create_large_schema(count: usize) -> ConfigSchema {
        let mut settings = Vec::new();

        for i in 0..count {
            let group_name = if i % 3 == 0 {
                "group_a"
            } else if i % 3 == 1 {
                "group_b"
            } else {
                "group_c"
            };

            let visibility = match i % 4 {
                0 => Visibility::Public,
                1 => Visibility::Hidden,
                2 => Visibility::Secret,
                _ => Visibility::Advanced,
            };

            settings.push(SettingMetadata {
                key: format!("setting_{}", i),
                label: format!("Setting {}", i),
                description: format!("Test setting number {}", i),
                setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                default: if i % 2 == 0 { Some(json!(format!("value_{}", i))) } else { None },
                constraints: if i % 5 == 0 { vec![Constraint::Required] } else { vec![] },
                visibility,
                group: Some(group_name.to_string()),
            });
        }

        ConfigSchema {
            name: "large-app".to_string(),
            version: "1.0.0".to_string(),
            settings,
            groups: vec![
                SettingGroup {
                    name: "group_a".to_string(),
                    label: "Group A".to_string(),
                    description: "First group".to_string(),
                    settings: (0..count)
                        .filter(|i| i % 3 == 0)
                        .map(|i| format!("setting_{}", i))
                        .collect(),
                },
                SettingGroup {
                    name: "group_b".to_string(),
                    label: "Group B".to_string(),
                    description: "Second group".to_string(),
                    settings: (0..count)
                        .filter(|i| i % 3 == 1)
                        .map(|i| format!("setting_{}", i))
                        .collect(),
                },
                SettingGroup {
                    name: "group_c".to_string(),
                    label: "Group C".to_string(),
                    description: "Third group".to_string(),
                    settings: (0..count)
                        .filter(|i| i % 3 == 2)
                        .map(|i| format!("setting_{}", i))
                        .collect(),
                },
            ],
        }
    }

    /// Create an empty schema for edge case testing
    fn create_empty_schema() -> ConfigSchema {
        ConfigSchema {
            name: "empty-app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![],
            groups: vec![],
        }
    }

    // ========================================================================
    // TEST IMPLEMENTATIONS
    // ========================================================================

    /// Test implementation for testing purposes
    struct TestSettings;

    impl SettingsIntrospection for TestSettings {
        fn schema(&self) -> ConfigSchema {
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

    /// Test implementation with complex nested objects
    struct NestedTestSettings;

    impl SettingsIntrospection for NestedTestSettings {
        fn schema(&self) -> ConfigSchema {
            ConfigSchema {
                name: "nested-app".to_string(),
                version: "1.0.0".to_string(),
                settings: vec![create_nested_object_metadata()],
                groups: vec![SettingGroup {
                    name: "database".to_string(),
                    label: "Database Settings".to_string(),
                    description: "Database configuration".to_string(),
                    settings: vec![
                        "database.host".to_string(),
                        "database.port".to_string(),
                        "database.password".to_string(),
                    ],
                }],
            }
        }
    }

    /// Test implementation with empty schema
    struct EmptyTestSettings;

    impl SettingsIntrospection for EmptyTestSettings {
        fn schema(&self) -> ConfigSchema {
            create_empty_schema()
        }
    }

    /// Test implementation with large schema
    struct LargeTestSettings {
        count: usize,
    }

    impl SettingsIntrospection for LargeTestSettings {
        fn schema(&self) -> ConfigSchema {
            create_large_schema(self.count)
        }
    }

    // ========================================================================
    // ORIGINAL TESTS
    // ========================================================================

    #[test]
    fn test_get_schema_returns_full_schema() {
        let settings = TestSettings;
        let schema = settings.schema();

        assert_eq!(schema.name, "test-app");
        assert_eq!(schema.version, "1.0.0");
        assert_eq!(schema.settings.len(), 2);
    }

    #[test]
    fn test_get_setting_by_key() {
        let settings = TestSettings;
        let setting = settings.get_setting_metadata("api_url");

        assert!(setting.is_some());
        let setting = setting.unwrap();
        assert_eq!(setting.key, "api_url");
        assert_eq!(setting.label, "API URL");
    }

    #[test]
    fn test_get_public_settings() {
        let settings = TestSettings;
        let public = settings.public_settings();

        assert_eq!(public.len(), 1);
        assert_eq!(public[0].key, "api_url");
    }

    #[test]
    fn test_get_secret_settings() {
        let settings = TestSettings;
        let secret = settings.secret_settings();

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
        assert_eq!(settings.settings_count(), 2);
    }

    // ========================================================================
    // NEW ENHANCEMENT TESTS
    // ========================================================================

    /// Test introspection with complex nested objects
    #[test]
    fn test_introspection_with_nested_objects_enhanced() {
        let settings = NestedTestSettings;
        let schema = settings.schema();

        assert_eq!(schema.settings.len(), 1);
        let database_setting = &schema.settings[0];
        assert_eq!(database_setting.key, "database");

        if let SettingType::Object { fields } = &database_setting.setting_type {
            assert_eq!(fields.len(), 3);
            assert_eq!(fields[0].key, "database.host");
            assert_eq!(fields[1].key, "database.port");
            assert_eq!(fields[2].key, "database.password");
        } else {
            panic!("Expected Object type");
        }
    }

    /// Test nested object fields have correct constraints
    #[test]
    fn test_nested_object_constraints() {
        let settings = NestedTestSettings;
        let database_setting = settings.get_setting_metadata("database").unwrap();

        if let SettingType::Object { fields } = &database_setting.setting_type {
            let port_field = &fields[1];
            assert!(!port_field.constraints.is_empty());
            assert!(port_field
                .constraints
                .iter()
                .any(|c| matches!(c, Constraint::Range { .. })));
        } else {
            panic!("Expected Object type");
        }
    }

    /// Test empty metadata array edge case
    #[test]
    fn test_empty_metadata_edge_case() {
        let settings = EmptyTestSettings;
        let schema = settings.schema();

        assert!(schema.settings.is_empty());
        assert!(schema.groups.is_empty());
        assert_eq!(settings.settings_count(), 0);
    }

    /// Test queries on empty schema
    #[test]
    fn test_queries_on_empty_schema() {
        let settings = EmptyTestSettings;

        assert!(settings.public_settings().is_empty());
        assert!(settings.secret_settings().is_empty());
        assert!(settings.settings_in_group("any").is_empty());
        assert_eq!(settings.search_settings("test").len(), 0);
    }

    /// Test large schema performance
    #[test]
    fn test_large_schema_performance() {
        let settings = LargeTestSettings { count: 100 };
        let start = Instant::now();

        let _schema = settings.schema();
        let _public = settings.public_settings();
        let _secret = settings.secret_settings();
        let _group_a = settings.settings_in_group("group_a");
        let _search = settings.search_settings("setting");

        let elapsed = start.elapsed();

        // Should complete in reasonable time (< 100ms for 100 settings)
        assert!(elapsed.as_millis() < 100, "Large schema query took {:?}", elapsed);
    }

    /// Test settings_in_group with overlapping group assignments
    #[test]
    fn test_settings_in_group_with_multiple_groups() {
        let settings = LargeTestSettings { count: 30 };

        let group_a = settings.settings_in_group("group_a");
        let group_b = settings.settings_in_group("group_b");
        let group_c = settings.settings_in_group("group_c");

        // Each group should have approximately 10 settings (30/3)
        assert!(!group_a.is_empty());
        assert!(!group_b.is_empty());
        assert!(!group_c.is_empty());

        // No overlap between groups
        let keys_a: Vec<_> = group_a.iter().map(|s| &s.key).collect();
        for setting_b in &group_b {
            assert!(!keys_a.contains(&&setting_b.key));
        }
    }

    /// Test visibility distribution with large schema
    #[test]
    fn test_visibility_distribution_large_schema() {
        let settings = LargeTestSettings { count: 100 };
        let distribution = settings.visibility_distribution();

        // With 100 settings and round-robin visibility assignment,
        // we should have roughly equal distribution
        let public = distribution.get("public").unwrap_or(&0);
        let secret = distribution.get("secret").unwrap_or(&0);

        assert!(*public > 0);
        assert!(*secret > 0);
    }

    /// Test type distribution reporting
    #[test]
    fn test_type_distribution_reporting() {
        let settings = LargeTestSettings { count: 50 };
        let distribution = settings.type_distribution();

        // All settings in large test are String type
        assert!(distribution.contains_key("string"));
        let string_count = distribution.get("string").unwrap_or(&0);
        assert_eq!(*string_count, 50);
    }

    /// Test constraint statistics with large schema
    #[test]
    fn test_constraint_statistics_large_schema() {
        let settings = LargeTestSettings { count: 50 };
        let stats = settings.constraint_statistics();

        // With 50 settings, every 5th has Required constraint (10 total)
        let required_count = stats.get("required").unwrap_or(&0);
        assert_eq!(*required_count, 10);
    }

    /// Test trait object usage
    #[test]
    fn test_trait_object_usage() {
        let introspector: Box<dyn SettingsIntrospection> = Box::new(TestSettings);

        let schema = introspector.schema();
        assert_eq!(schema.name, "test-app");

        let settings = introspector.public_settings();
        assert_eq!(settings.len(), 1);
    }

    /// Test multiple implementations with same schema
    #[test]
    fn test_multiple_introspection_implementations() {
        let impl1 = TestSettings;
        let impl2 = NestedTestSettings;

        // Both should be able to introspect
        let schema1 = impl1.schema();
        let schema2 = impl2.schema();

        assert!(!schema1.name.is_empty());
        assert!(!schema2.name.is_empty());

        // Different schemas
        assert_ne!(schema1.name, schema2.name);
    }

    /// Test special characters in keys
    #[test]
    fn test_settings_with_special_characters_in_keys() {
        let settings = TestSettings;

        // Underscore should work in keys
        let metadata = create_simple_metadata();
        assert_eq!(metadata.key, "api_url");
        assert!(metadata.key.contains('_'));

        // Verify get_setting works with underscores
        let api_url = settings.get_setting_metadata("api_url");
        assert!(api_url.is_some());
    }

    /// Test search with special characters
    #[test]
    fn test_search_with_dotted_keys() {
        let settings = NestedTestSettings;
        let results = settings.search_settings("database");

        // Should find settings with "database" in key, label, or description
        assert!(!results.is_empty());
    }

    /// Test advanced settings filtering
    #[test]
    fn test_get_advanced_settings() {
        let settings = LargeTestSettings { count: 40 };
        let advanced = settings.advanced_settings();

        // With 40 settings and 4-way visibility split, expect ~10 advanced
        assert!(!advanced.is_empty());
        for setting in advanced {
            assert_eq!(setting.visibility, Visibility::Advanced);
        }
    }

    /// Test hidden settings filtering
    #[test]
    fn test_get_hidden_settings() {
        let settings = LargeTestSettings { count: 40 };
        let hidden = settings.hidden_settings();

        // With 40 settings and 4-way visibility split, expect ~10 hidden
        assert!(!hidden.is_empty());
        for setting in hidden {
            assert_eq!(setting.visibility, Visibility::Hidden);
        }
    }

    /// Test settings with defaults
    #[test]
    fn test_get_settings_with_defaults() {
        let settings = LargeTestSettings { count: 20 };
        let with_defaults = settings.settings_with_defaults();

        // Every other setting has a default
        assert!(!with_defaults.is_empty());
        for setting in with_defaults {
            assert!(setting.default.is_some());
        }
    }

    /// Test constraint filtering
    #[test]
    fn test_get_settings_with_specific_constraint() {
        let settings = LargeTestSettings { count: 50 };
        let required = settings.settings_with_constraint(&Constraint::Required);

        // Every 5th setting has Required constraint
        assert!(!required.is_empty());
        for setting in required {
            assert!(setting.constraints.contains(&Constraint::Required));
        }
    }

    // ========================================================================
    // VALIDATION INTEGRATION TESTS
    // ========================================================================

    /// Test validate_setting_value with valid value
    #[test]
    fn test_validate_setting_value_valid() {
        let settings = TestSettings;
        let result = settings.validate_setting_value("api_url", &json!("http://localhost:8080"));

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.is_valid());
    }

    /// Test validate_setting_value with invalid value (wrong type)
    #[test]
    fn test_validate_setting_value_type_mismatch() {
        let settings = TestSettings;
        let result = settings.validate_setting_value("api_url", &json!(8080));

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.is_valid());
        assert_eq!(validation.errors().len(), 1);
    }

    /// Test validate_setting_value with non-existent setting
    #[test]
    fn test_validate_setting_value_not_found() {
        let settings = TestSettings;
        let result = settings.validate_setting_value("nonexistent_key", &json!("value"));

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.key() == Some("nonexistent_key"));
        }
    }

    /// Test validate_setting_value with required constraint
    #[test]
    fn test_validate_setting_value_required_constraint() {
        let settings = TestSettings;

        // api_key has Required constraint
        let result = settings.validate_setting_value("api_key", &json!(null));
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.is_valid());
        assert!(!validation.errors().is_empty());
    }

    /// Test validate_setting_value with proper secret value
    #[test]
    fn test_validate_setting_value_secret_valid() {
        let settings = TestSettings;
        let result = settings.validate_setting_value("api_key", &json!("super-secret-key"));

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.is_valid());
    }

    /// Test validate_config with valid configuration
    #[test]
    fn test_validate_config_valid() {
        let settings = TestSettings;
        let config = json!({
            "api_url": "http://localhost:8080",
            "api_key": "secret-key"
        });

        let result = settings.validate_config(&config);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.is_valid());
    }

    /// Test validate_config with multiple errors
    #[test]
    fn test_validate_config_multiple_errors() {
        let settings = TestSettings;
        let config = json!({
            "api_url": 8080,  // Wrong type (should be string)
            "api_key": null   // Missing required value
        });

        let result = settings.validate_config(&config);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.is_valid());
        assert_eq!(validation.errors().len(), 2);
    }

    /// Test validate_config with partial configuration
    #[test]
    fn test_validate_config_partial() {
        let settings = TestSettings;
        let config = json!({
            "api_url": "http://localhost:8080"
        });

        let result = settings.validate_config(&config);
        assert!(result.is_ok());
        let validation = result.unwrap();
        // api_key is missing (not provided in config), so validation should pass
        // Validation doesn't fail for missing optional values - only validates what's present
        assert!(validation.is_valid());
    }

    /// Test validate_config with extra fields (should be ignored)
    #[test]
    fn test_validate_config_extra_fields() {
        let settings = TestSettings;
        let config = json!({
            "api_url": "http://localhost:8080",
            "api_key": "secret-key",
            "extra_field": "should be ignored"
        });

        let result = settings.validate_config(&config);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.is_valid());
    }

    /// Test validate_config with complex nested configuration
    #[test]
    fn test_validate_config_nested_objects() {
        let settings = NestedTestSettings;
        let config = json!({
            "database": {
                "database.host": "localhost",
                "database.port": 5432,
                "database.password": "db-password"
            }
        });

        let result = settings.validate_config(&config);
        assert!(result.is_ok());
        let validation = result.unwrap();
        // Object validation validates fields if they exist in the nested config
        // Empty config should be valid (no violations found)
        assert!(validation.is_valid());
    }

    /// Test validate_setting_value on main nested object
    #[test]
    fn test_validate_setting_value_nested_object() {
        let settings = NestedTestSettings;

        // database is an Object type containing nested fields
        let config = json!({
            "database.host": "localhost",
            "database.port": 5432,
            "database.password": "secret"
        });
        let result = settings.validate_setting_value("database", &config);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.is_valid());
    }

    /// Test validate_setting_value with nested object type mismatch
    #[test]
    fn test_validate_setting_value_nested_object_type_mismatch() {
        let settings = NestedTestSettings;

        // database should be an object, not a string
        let result = settings.validate_setting_value("database", &json!("not-an-object"));
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.is_valid());
        assert_eq!(validation.errors().len(), 1);
    }

    /// Test validate_config with trait object
    #[test]
    fn test_validate_config_trait_object() {
        let introspector: Box<dyn SettingsIntrospection> = Box::new(TestSettings);

        let config = json!({
            "api_url": "http://localhost:8080",
            "api_key": "secret-key"
        });

        let result = introspector.validate_config(&config);
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.is_valid());
    }

    /// Test validate_setting_value with trait object
    #[test]
    fn test_validate_setting_value_trait_object() {
        let introspector: Box<dyn SettingsIntrospection> = Box::new(TestSettings);

        let result = introspector.validate_setting_value("api_url", &json!("http://localhost:8080"));
        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(validation.is_valid());
    }

    /// Test error context preservation through validation
    #[test]
    fn test_validation_error_context_preservation() {
        let settings = TestSettings;
        let result = settings.validate_setting_value("api_url", &json!(123));

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.is_valid());

        // Error should preserve the key context
        let error = &validation.errors()[0];
        assert_eq!(error.key(), Some("api_url"));
    }

    /// Test validation result merging in validate_config
    #[test]
    fn test_validation_result_merging() {
        let settings = TestSettings;
        let config = json!({
            "api_url": 123,      // Type error
            "api_key": null      // Missing required
        });

        let result = settings.validate_config(&config);
        assert!(result.is_ok());
        let validation = result.unwrap();

        // Both errors should be merged
        assert!(!validation.is_valid());
        assert_eq!(validation.errors().len(), 2);

        // Verify keys are preserved
        let keys: Vec<_> = validation
            .errors()
            .iter()
            .filter_map(|e| e.key().map(String::from))
            .collect();
        assert!(keys.contains(&"api_url".to_string()));
        assert!(keys.contains(&"api_key".to_string()));
    }
}
