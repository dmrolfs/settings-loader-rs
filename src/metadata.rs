//! Settings Metadata & Introspection
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
//!
//! # Feature: `metadata`
//!
//! This module requires the `metadata` feature.

use serde_json::{json, Value};
use std::collections::HashMap;
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

impl SettingType {
    /// Convert this type into a JSON Schema fragment.
    pub fn to_json_schema(&self) -> Value {
        match self {
            SettingType::String { pattern, min_length, max_length } => {
                let mut schema = json!({ "type": "string" });
                if let Some(p) = pattern {
                    schema["pattern"] = json!(p);
                }
                if let Some(min) = min_length {
                    schema["minLength"] = json!(min);
                }
                if let Some(max) = max_length {
                    schema["maxLength"] = json!(max);
                }
                schema
            },
            SettingType::Integer { min, max } => {
                let mut schema = json!({ "type": "integer" });
                if let Some(m) = min {
                    schema["minimum"] = json!(m);
                }
                if let Some(m) = max {
                    schema["maximum"] = json!(m);
                }
                schema
            },
            SettingType::Float { min, max } => {
                let mut schema = json!({ "type": "number" });
                if let Some(m) = min {
                    schema["minimum"] = json!(m);
                }
                if let Some(m) = max {
                    schema["maximum"] = json!(m);
                }
                schema
            },
            SettingType::Boolean => json!({ "type": "boolean" }),
            SettingType::Duration { .. } => {
                // Durations are often represented as strings or numbers in config
                // For JSON schema, we'll treat them as strings for now
                json!({ "type": "string" })
            },
            SettingType::Path { .. } => json!({ "type": "string" }),
            SettingType::Url { .. } => json!({ "type": "string", "format": "uri" }),
            SettingType::Enum { variants } => json!({ "type": "string", "enum": variants }),
            SettingType::Secret => json!({ "type": "string" }),
            SettingType::Array { element_type, min_items, max_items } => {
                let mut schema = json!({
                    "type": "array",
                    "items": element_type.to_json_schema()
                });
                if let Some(min) = min_items {
                    schema["minItems"] = json!(min);
                }
                if let Some(max) = max_items {
                    schema["maxItems"] = json!(max);
                }
                schema
            },
            SettingType::Object { fields } => {
                let mut properties = json!({});
                let mut required = Vec::new();

                for field in fields {
                    properties[field.key.as_str()] = field.to_json_schema();
                    if field.is_required() {
                        required.push(field.key.clone());
                    }
                }

                let mut schema = json!({
                    "type": "object",
                    "properties": properties
                });

                if !required.is_empty() {
                    schema["required"] = json!(required);
                }

                schema
            },
            SettingType::Any => json!({}),
        }
    }
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

impl SettingMetadata {
    /// Returns true if this setting is marked as required.
    pub fn is_required(&self) -> bool {
        self.constraints.iter().any(|c| matches!(c, Constraint::Required))
    }

    /// Convert this metadata into a JSON Schema fragment.
    pub fn to_json_schema(&self) -> Value {
        let mut schema = self.setting_type.to_json_schema();

        // Add common metadata
        if !self.label.is_empty() {
            schema["title"] = json!(self.label);
        }
        if !self.description.is_empty() {
            schema["description"] = json!(self.description);
        }
        if let Some(default) = &self.default {
            schema["default"] = default.clone();
        }

        // Add constraints that aren't already handled by SettingType
        for constraint in &self.constraints {
            match constraint {
                Constraint::Pattern(p) => schema["pattern"] = json!(p),
                Constraint::Range { min, max } => {
                    schema["minimum"] = json!(min);
                    schema["maximum"] = json!(max);
                },
                Constraint::Length { min, max } => {
                    schema["minLength"] = json!(min);
                    schema["maxLength"] = json!(max);
                },
                Constraint::OneOf(options) => schema["enum"] = json!(options),
                Constraint::Required | Constraint::Custom(_) => {},
            }
        }

        schema
    }
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

impl ConfigSchema {
    /// Convert the entire configuration schema into a full JSON Schema document.
    pub fn to_json_schema(&self) -> Value {
        let mut properties = json!({});
        let mut required = Vec::new();

        // If settings are a flat list with dot-separated keys, we should ideally
        // reconstruct the hierarchy. But for now, let's see if we can just
        // iterate and build it.

        for setting in &self.settings {
            self.insert_into_schema(&mut properties, &mut required, &setting.key, setting);
        }

        json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": self.name,
            "description": format!("Configuration schema for {} (v{})", self.name, self.version),
            "type": "object",
            "properties": properties,
            "required": if required.is_empty() { Value::Null } else { json!(required) }
        })
    }

    fn insert_into_schema(
        &self, properties: &mut Value, required_list: &mut Vec<String>, key: &str, setting: &SettingMetadata,
    ) {
        let parts: Vec<&str> = key.split('.').collect();
        Self::insert_recursive(properties, required_list, &parts, setting);
    }

    fn insert_recursive(
        current_obj: &mut Value, required_list: &mut Vec<String>, parts: &[&str], setting: &SettingMetadata,
    ) {
        if parts.is_empty() {
            return;
        }

        let head = parts[0];
        if parts.len() == 1 {
            current_obj[head] = setting.to_json_schema();
            if setting.is_required() {
                required_list.push(head.to_string());
            }
        } else {
            // Need to nest
            if current_obj[head].is_null() {
                current_obj[head] = json!({
                    "type": "object",
                    "properties": {}
                });
            }

            let mut nested_required = Vec::new();

            // Check if there's already a required field in the JSON
            if let Some(req) = current_obj[head].get("required") {
                if let Some(arr) = req.as_array() {
                    nested_required = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                }
            }

            let nested_properties = current_obj[head].get_mut("properties").unwrap();
            Self::insert_recursive(nested_properties, &mut nested_required, &parts[1..], setting);

            if !nested_required.is_empty() {
                current_obj[head]["required"] = json!(nested_required);
            }
        }
    }

