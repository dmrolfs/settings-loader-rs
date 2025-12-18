//! Phase 5: Settings Metadata & Introspection
//!
//! This module provides core types for describing, introspecting, and validating
//! application configuration settings at runtime.
//!
//! # Core Types
//!
//! - [`Visibility`] - Controls UI display of settings
//! - [`Constraint`] - Additional validation rules
//! - [`SettingType`] - Rich type system with built-in validation
//! - [`SettingGroup`] - Organize settings into logical groups
//! - [`SettingMetadata`] - Complete description of a single setting
//! - [`ConfigSchema`] - Schema for an application's entire configuration

use std::time::Duration;

/// Controls UI visibility and display of settings
///
/// # Examples
///
/// ```ignore
/// use settings_loader::metadata::Visibility;
///
/// let public = Visibility::Public;
/// let secret = Visibility::Secret;
/// let adv = Visibility::Advanced;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum Visibility {
    /// Always visible in UI
    #[default]
    Public,
    /// Hidden from UI (accessible programmatically)
    Hidden,
    /// Secret value (show masked/redacted in UI)
    Secret,
    /// Advanced setting (show in "Advanced" section)
    Advanced,
}

/// Additional validation constraints beyond type checking
///
/// # Examples
///
/// ```ignore
/// use settings_loader::metadata::Constraint;
///
/// let required = Constraint::Required;
/// let range = Constraint::Range { min: 1.0, max: 100.0 };
/// let pattern = Constraint::Pattern("[0-9]+".to_string());
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "constraint", rename_all = "snake_case"))]
pub enum Constraint {
    /// Must match regex pattern
    Pattern(String),
    /// Numeric value must be in range [min, max]
    Range { min: f64, max: f64 },
    /// String length must be in range [min, max]
    Length { min: usize, max: usize },
    /// Value is required (cannot be None/null)
    Required,
    /// Value must be one of the specified options
    OneOf(Vec<String>),
    /// Custom validation (name only, validator in application)
    Custom(String),
}

/// Rich type system with built-in validation hints
///
/// Supports 12 variants covering primitives, complex types, and recursive structures.
///
/// # Examples
///
/// ```ignore
/// use settings_loader::metadata::SettingType;
/// use std::time::Duration;
///
/// let string = SettingType::String {
///     pattern: Some("[a-z]+".to_string()),
///     min_length: Some(1),
///     max_length: Some(255),
/// };
///
/// let integer = SettingType::Integer {
///     min: Some(1),
///     max: Some(300),
/// };
///
/// let url = SettingType::Url {
///     schemes: vec!["http".to_string(), "https".to_string()],
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "snake_case"))]
pub enum SettingType {
    /// String value with optional constraints
    String {
        /// Regex pattern to match
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pattern: Option<String>,
        /// Minimum string length
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        min_length: Option<usize>,
        /// Maximum string length
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        max_length: Option<usize>,
    },
    /// Signed integer with optional range
    Integer {
        /// Minimum value
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        min: Option<i64>,
        /// Maximum value
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        max: Option<i64>,
    },
    /// Floating point with optional range
    Float {
        /// Minimum value
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        min: Option<f64>,
        /// Maximum value
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        max: Option<f64>,
    },
    /// Boolean flag
    Boolean,
    /// Duration with optional range
    Duration {
        /// Minimum duration
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        min: Option<Duration>,
        /// Maximum duration
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        max: Option<Duration>,
    },
    /// Filesystem path
    Path {
        /// Path must exist on filesystem
        #[cfg_attr(feature = "serde", serde(default))]
        must_exist: bool,
        /// Path must be a directory (not a file)
        #[cfg_attr(feature = "serde", serde(default))]
        is_directory: bool,
    },
    /// URL with optional scheme restrictions
    Url {
        /// Allowed schemes (empty = all allowed)
        #[cfg_attr(feature = "serde", serde(default))]
        schemes: Vec<String>,
    },
    /// Enum with fixed set of string variants
    Enum {
        /// Available option values
        variants: Vec<String>,
    },
    /// Secret value (password, API key, etc.)
    Secret,
    /// Array of values
    Array {
        /// Type of array elements
        element_type: Box<SettingType>,
        /// Minimum number of items
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        min_items: Option<usize>,
        /// Maximum number of items
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        max_items: Option<usize>,
    },
    /// Nested object with fields
    Object {
        /// Object field metadata
        fields: Vec<SettingMetadata>,
    },
    /// Any type (no validation)
    Any,
}

