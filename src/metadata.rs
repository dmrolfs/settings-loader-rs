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
            html.push_str(&format!("    <div class=\"group\">\n        <h2>{}</h2>\n", group_name));
            if let Some(group_info) = self.groups.iter().find(|g| g.name == group_name) {
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
}