    /// Generate an HTML documentation page for this configuration schema.
    pub fn to_html(&self) -> String {
        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("    <meta charset=\"UTF-8\">\n");
        html.push_str(&format!("    <title>{} Configuration Reference</title>\n", self.name));
        html.push_str("    <style>\n");
        html.push_str("        body { font-family: -apple-system, BlinkMacSystemFont, \"Segoe UI\", Roboto, Helvetica, Arial, sans-serif; line-height: 1.6; color: #333; max-width: 1000px; margin: 0 auto; padding: 2rem; }\n");
        html.push_str("        h1 { border-bottom: 2px solid #eee; padding-bottom: 0.5rem; color: #2c3e50; }\n");
        html.push_str("        h2 { color: #34495e; margin-top: 2rem; border-bottom: 1px solid #eee; }\n");
        html.push_str("        .group { margin-bottom: 3rem; }\n");
        html.push_str("        .setting { background: #f8f9fa; border: 1px solid #e9ecef; border-radius: 4px; padding: 1.5rem; margin-bottom: 1rem; }\n");
        html.push_str("        .setting-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.5rem; }\n");
        html.push_str(
            "        .setting-key { font-family: monospace; font-weight: bold; font-size: 1.1rem; color: #007bff; }\n",
        );
        html.push_str("        .setting-type { font-size: 0.85rem; color: #6c757d; background: #e9ecef; padding: 0.2rem 0.5rem; border-radius: 3px; }\n");
        html.push_str("        .setting-label { font-size: 0.9rem; font-weight: 500; color: #495057; display: block; margin-bottom: 0.5rem; }\n");
        html.push_str("        .setting-desc { margin-bottom: 1rem; }\n");
        html.push_str("        .setting-details { font-size: 0.9rem; color: #495057; border-top: 1px solid #dee2e6; padding-top: 0.5rem; display: flex; gap: 2rem; }\n");
        html.push_str("        .detail-item { display: flex; flex-direction: column; }\n");
        html.push_str("        .detail-label { font-weight: bold; font-size: 0.75rem; text-transform: uppercase; color: #adb5bd; }\n");
        html.push_str("        .badge { display: inline-block; padding: 0.2rem 0.4rem; font-size: 0.75rem; font-weight: bold; border-radius: 3px; margin-left: 0.5rem; }\n");
        html.push_str("        .badge-secret { background: #dc3545; color: white; }\n");
        html.push_str("        .badge-required { background: #ffc107; color: #212529; }\n");
        html.push_str("    </style>\n</head>\n<body>\n");
        html.push_str(&format!("    <h1>{} Configuration Reference</h1>\n", self.name));
        html.push_str(&format!("    <p>Version: {}</p>\n", self.version));

        // Group settings
        let mut grouped_settings: HashMap<String, Vec<&SettingMetadata>> = HashMap::new();
        for setting in &self.settings {
            let group = setting.group.clone().unwrap_or_else(|| "General".to_string());
            grouped_settings.entry(group).or_default().push(setting);
        }

        // Sort groups: General first, then alphabetical
        let mut group_names: Vec<String> = grouped_settings.keys().cloned().collect();
        group_names.sort_by(|a, b| {
            if a == "General" {
                std::cmp::Ordering::Less
            } else if b == "General" {
                std::cmp::Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        for group_name in group_names {
            let group_info = self.groups.iter().find(|g| g.name == group_name);
            html.push_str(&format!(
                "    <div class=\"group\">\n        <h2>{}</h2>\n",
                group_info.map(|g| g.label.as_str()).unwrap_or(&group_name)
            ));
            if let Some(group_info) = group_info {
                html.push_str(&format!("        <p>{}</p>\n", group_info.description));
            }

            for setting in grouped_settings.get(&group_name).unwrap() {
                html.push_str("        <div class=\"setting\">\n");
                html.push_str("            <div class=\"setting-header\">\n");
                html.push_str(&format!(
                    "                <span class=\"setting-key\">{}</span>\n",
                    setting.key
                ));
                html.push_str(&format!(
                    "                <span class=\"setting-type\">{:?}</span>\n",
                    setting.setting_type
                ));
                html.push_str("            </div>\n");

                if !setting.label.is_empty() {
                    html.push_str(&format!(
                        "            <span class=\"setting-label\">{}</span>\n",
                        setting.label
                    ));
                }

                html.push_str(&format!(
                    "            <div class=\"setting-desc\">{}</div>\n",
                    setting.description
                ));

                html.push_str("            <div class=\"setting-details\">\n");
                if let Some(default) = &setting.default {
                    html.push_str(&format!("                <div class=\"detail-item\"><span class=\"detail-label\">Default</span><span>{}</span></div>\n", default));
                }

                if setting.is_required() {
                    html.push_str("                <span class=\"badge badge-required\">Required</span>\n");
                }
                if setting.visibility == Visibility::Secret {
                    html.push_str("                <span class=\"badge badge-secret\">Secret</span>\n");
                }
                html.push_str("            </div>\n");
                html.push_str("        </div>\n");
            }
            html.push_str("    </div>\n");
        }

        html.push_str("</body>\n</html>");
        html
    }

    /// Generate an example TOML configuration with comments and default values.
    pub fn to_example_toml(&self) -> String {
        let mut toml = String::new();
        toml.push_str(&format!("# Example configuration for {}\n", self.name));
        toml.push_str(&format!("# Version: {}\n\n", self.version));

        // For TOML, we should probably group by the first part of the key if it's nested
        // But for a simple example, let's just iterate through settings.

        let mut current_section: Option<String> = None;

        for setting in &self.settings {
            let key_parts: Vec<&str> = setting.key.split('.').collect();

            if key_parts.len() > 1 {
                let section = key_parts[0..key_parts.len() - 1].join(".");
                if current_section.as_deref() != Some(&section) {
                    toml.push_str(&format!("\n[{}]\n", section));
                    current_section = Some(section);
                }
            } else if current_section.is_some() {
                toml.push('\n');
                current_section = None;
            }

            // Description as comment
            for line in setting.description.lines() {
                toml.push_str(&format!("# {}\n", line));
            }

            if setting.is_required() {
                toml.push_str("# (REQUIRED)\n");
            }

            let key = key_parts.last().unwrap();
            let value = if let Some(def) = &setting.default {
                if def.is_string() {
                    format!("\"{}\"", def.as_str().unwrap())
                } else {
                    def.to_string()
                }
            } else {
                match &setting.setting_type {
                    SettingType::Integer { .. } => "0".to_string(),
                    SettingType::Float { .. } => "0.0".to_string(),
                    SettingType::Boolean => "false".to_string(),
                    SettingType::Array { .. } => "[]".to_string(),
                    SettingType::Object { .. } => "{}".to_string(),
                    _ => "\"\"".to_string(),
                }
            };

            toml.push_str(&format!("{} = {}\n\n", key, value));
        }

        toml
    }
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

    #[test]
    fn setting_type_to_json_schema() {
        // String
        let st = SettingType::String {
            pattern: Some(".*".into()),
            min_length: Some(1),
            max_length: Some(10),
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["pattern"], ".*");
        assert_eq!(schema["minLength"], 1);
        assert_eq!(schema["maxLength"], 10);

        // Integer
        let st = SettingType::Integer { min: Some(0), max: Some(100) };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["minimum"], 0);
        assert_eq!(schema["maximum"], 100);

        // Float
        let st = SettingType::Float { min: Some(0.0), max: Some(1.0) };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "number");
        assert_eq!(schema["minimum"], 0.0);
        assert_eq!(schema["maximum"], 1.0);

        // Boolean
        assert_eq!(SettingType::Boolean.to_json_schema()["type"], "boolean");

        // Duration
        assert_eq!(
            SettingType::Duration { min: None, max: None }.to_json_schema()["type"],
            "string"
        );

        // Path
        assert_eq!(
            SettingType::Path { must_exist: false, is_directory: false }.to_json_schema()["type"],
            "string"
        );

        // Url
        let schema = SettingType::Url { schemes: vec!["https".into()] }.to_json_schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["format"], "uri");

        // Enum
        let schema = SettingType::Enum { variants: vec!["A".into(), "B".into()] }.to_json_schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["enum"], json!(["A", "B"]));