/// Organize settings into logical groups
///
/// Groups provide UI organization for settings. A setting belongs to at most one group,
/// referenced by the group's name.
///
/// # Examples
///
/// ```ignore
/// use settings_loader::metadata::SettingGroup;
///
/// let group = SettingGroup {
///     name: "api".to_string(),
///     label: "API Settings".to_string(),
///     description: "Configuration for API endpoints".to_string(),
///     settings: vec![
///         "api_url".to_string(),
///         "api_key".to_string(),
///         "timeout_secs".to_string(),
///     ],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SettingGroup {
    /// Group identifier (machine-readable)
    pub name: String,
    /// Human-readable label for UI
    pub label: String,
    /// Group description/documentation
    pub description: String,
    /// Setting keys in this group
    pub settings: Vec<String>,
}

/// Complete description of a single configuration setting
///
/// Combines type information, validation constraints, UI hints, and documentation
/// into a single metadata object describing a configuration parameter.
///
/// # Examples
///
/// ```ignore
/// use settings_loader::metadata::{SettingMetadata, SettingType, Visibility, Constraint};
/// use serde_json::json;
///
/// let metadata = SettingMetadata {
///     key: "api_url".to_string(),
///     label: "API URL".to_string(),
///     description: "API endpoint URL".to_string(),
///     setting_type: SettingType::Url {
///         schemes: vec!["http".to_string(), "https".to_string()],
///     },
///     default: Some(json!("http://localhost:8080")),
///     constraints: vec![Constraint::Required],
///     visibility: Visibility::Public,
///     group: Some("api".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SettingMetadata {
    /// Setting key (dot-separated path, e.g., "database.host")
    pub key: String,
    /// Human-readable label for UI display
    pub label: String,
    /// Detailed description/documentation
    pub description: String,
    /// Type information with built-in constraints
    pub setting_type: SettingType,
    /// Default value (JSON for flexibility across types)
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub default: Option<serde_json::Value>,
    /// Additional validation constraints
    #[cfg_attr(feature = "serde", serde(default))]
    pub constraints: Vec<Constraint>,
    /// UI visibility control
    #[cfg_attr(feature = "serde", serde(default))]
    pub visibility: Visibility,
    /// Group/category for organization
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub group: Option<String>,
}

/// Complete schema for an application's settings
///
/// Provides a comprehensive description of all configuration parameters,
/// their types, constraints, defaults, and organization.
///
/// # Examples
///
/// ```ignore
/// use settings_loader::metadata::{ConfigSchema, SettingMetadata, SettingType, SettingGroup};
///
/// let schema = ConfigSchema {
///     name: "my-app".to_string(),
///     version: "1.0.0".to_string(),
///     settings: vec![
///         SettingMetadata {
///             key: "api_url".to_string(),
///             label: "API URL".to_string(),
///             description: "API endpoint".to_string(),
///             setting_type: SettingType::String {
///                 pattern: None,
///                 min_length: None,
///                 max_length: None,
///             },
///             default: None,
///             constraints: vec![],
///             visibility: Default::default(),
///             group: Some("api".to_string()),
///         },
///     ],
///     groups: vec![
///         SettingGroup {
///             name: "api".to_string(),
///             label: "API Settings".to_string(),
///             description: "API configuration".to_string(),
///             settings: vec!["api_url".to_string()],
///         },
///     ],
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConfigSchema {
    /// Application name
    pub name: String,
    /// Schema version (semantic versioning recommended)
    pub version: String,
    /// All setting metadata (flat list)
    pub settings: Vec<SettingMetadata>,
    /// Optional: Grouped settings for UI organization
    #[cfg_attr(feature = "serde", serde(default))]
    pub groups: Vec<SettingGroup>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visibility_default_is_public() {
        assert_eq!(Visibility::default(), Visibility::Public);
    }

    #[test]
    fn visibility_is_copy() {
        let v1 = Visibility::Secret;
        let v2 = v1;
        assert_eq!(v1, v2);
    }
}