        // Secret
        assert_eq!(SettingType::Secret.to_json_schema()["type"], "string");

        // Array
        let st = SettingType::Array {
            element_type: Box::new(SettingType::Integer { min: None, max: None }),
            min_items: Some(1),
            max_items: Some(5),
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "integer");
        assert_eq!(schema["minItems"], 1);
        assert_eq!(schema["maxItems"], 5);

        // Object
        let st = SettingType::Object {
            fields: vec![SettingMetadata {
                key: "nested".into(),
                label: "Nested".into(),
                description: "D".into(),
                setting_type: SettingType::Boolean,
                default: None,
                constraints: vec![Constraint::Required],
                visibility: Visibility::Public,
                group: None,
            }],
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["nested"]["type"], "boolean");
        assert_eq!(schema["required"], json!(["nested"]));

        // Any
        assert_eq!(SettingType::Any.to_json_schema(), json!({}));
    }

    #[test]
    fn setting_metadata_to_json_schema_constraints() {
        let meta = SettingMetadata {
            key: "test".into(),
            label: "Label".into(),
            description: "Desc".into(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: Some(json!("default")),
            constraints: vec![
                Constraint::Pattern("^[a-z]+$".into()),
                Constraint::Range { min: 1.0, max: 10.0 },
                Constraint::Length { min: 2, max: 5 },
                Constraint::OneOf(vec!["a".into(), "b".into()]),
            ],
            visibility: Visibility::Secret,
            group: None,
        };

        let schema = meta.to_json_schema();
        assert_eq!(schema["title"], "Label");
        assert_eq!(schema["description"], "Desc");
        assert_eq!(schema["default"], "default");
        assert_eq!(schema["pattern"], "^[a-z]+$");
        assert_eq!(schema["minimum"], 1.0);
        assert_eq!(schema["maximum"], 10.0);
        assert_eq!(schema["minLength"], 2);
        assert_eq!(schema["maxLength"], 5);
        assert_eq!(schema["enum"], json!(["a", "b"]));
    }

    #[test]
    fn config_schema_to_json_schema_nested() {
        let schema = ConfigSchema {
            name: "app".into(),
            version: "1.0".into(),
            settings: vec![
                SettingMetadata {
                    key: "db.host".into(),
                    label: "Host".into(),
                    description: "D".into(),
                    setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                    default: None,
                    constraints: vec![Constraint::Required],
                    visibility: Visibility::Public,
                    group: None,
                },
                SettingMetadata {
                    key: "db.port".into(),
                    label: "Port".into(),
                    description: "D".into(),
                    setting_type: SettingType::Integer { min: None, max: None },
                    default: Some(json!(5432)),
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
            ],
            groups: vec![],
        };

        let json = schema.to_json_schema();
        assert_eq!(json["title"], "app");
        assert_eq!(json["type"], "object");
        assert_eq!(json["properties"]["db"]["type"], "object");
        assert_eq!(json["properties"]["db"]["properties"]["host"]["type"], "string");
        assert_eq!(json["properties"]["db"]["properties"]["port"]["type"], "integer");
        assert_eq!(json["properties"]["db"]["required"], json!(["host"]));
    }

    #[test]
    fn config_schema_to_html() {
        let schema = ConfigSchema {
            name: "app".into(),
            version: "1.0".into(),
            settings: vec![SettingMetadata {
                key: "key".into(),
                label: "Label".into(),
                description: "Desc".into(),
                setting_type: SettingType::Secret,
                default: Some(json!("val")),
                constraints: vec![Constraint::Required],
                visibility: Visibility::Secret,
                group: Some("Security".into()),
            }],
            groups: vec![SettingGroup {
                name: "Security".into(),
                label: "Security Label".into(),
                description: "Security Desc".into(),
                settings: vec!["key".into()],
            }],
        };

        let html = schema.to_html();
        assert!(html.contains("app Configuration Reference"));
        assert!(html.contains("Security Label"));
        assert!(html.contains("Security Desc"));
        assert!(html.contains("key"));
        assert!(html.contains("badge-secret"));
        assert!(html.contains("badge-required"));
    }

    #[test]
    fn config_schema_to_example_toml() {
        let schema = ConfigSchema {
            name: "app".into(),
            version: "1.0".into(),
            settings: vec![
                SettingMetadata {
                    key: "db.host".into(),
                    label: "Host".into(),
                    description: "Host Desc".into(),
                    setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                    default: Some(json!("localhost")),
                    constraints: vec![Constraint::Required],
                    visibility: Visibility::Public,
                    group: None,
                },
                SettingMetadata {
                    key: "debug".into(),
                    label: "Debug".into(),
                    description: "Debug mode".into(),
                    setting_type: SettingType::Boolean,
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
            ],
            groups: vec![],
        };

        let toml = schema.to_example_toml();
        assert!(toml.contains("[db]"));
        assert!(toml.contains("# Host Desc"));
        assert!(toml.contains("# (REQUIRED)"));
        assert!(toml.contains("host = \"localhost\""));
        assert!(toml.contains("debug = false"));
    }

    // ========================================================================
    // COMPREHENSIVE VISIBILITY TESTS
    // ========================================================================

    #[test]
    fn visibility_all_variants() {
        assert_eq!(Visibility::Public, Visibility::Public);
        assert_eq!(Visibility::Hidden, Visibility::Hidden);
        assert_eq!(Visibility::Secret, Visibility::Secret);
        assert_eq!(Visibility::Advanced, Visibility::Advanced);
    }

    #[test]
    fn visibility_ordering() {
        let mut visibilities = [
            Visibility::Secret,
            Visibility::Public,
            Visibility::Advanced,
            Visibility::Hidden,
        ];
        visibilities.sort_by_key(|v| format!("{:?}", v));
        // Verify they're sortable
        assert_eq!(visibilities.len(), 4);
    }

    #[test]
    fn visibility_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Visibility::Public);
        set.insert(Visibility::Secret);
        set.insert(Visibility::Hidden);
        set.insert(Visibility::Advanced);
        set.insert(Visibility::Public); // Duplicate
        assert_eq!(set.len(), 4);
    }

    // ========================================================================
    // COMPREHENSIVE CONSTRAINT TESTS
    // ========================================================================

    #[test]
    fn constraint_pattern_variant() {
        let c = Constraint::Pattern("[a-z]+".to_string());
        assert_eq!(c, Constraint::Pattern("[a-z]+".to_string()));
        assert_ne!(c, Constraint::Pattern("[A-Z]+".to_string()));
    }

    #[test]
    fn constraint_range_variant() {
        let c = Constraint::Range { min: 0.0, max: 100.0 };
        assert_eq!(c, Constraint::Range { min: 0.0, max: 100.0 });
        assert_ne!(c, Constraint::Range { min: 1.0, max: 100.0 });
    }

    #[test]
    fn constraint_length_variant() {
        let c = Constraint::Length { min: 1, max: 10 };
        assert_eq!(c, Constraint::Length { min: 1, max: 10 });
        assert_ne!(c, Constraint::Length { min: 1, max: 20 });
    }

    #[test]
    fn constraint_required_variant() {
        let c = Constraint::Required;
        assert_eq!(c, Constraint::Required);
    }

    #[test]
    fn constraint_one_of_variant() {
        let c = Constraint::OneOf(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(c, Constraint::OneOf(vec!["a".to_string(), "b".to_string()]));
        assert_ne!(c, Constraint::OneOf(vec!["a".to_string()]));
    }

    #[test]
    fn constraint_custom_variant() {
        let c = Constraint::Custom("my_validator".to_string());
        assert_eq!(c, Constraint::Custom("my_validator".to_string()));
        assert_ne!(c, Constraint::Custom("other_validator".to_string()));
    }

    #[test]
    fn constraint_all_variants_are_distinct() {
        let constraints = [
            Constraint::Pattern(".*".to_string()),
            Constraint::Range { min: 0.0, max: 100.0 },
            Constraint::Length { min: 1, max: 10 },
            Constraint::Required,
            Constraint::OneOf(vec!["a".to_string()]),
            Constraint::Custom("custom".to_string()),
        ];
        // Verify all are distinct
        for i in 0..constraints.len() {
            for j in (i + 1)..constraints.len() {
                assert_ne!(constraints[i], constraints[j]);
            }
        }
    }

    // ========================================================================
    // COMPREHENSIVE SETTING TYPE TESTS - JSON SCHEMA GENERATION
    // ========================================================================

    #[test]
    fn setting_type_string_no_constraints() {
        let st = SettingType::String { pattern: None, min_length: None, max_length: None };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
        assert!(schema.get("pattern").is_none());
        assert!(schema.get("minLength").is_none());
        assert!(schema.get("maxLength").is_none());
    }

    #[test]
    fn setting_type_string_pattern_only() {
        let st = SettingType::String {
            pattern: Some("^[a-z0-9]+$".to_string()),
            min_length: None,
            max_length: None,
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["pattern"], "^[a-z0-9]+$");
    }

    #[test]
    fn setting_type_string_all_constraints() {
        let st = SettingType::String {
            pattern: Some("[0-9]{3}".to_string()),
            min_length: Some(3),
            max_length: Some(10),
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["pattern"], "[0-9]{3}");
        assert_eq!(schema["minLength"], 3);
        assert_eq!(schema["maxLength"], 10);
    }

    #[test]
    fn setting_type_integer_no_constraints() {
        let st = SettingType::Integer { min: None, max: None };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "integer");
        assert!(schema.get("minimum").is_none());
        assert!(schema.get("maximum").is_none());
    }

    #[test]
    fn setting_type_integer_range() {
        let st = SettingType::Integer { min: Some(-100), max: Some(200) };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["minimum"], -100);
        assert_eq!(schema["maximum"], 200);
    }

    #[test]
    fn setting_type_float_no_constraints() {
        let st = SettingType::Float { min: None, max: None };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "number");
        assert!(schema.get("minimum").is_none());
        assert!(schema.get("maximum").is_none());
    }

    #[test]
    fn setting_type_float_range() {
        let st = SettingType::Float { min: Some(0.5), max: Some(99.5) };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "number");
        assert_eq!(schema["minimum"], 0.5);
        assert_eq!(schema["maximum"], 99.5);
    }

    #[test]
    fn setting_type_boolean() {
        let schema = SettingType::Boolean.to_json_schema();
        assert_eq!(schema["type"], "boolean");
        assert_eq!(schema.as_object().unwrap().len(), 1);
    }

    #[test]
    fn setting_type_duration() {
        let st = SettingType::Duration { min: None, max: None };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn setting_type_duration_with_range() {
        let st = SettingType::Duration {
            min: Some(Duration::from_secs(1)),
            max: Some(Duration::from_secs(3600)),
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn setting_type_path_basic() {
        let st = SettingType::Path { must_exist: false, is_directory: false };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn setting_type_path_must_exist() {
        let st = SettingType::Path { must_exist: true, is_directory: false };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn setting_type_path_directory() {
        let st = SettingType::Path { must_exist: true, is_directory: true };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn setting_type_url_no_schemes() {
        let st = SettingType::Url { schemes: vec![] };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["format"], "uri");
    }

    #[test]
    fn setting_type_url_with_schemes() {
        let st = SettingType::Url {
            schemes: vec!["http".to_string(), "https".to_string(), "ftp".to_string()],
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["format"], "uri");
    }

    #[test]
    fn setting_type_enum_single_variant() {
        let st = SettingType::Enum { variants: vec!["only".to_string()] };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["enum"], json!(["only"]));
    }

    #[test]
    fn setting_type_enum_multiple_variants() {
        let st = SettingType::Enum {
            variants: vec!["dev".to_string(), "staging".to_string(), "prod".to_string()],
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["enum"], json!(["dev", "staging", "prod"]));
    }

    #[test]
    fn setting_type_secret() {
        let schema = SettingType::Secret.to_json_schema();
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn setting_type_array_simple() {
        let st = SettingType::Array {
            element_type: Box::new(SettingType::String { pattern: None, min_length: None, max_length: None }),
            min_items: None,
            max_items: None,
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "string");
        assert!(schema.get("minItems").is_none());
        assert!(schema.get("maxItems").is_none());
    }

    #[test]
    fn setting_type_array_with_constraints() {
        let st = SettingType::Array {
            element_type: Box::new(SettingType::Integer { min: None, max: None }),
            min_items: Some(1),
            max_items: Some(100),
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "integer");
        assert_eq!(schema["minItems"], 1);
        assert_eq!(schema["maxItems"], 100);
    }

    #[test]
    fn setting_type_array_nested() {
        let st = SettingType::Array {
            element_type: Box::new(SettingType::Array {
                element_type: Box::new(SettingType::Float { min: None, max: None }),
                min_items: None,
                max_items: None,
            }),
            min_items: None,
            max_items: None,
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "array");
        assert_eq!(schema["items"]["items"]["type"], "number");
    }

    #[test]
    fn setting_type_object_empty() {
        let st = SettingType::Object { fields: vec![] };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"], json!({}));
    }

    #[test]
    fn setting_type_object_single_field() {
        let st = SettingType::Object {
            fields: vec![SettingMetadata {
                key: "field1".to_string(),
                label: "Field 1".to_string(),
                description: "Desc".to_string(),
                setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                default: None,
                constraints: vec![],
                visibility: Visibility::Public,
                group: None,
            }],
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["properties"]["field1"]["type"], "string");
        assert!(schema.get("required").is_none());
    }

    #[test]
    fn setting_type_object_required_field() {
        let st = SettingType::Object {
            fields: vec![SettingMetadata {
                key: "required_field".to_string(),
                label: "Required".to_string(),
                description: "Desc".to_string(),
                setting_type: SettingType::Boolean,
                default: None,
                constraints: vec![Constraint::Required],
                visibility: Visibility::Public,
                group: None,
            }],
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["type"], "object");
        assert_eq!(schema["required"], json!(["required_field"]));
    }

    #[test]
    fn setting_type_object_mixed_required() {
        let st = SettingType::Object {
            fields: vec![
                SettingMetadata {
                    key: "required".to_string(),
                    label: "R".to_string(),
                    description: "D".to_string(),
                    setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                    default: None,
                    constraints: vec![Constraint::Required],
                    visibility: Visibility::Public,
                    group: None,
                },
                SettingMetadata {
                    key: "optional".to_string(),
                    label: "O".to_string(),
                    description: "D".to_string(),
                    setting_type: SettingType::Integer { min: None, max: None },
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
            ],
        };
        let schema = st.to_json_schema();
        assert_eq!(schema["required"], json!(["required"]));
        assert!(schema["properties"].get("optional").is_some());
    }

    #[test]
    fn setting_type_any() {
        let schema = SettingType::Any.to_json_schema();
        assert_eq!(schema, json!({}));
    }

    // ========================================================================
    // COMPREHENSIVE SETTING METADATA TESTS
    // ========================================================================

    #[test]
    fn setting_metadata_is_required_true() {
        let meta = SettingMetadata {
            key: "test".to_string(),
            label: "Test".to_string(),
            description: "Desc".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: None,
            constraints: vec![Constraint::Required],
            visibility: Visibility::Public,
            group: None,
        };
        assert!(meta.is_required());
    }

    #[test]
    fn setting_metadata_is_required_false() {
        let meta = SettingMetadata {
            key: "test".to_string(),
            label: "Test".to_string(),
            description: "Desc".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
        };
        assert!(!meta.is_required());
    }

    #[test]
    fn setting_metadata_is_required_with_multiple_constraints() {
        let meta = SettingMetadata {
            key: "test".to_string(),
            label: "Test".to_string(),
            description: "Desc".to_string(),
            setting_type: SettingType::Integer { min: Some(0), max: Some(100) },
            default: None,
            constraints: vec![
                Constraint::Range { min: 0.0, max: 100.0 },
                Constraint::Required,
                Constraint::Custom("custom".to_string()),
            ],
            visibility: Visibility::Public,
            group: None,
        };
        assert!(meta.is_required());
    }

    #[test]
    fn setting_metadata_to_json_schema_minimal() {
        let meta = SettingMetadata {
            key: "simple".to_string(),
            label: "Simple".to_string(),
            description: "".to_string(),
            setting_type: SettingType::Boolean,
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
        };
        let schema = meta.to_json_schema();
        assert_eq!(schema["type"], "boolean");
        assert_eq!(schema["title"], "Simple");
        assert!(schema.get("description").is_none() || schema["description"] == "");
    }

    #[test]
    fn setting_metadata_to_json_schema_full() {
        let meta = SettingMetadata {
            key: "full".to_string(),
            label: "Full Setting".to_string(),
            description: "A comprehensive setting".to_string(),
            setting_type: SettingType::String {
                pattern: Some("[a-z]+".to_string()),
                min_length: Some(1),
                max_length: Some(50),
            },
            default: Some(json!("default")),
            constraints: vec![Constraint::Required, Constraint::Pattern("[a-z]+".to_string())],
            visibility: Visibility::Secret,
            group: Some("group1".to_string()),
        };
        let schema = meta.to_json_schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["title"], "Full Setting");
        assert_eq!(schema["description"], "A comprehensive setting");
        assert_eq!(schema["default"], "default");
        assert_eq!(schema["minLength"], 1);
        assert_eq!(schema["maxLength"], 50);
        assert_eq!(schema["pattern"], "[a-z]+");
    }

    #[test]
    fn setting_metadata_to_json_schema_range_constraint() {
        let meta = SettingMetadata {
            key: "port".to_string(),
            label: "Port".to_string(),
            description: "Port number".to_string(),
            setting_type: SettingType::Integer { min: None, max: None },
            default: Some(json!(8080)),
            constraints: vec![Constraint::Range { min: 1024.0, max: 65535.0 }],
            visibility: Visibility::Public,
            group: None,
        };
        let schema = meta.to_json_schema();
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["minimum"], 1024.0);
        assert_eq!(schema["maximum"], 65535.0);
        assert_eq!(schema["default"], 8080);
    }

    #[test]
    fn setting_metadata_to_json_schema_length_constraint() {
        let meta = SettingMetadata {
            key: "username".to_string(),
            label: "Username".to_string(),
            description: "User name".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: None,
            constraints: vec![Constraint::Length { min: 3, max: 20 }],
            visibility: Visibility::Public,
            group: None,
        };
        let schema = meta.to_json_schema();
        assert_eq!(schema["minLength"], 3);
        assert_eq!(schema["maxLength"], 20);
    }

    #[test]
    fn setting_metadata_to_json_schema_one_of_constraint() {
        let meta = SettingMetadata {
            key: "env".to_string(),
            label: "Environment".to_string(),
            description: "Deployment environment".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: Some(json!("dev")),
            constraints: vec![Constraint::OneOf(vec![
                "dev".to_string(),
                "staging".to_string(),
                "prod".to_string(),
            ])],
            visibility: Visibility::Public,
            group: None,
        };
        let schema = meta.to_json_schema();
        assert_eq!(schema["enum"], json!(["dev", "staging", "prod"]));
        assert_eq!(schema["default"], "dev");
    }

    #[test]
    fn setting_metadata_clone() {
        let meta1 = SettingMetadata {
            key: "test".to_string(),
            label: "Test".to_string(),
            description: "Desc".to_string(),
            setting_type: SettingType::Boolean,
            default: Some(json!(true)),
            constraints: vec![Constraint::Required],
            visibility: Visibility::Secret,
            group: Some("group".to_string()),
        };
        let meta2 = meta1.clone();
        assert_eq!(meta1, meta2);
    }

    #[test]
    fn setting_metadata_equality() {
        let meta1 = SettingMetadata {
            key: "test".to_string(),
            label: "Test".to_string(),
            description: "Desc".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
        };
        let meta2 = SettingMetadata {
            key: "test".to_string(),
            label: "Test".to_string(),
            description: "Desc".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
        };
        assert_eq!(meta1, meta2);
    }

    #[test]
    fn setting_metadata_inequality() {
        let meta1 = SettingMetadata {
            key: "test1".to_string(),
            label: "Test".to_string(),
            description: "Desc".to_string(),
            setting_type: SettingType::Boolean,
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
        };
        let meta2 = SettingMetadata {
            key: "test2".to_string(),
            label: "Test".to_string(),
            description: "Desc".to_string(),
            setting_type: SettingType::Boolean,
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
        };
        assert_ne!(meta1, meta2);
    }

    // ========================================================================
    // COMPREHENSIVE SETTING GROUP TESTS
    // ========================================================================

    #[test]
    fn setting_group_basic() {
        let group = SettingGroup {
            name: "api".to_string(),
            label: "API Settings".to_string(),
            description: "Configuration for API".to_string(),
            settings: vec!["api_url".to_string(), "api_key".to_string()],
        };
        assert_eq!(group.name, "api");
        assert_eq!(group.label, "API Settings");
        assert_eq!(group.settings.len(), 2);
    }

    #[test]
    fn setting_group_empty() {
        let group = SettingGroup {
            name: "empty".to_string(),
            label: "Empty Group".to_string(),
            description: "No settings".to_string(),
            settings: vec![],
        };
        assert!(group.settings.is_empty());
    }

    #[test]
    fn setting_group_equality() {
        let g1 = SettingGroup {
            name: "api".to_string(),
            label: "API".to_string(),
            description: "API config".to_string(),
            settings: vec!["key1".to_string(), "key2".to_string()],
        };
        let g2 = SettingGroup {
            name: "api".to_string(),
            label: "API".to_string(),
            description: "API config".to_string(),
            settings: vec!["key1".to_string(), "key2".to_string()],
        };
        assert_eq!(g1, g2);
    }

    #[test]
    fn setting_group_clone() {
        let g1 = SettingGroup {
            name: "db".to_string(),
            label: "Database".to_string(),
            description: "DB config".to_string(),
            settings: vec!["host".to_string(), "port".to_string()],
        };
        let g2 = g1.clone();
        assert_eq!(g1, g2);
    }

    // ========================================================================
    // COMPREHENSIVE CONFIG SCHEMA TESTS - JSON SCHEMA GENERATION
    // ========================================================================

    #[test]
    fn config_schema_empty() {
        let schema = ConfigSchema {
            name: "empty-app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![],
            groups: vec![],
        };
        let json = schema.to_json_schema();
        assert_eq!(json["title"], "empty-app");
        assert_eq!(json["type"], "object");
        assert_eq!(json["properties"], json!({}));
    }

    #[test]
    fn config_schema_single_setting() {
        let schema = ConfigSchema {
            name: "single".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![SettingMetadata {
                key: "debug".to_string(),
                label: "Debug Mode".to_string(),
                description: "Enable debug logging".to_string(),
                setting_type: SettingType::Boolean,
                default: Some(json!(false)),
                constraints: vec![],
                visibility: Visibility::Public,
                group: None,
            }],
            groups: vec![],
        };
        let json = schema.to_json_schema();
        assert_eq!(json["properties"]["debug"]["type"], "boolean");
        assert_eq!(json["properties"]["debug"]["default"], false);
    }

    #[test]
    fn config_schema_nested_three_levels() {
        let schema = ConfigSchema {
            name: "nested".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![SettingMetadata {
                key: "db.pool.min_size".to_string(),
                label: "Min Pool Size".to_string(),
                description: "Minimum connection pool size".to_string(),
                setting_type: SettingType::Integer { min: Some(1), max: Some(100) },
                default: Some(json!(5)),
                constraints: vec![],
                visibility: Visibility::Public,
                group: None,
            }],
            groups: vec![],
        };
        let json = schema.to_json_schema();
        assert_eq!(json["properties"]["db"]["type"], "object");
        assert_eq!(json["properties"]["db"]["properties"]["pool"]["type"], "object");
        assert_eq!(
            json["properties"]["db"]["properties"]["pool"]["properties"]["min_size"]["type"],
            "integer"
        );
        assert_eq!(
            json["properties"]["db"]["properties"]["pool"]["properties"]["min_size"]["default"],
            5
        );
    }

    #[test]
    fn config_schema_required_settings() {
        let schema = ConfigSchema {
            name: "required".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![
                SettingMetadata {
                    key: "required_setting".to_string(),
                    label: "Required".to_string(),
                    description: "Must be set".to_string(),
                    setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                    default: None,
                    constraints: vec![Constraint::Required],
                    visibility: Visibility::Public,
                    group: None,
                },
                SettingMetadata {
                    key: "optional_setting".to_string(),
                    label: "Optional".to_string(),
                    description: "May be set".to_string(),
                    setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
            ],
            groups: vec![],
        };
        let json = schema.to_json_schema();
        assert_eq!(json["required"], json!(["required_setting"]));
    }

    #[test]
    fn config_schema_multiple_sections() {
        let schema = ConfigSchema {
            name: "multi".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![
                SettingMetadata {
                    key: "server.port".to_string(),
                    label: "Server Port".to_string(),
                    description: "Port".to_string(),
                    setting_type: SettingType::Integer { min: None, max: None },
                    default: Some(json!(8080)),
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
                SettingMetadata {
                    key: "db.host".to_string(),
                    label: "DB Host".to_string(),
                    description: "Host".to_string(),
                    setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
            ],
            groups: vec![],
        };
        let json = schema.to_json_schema();
        assert_eq!(json["properties"]["server"]["type"], "object");
        assert_eq!(json["properties"]["db"]["type"], "object");
        assert_eq!(json["properties"]["server"]["properties"]["port"]["type"], "integer");
        assert_eq!(json["properties"]["server"]["properties"]["port"]["default"], 8080);
        assert_eq!(json["properties"]["db"]["properties"]["host"]["type"], "string");
    }

    // ========================================================================
    // COMPREHENSIVE CONFIG SCHEMA TESTS - HTML GENERATION
    // ========================================================================

    #[test]
    fn config_schema_to_html_structure() {
        let schema = ConfigSchema {
            name: "html-test".to_string(),
            version: "2.0.0".to_string(),
            settings: vec![],
            groups: vec![],
        };
        let html = schema.to_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("html-test Configuration Reference"));
        assert!(html.contains("2.0.0"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn config_schema_to_html_with_settings() {
        let schema = ConfigSchema {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![SettingMetadata {
                key: "api_key".to_string(),
                label: "API Key".to_string(),
                description: "API key for authentication".to_string(),
                setting_type: SettingType::Secret,
                default: None,
                constraints: vec![Constraint::Required],
                visibility: Visibility::Secret,
                group: None,
            }],
            groups: vec![],
        };
        let html = schema.to_html();
        assert!(html.contains("api_key"));
        assert!(html.contains("API Key"));
        assert!(html.contains("API key for authentication"));
        assert!(html.contains("badge-secret"));
        assert!(html.contains("badge-required"));
    }

    #[test]
    fn config_schema_to_html_with_groups() {
        let schema = ConfigSchema {
            name: "grouped".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![
                SettingMetadata {
                    key: "setting1".to_string(),
                    label: "Setting 1".to_string(),
                    description: "First setting".to_string(),
                    setting_type: SettingType::Boolean,
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: Some("group1".to_string()),
                },
                SettingMetadata {
                    key: "setting2".to_string(),
                    label: "Setting 2".to_string(),
                    description: "Second setting".to_string(),
                    setting_type: SettingType::Integer { min: None, max: None },
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: Some("group2".to_string()),
                },
            ],
            groups: vec![
                SettingGroup {
                    name: "group1".to_string(),
                    label: "Group One".to_string(),
                    description: "First group".to_string(),
                    settings: vec!["setting1".to_string()],
                },
                SettingGroup {
                    name: "group2".to_string(),
                    label: "Group Two".to_string(),
                    description: "Second group".to_string(),
                    settings: vec!["setting2".to_string()],
                },
            ],
        };
        let html = schema.to_html();
        assert!(html.contains("Group One"));
        assert!(html.contains("Group Two"));
        assert!(html.contains("First group"));
        assert!(html.contains("Second group"));
    }

    #[test]
    fn config_schema_to_html_general_group() {
        let schema = ConfigSchema {
            name: "general".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![SettingMetadata {
                key: "setting".to_string(),
                label: "Setting".to_string(),
                description: "A setting".to_string(),
                setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                default: None,
                constraints: vec![],
                visibility: Visibility::Public,
                group: None,
            }],
            groups: vec![],
        };
        let html = schema.to_html();
        assert!(html.contains("General"));
    }

    // ========================================================================
    // COMPREHENSIVE CONFIG SCHEMA TESTS - TOML GENERATION
    // ========================================================================

    #[test]
    fn config_schema_to_toml_header() {
        let schema = ConfigSchema {
            name: "myapp".to_string(),
            version: "3.1.4".to_string(),
            settings: vec![],
            groups: vec![],
        };
        let toml = schema.to_example_toml();
        assert!(toml.contains("# Example configuration for myapp"));
        assert!(toml.contains("# Version: 3.1.4"));
    }

    #[test]
    fn config_schema_to_toml_flat_settings() {
        let schema = ConfigSchema {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![
                SettingMetadata {
                    key: "debug".to_string(),
                    label: "Debug".to_string(),
                    description: "Enable debug".to_string(),
                    setting_type: SettingType::Boolean,
                    default: Some(json!(false)),
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
                SettingMetadata {
                    key: "timeout".to_string(),
                    label: "Timeout".to_string(),
                    description: "Timeout seconds".to_string(),
                    setting_type: SettingType::Integer { min: None, max: None },
                    default: Some(json!(30)),
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
            ],
            groups: vec![],
        };
        let toml = schema.to_example_toml();
        assert!(toml.contains("# Enable debug"));
        assert!(toml.contains("debug = false"));
        assert!(toml.contains("# Timeout seconds"));
        assert!(toml.contains("timeout = 30"));
    }

    #[test]
    fn config_schema_to_toml_nested_sections() {
        let schema = ConfigSchema {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![
                SettingMetadata {
                    key: "db.host".to_string(),
                    label: "Host".to_string(),
                    description: "Database host".to_string(),
                    setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                    default: Some(json!("localhost")),
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
                SettingMetadata {
                    key: "db.port".to_string(),
                    label: "Port".to_string(),
                    description: "Database port".to_string(),
                    setting_type: SettingType::Integer { min: None, max: None },
                    default: Some(json!(5432)),
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
            ],
            groups: vec![],
        };
        let toml = schema.to_example_toml();
        assert!(toml.contains("[db]"));
        assert!(toml.contains("host = \"localhost\""));
        assert!(toml.contains("port = 5432"));
    }

    #[test]
    fn config_schema_to_toml_mixed_sections() {
        let schema = ConfigSchema {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![
                SettingMetadata {
                    key: "server.port".to_string(),
                    label: "Port".to_string(),
                    description: "Server port".to_string(),
                    setting_type: SettingType::Integer { min: None, max: None },
                    default: Some(json!(8080)),
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
                SettingMetadata {
                    key: "debug".to_string(),
                    label: "Debug".to_string(),
                    description: "Debug mode".to_string(),
                    setting_type: SettingType::Boolean,
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
                SettingMetadata {
                    key: "db.host".to_string(),
                    label: "DB Host".to_string(),
                    description: "Database host".to_string(),
                    setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
            ],
            groups: vec![],
        };
        let toml = schema.to_example_toml();
        assert!(toml.contains("[server]"));
        assert!(toml.contains("[db]"));
        assert!(toml.contains("debug = false")); // Top-level setting defaults to false
    }

    #[test]
    fn config_schema_to_toml_required_setting() {
        let schema = ConfigSchema {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![SettingMetadata {
                key: "api_key".to_string(),
                label: "API Key".to_string(),
                description: "Your API key".to_string(),
                setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                default: None,
                constraints: vec![Constraint::Required],
                visibility: Visibility::Public,
                group: None,
            }],
            groups: vec![],
        };
        let toml = schema.to_example_toml();
        assert!(toml.contains("# Your API key"));
        assert!(toml.contains("# (REQUIRED)"));
        assert!(toml.contains("api_key = \"\""));
    }

    #[test]
    fn config_schema_to_toml_type_specific_defaults() {
        let schema = ConfigSchema {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![
                SettingMetadata {
                    key: "count".to_string(),
                    label: "Count".to_string(),
                    description: "Some count".to_string(),
                    setting_type: SettingType::Integer { min: None, max: None },
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
                SettingMetadata {
                    key: "ratio".to_string(),
                    label: "Ratio".to_string(),
                    description: "Some ratio".to_string(),
                    setting_type: SettingType::Float { min: None, max: None },
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
                SettingMetadata {
                    key: "flags".to_string(),
                    label: "Flags".to_string(),
                    description: "Some flags".to_string(),
                    setting_type: SettingType::Array {
                        element_type: Box::new(SettingType::String {
                            pattern: None,
                            min_length: None,
                            max_length: None,
                        }),
                        min_items: None,
                        max_items: None,
                    },
                    default: None,
                    constraints: vec![],
                    visibility: Visibility::Public,
                    group: None,
                },
            ],
            groups: vec![],
        };
        let toml = schema.to_example_toml();
        assert!(toml.contains("count = 0"));
        assert!(toml.contains("ratio = 0.0"));
        assert!(toml.contains("flags = []"));
    }

    #[test]
    fn config_schema_multiline_description() {
        let schema = ConfigSchema {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![SettingMetadata {
                key: "setting".to_string(),
                label: "Setting".to_string(),
                description: "Line 1\nLine 2\nLine 3".to_string(),
                setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
                default: None,
                constraints: vec![],
                visibility: Visibility::Public,
                group: None,
            }],
            groups: vec![],
        };
        let toml = schema.to_example_toml();
        assert!(toml.contains("# Line 1"));
        assert!(toml.contains("# Line 2"));
        assert!(toml.contains("# Line 3"));
    }

    #[test]
    fn config_schema_clone() {
        let schema1 = ConfigSchema {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![],
            groups: vec![],
        };
        let schema2 = schema1.clone();
        assert_eq!(schema1, schema2);
    }

    #[test]
    fn config_schema_equality() {
        let s1 = ConfigSchema {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![],
            groups: vec![],
        };
        let s2 = ConfigSchema {
            name: "app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![],
            groups: vec![],
        };
        assert_eq!(s1, s2);
    }
}
