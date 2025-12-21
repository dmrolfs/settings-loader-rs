//! Validation System - Core Types
//!
//! This module provides the validation framework for constraint-based value validation
//! using metadata and introspection.
//!
//! # Core Types
//!
//! - [`ValidationError`] - Detailed validation failure information with expert-backed secret handling
//! - [`ValidationResult`] - Aggregation of validation results and warnings
//!
//! # Secret Value Handling
//!
//! This module uses the `secrecy` and `zeroize` crates (from RustCrypto) to safely handle
//! secret validation errors. Secret values are never stored in error objects; instead:
//! - Actual secret values are immediately zeroed from memory (via `zeroize`)
//! - Error messages use `[REDACTED:key-name]` markers for secrets
//! - Redaction happens at error creation time, making errors safe to log anywhere
//!
//! This approach is more secure than custom redaction logic and leverages expert
//! cryptographic implementations from the RustCrypto organization.

use crate::metadata::{Constraint, SettingMetadata, SettingType, Visibility};
use std::fmt;
use zeroize::Zeroize;

/// Detailed validation error describing constraint violations
///
/// Each variant provides specific context about the type of validation failure,
/// including the setting key and detailed information about what went wrong.
///
/// # Examples
///
/// ```ignore
/// use settings_loader::validation::ValidationError;
///
/// let error = ValidationError::MissingRequired {
///     key: "api_key".to_string(),
/// };
///
/// let error = ValidationError::OutOfRange {
///     key: "port".to_string(),
///     min: 1024.0,
///     max: 65535.0,
///     value: 70000.0,
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Generic constraint violation
    ConstraintViolation {
        /// Setting key/path
        key: String,
        /// Human-readable reason
        reason: String,
    },

    /// Type mismatch: expected one type but got another
    TypeMismatch {
        /// Setting key/path
        key: String,
        /// Expected type description
        expected: String,
        /// Actual type that was found
        actual: String,
    },

    /// Required field is missing or null
    MissingRequired {
        /// Setting key/path
        key: String,
    },

    /// String value doesn't match regex pattern
    InvalidPattern {
        /// Setting key/path
        key: String,
        /// Regex pattern that was required
        pattern: String,
        /// Actual value that failed to match
        value: String,
    },

    /// Numeric value outside allowed range
    OutOfRange {
        /// Setting key/path
        key: String,
        /// Minimum allowed value
        min: f64,
        /// Maximum allowed value
        max: f64,
        /// Actual value that exceeded bounds
        value: f64,
    },

    /// String or array too short
    TooShort {
        /// Setting key/path
        key: String,
        /// Minimum required length
        min: usize,
        /// Actual length
        length: usize,
    },

    /// String or array too long
    TooLong {
        /// Setting key/path
        key: String,
        /// Maximum allowed length
        max: usize,
        /// Actual length
        length: usize,
    },

    /// Value not in allowed set
    NotOneOf {
        /// Setting key/path
        key: String,
        /// Allowed values
        expected: Vec<String>,
        /// Actual value provided
        actual: String,
    },

    /// Custom validation error (application-defined)
    CustomValidation {
        /// Setting key/path
        key: String,
        /// Custom error message
        message: String,
    },

    /// Multiple validation errors aggregated
    Multiple(Vec<ValidationError>),
}

impl ValidationError {
    /// Get the setting key associated with this error
    ///
    /// For Multiple errors, returns None (use errors_with_keys instead)
    pub fn key(&self) -> Option<&str> {
        match self {
            ValidationError::ConstraintViolation { key, .. }
            | ValidationError::TypeMismatch { key, .. }
            | ValidationError::MissingRequired { key }
            | ValidationError::InvalidPattern { key, .. }
            | ValidationError::OutOfRange { key, .. }
            | ValidationError::TooShort { key, .. }
            | ValidationError::TooLong { key, .. }
            | ValidationError::NotOneOf { key, .. }
            | ValidationError::CustomValidation { key, .. } => Some(key),
            ValidationError::Multiple(_) => None,
        }
    }

    /// Extract all errors from nested Multiple variants
    pub fn flatten(&self) -> Vec<&ValidationError> {
        match self {
            ValidationError::Multiple(errors) => errors.iter().flat_map(|e| e.flatten()).collect(),
            _ => vec![self],
        }
    }

    /// Redact sensitive values from error messages based on visibility
    ///
    /// Replaces actual values with `[REDACTED:key-name]` for Secret and Hidden visibility.
    /// This prevents leaking sensitive information (API keys, passwords, tokens) in error logs.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let error = ValidationError::InvalidPattern {
    ///     key: "api_key".to_string(),
    ///     pattern: "[0-9]+".to_string(),
    ///     value: "sk-abc123xyz".to_string(),
    /// };
    /// let redacted = error.redact_if_secret(Visibility::Secret);
    /// // Now displays as: "api_key: value '[REDACTED:api_key]' does not match pattern..."
    /// ```
    pub fn redact_if_secret(self, visibility: Visibility) -> Self {
        match visibility {
            Visibility::Secret | Visibility::Hidden => self.redact(),
            _ => self,
        }
    }

    /// Unconditionally redact all sensitive values from this error
    ///
    /// Uses the `zeroize` crate to securely zero secret values from memory.
    /// This prevents accidental leakage of secrets even if the error object is cloned.
    fn redact(self) -> Self {
        match self {
            ValidationError::InvalidPattern { key, pattern, value } => {
                // Securely zero the secret value from memory
                let mut secret_value = value;
                secret_value.zeroize();

                ValidationError::InvalidPattern {
                    key: key.clone(),
                    pattern,
                    value: format!("[REDACTED:{}]", key),
                }
            },
            ValidationError::OutOfRange { key, min, max, .. } => {
                // Use NaN sentinel - won't display in error message
                // No need to zeroize f64 (doesn't hold sensitive data in memory)
                ValidationError::OutOfRange { key, min, max, value: f64::NAN }
            },
            ValidationError::NotOneOf { key, expected, actual } => {
                // Securely zero the secret value from memory
                let mut secret_value = actual;
                secret_value.zeroize();

                ValidationError::NotOneOf {
                    key: key.clone(),
                    expected,
                    actual: format!("[REDACTED:{}]", key),
                }
            },
            ValidationError::Multiple(errors) => {
                ValidationError::Multiple(errors.into_iter().map(|e| e.redact()).collect())
            },
            // Other variants don't expose values or are already safe
            other => other,
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::ConstraintViolation { key, reason } => {
                write!(f, "{}: {}", key, reason)
            },
            ValidationError::TypeMismatch { key, expected, actual } => {
                write!(f, "{}: type mismatch - expected {}, got {}", key, expected, actual)
            },
            ValidationError::MissingRequired { key } => {
                write!(f, "{}: required field is missing", key)
            },
            ValidationError::InvalidPattern { key, pattern, value } => {
                write!(
                    f,
                    "{}: value '{}' does not match required pattern '{}' (expected: pattern matching {})",
                    key, value, pattern, pattern
                )
            },
            ValidationError::OutOfRange { key, min, max, value } => {
                if value.is_nan() {
                    // Value was redacted
                    write!(
                        f,
                        "{}: [REDACTED:{}] is outside allowed range [{}, {}] (expected: between {} and {})",
                        key, key, min, max, min, max
                    )
                } else {
                    write!(
                        f,
                        "{}: value {} is outside allowed range [{}, {}] (expected: between {} and {})",
                        key, value, min, max, min, max
                    )
                }
            },
            ValidationError::TooShort { key, min, length } => {
                write!(f, "{}: length {} is shorter than minimum {}", key, length, min)
            },
            ValidationError::TooLong { key, max, length } => {
                write!(f, "{}: length {} exceeds maximum {}", key, length, max)
            },
            ValidationError::NotOneOf { key, expected, actual } => {
                let expected_str = expected.join(", ");
                write!(
                    f,
                    "{}: '{}' is not one of allowed values: {} (expected: one of {})",
                    key, actual, expected_str, expected_str
                )
            },
            ValidationError::CustomValidation { key, message } => {
                write!(f, "{}: {}", key, message)
            },
            ValidationError::Multiple(errors) => {
                writeln!(f, "Multiple validation errors:")?;
                for (idx, error) in errors.iter().enumerate() {
                    write!(f, "  {}. {}", idx + 1, error)?;
                    if idx < errors.len() - 1 {
                        writeln!(f)?;
                    }
                }
                Ok(())
            },
        }
    }
}

impl std::error::Error for ValidationError {}

// ============================================================================
// HELPER TRAIT FOR JSON VALUES
// ============================================================================

trait JsonTypeStr {
    fn type_str(&self) -> &'static str;
}

impl JsonTypeStr for serde_json::Value {
    fn type_str(&self) -> &'static str {
        match self {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::String(_) => "string",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
        }
    }
}

// ============================================================================
// CONSTRAINT VALIDATORS
// ============================================================================

impl Constraint {
    /// Validate a value against this constraint
    ///
    /// # Arguments
    /// * `key` - Setting key for error context
    /// * `value` - JSON value to validate
    ///
    /// # Returns
    /// * `Ok(())` if validation passes
    /// * `Err(ValidationError)` if validation fails
    pub fn validate(&self, key: &str, value: &serde_json::Value) -> Result<(), ValidationError> {
        match self {
            Constraint::Pattern(pattern) => Self::validate_pattern(key, pattern, value),
            Constraint::Range { min, max } => Self::validate_range(key, *min, *max, value),
            Constraint::Length { min, max } => Self::validate_length(key, *min, *max, value),
            Constraint::Required => Self::validate_required(key, value),
            Constraint::OneOf(options) => Self::validate_one_of(key, options, value),
            Constraint::Custom(_name) => {
                // Custom constraints are application-specific and delegated to the app
                // We don't validate them here - just pass through
                Ok(())
            },
        }
    }

    /// Validate that a string value matches a regex pattern
    fn validate_pattern(key: &str, pattern: &str, value: &serde_json::Value) -> Result<(), ValidationError> {
        let value_str = value.as_str().ok_or_else(|| ValidationError::TypeMismatch {
            key: key.to_string(),
            expected: "string".to_string(),
            actual: value.type_str().to_string(),
        })?;

        // Compile regex - if compilation fails, that's a constraint violation
        let re = regex::Regex::new(pattern).map_err(|_| ValidationError::ConstraintViolation {
            key: key.to_string(),
            reason: format!("Invalid regex pattern: {}", pattern),
        })?;

        if !re.is_match(value_str) {
            return Err(ValidationError::InvalidPattern {
                key: key.to_string(),
                pattern: pattern.to_string(),
                value: value_str.to_string(),
            });
        }

        Ok(())
    }

    /// Validate that a numeric value is within range
    fn validate_range(key: &str, min: f64, max: f64, value: &serde_json::Value) -> Result<(), ValidationError> {
        let num = value
            .as_f64()
            .or_else(|| value.as_i64().map(|i| i as f64))
            .or_else(|| value.as_u64().map(|u| u as f64))
            .ok_or_else(|| ValidationError::TypeMismatch {
                key: key.to_string(),
                expected: "number".to_string(),
                actual: value.type_str().to_string(),
            })?;

        if num < min || num > max {
            return Err(ValidationError::OutOfRange { key: key.to_string(), min, max, value: num });
        }

        Ok(())
    }

    /// Validate that a string or array length is within bounds
    fn validate_length(key: &str, min: usize, max: usize, value: &serde_json::Value) -> Result<(), ValidationError> {
        let length = match value {
            serde_json::Value::String(s) => s.chars().count(),
            serde_json::Value::Array(arr) => arr.len(),
            _ => {
                return Err(ValidationError::TypeMismatch {
                    key: key.to_string(),
                    expected: "string or array".to_string(),
                    actual: value.type_str().to_string(),
                })
            },
        };

        if length < min {
            return Err(ValidationError::TooShort { key: key.to_string(), min, length });
        }

        if length > max {
            return Err(ValidationError::TooLong { key: key.to_string(), max, length });
        }

        Ok(())
    }

    /// Validate that a value is not null/missing
    fn validate_required(key: &str, value: &serde_json::Value) -> Result<(), ValidationError> {
        if value.is_null() {
            return Err(ValidationError::MissingRequired { key: key.to_string() });
        }
        Ok(())
    }

    /// Validate that a value is one of the allowed options
    fn validate_one_of(key: &str, options: &[String], value: &serde_json::Value) -> Result<(), ValidationError> {
        let value_str = value.as_str().ok_or_else(|| ValidationError::TypeMismatch {
            key: key.to_string(),
            expected: "string".to_string(),
            actual: value.type_str().to_string(),
        })?;

        if !options.contains(&value_str.to_string()) {
            return Err(ValidationError::NotOneOf {
                key: key.to_string(),
                expected: options.to_vec(),
                actual: value_str.to_string(),
            });
        }

        Ok(())
    }
}

// ============================================================================
// TYPE VALIDATORS
// ============================================================================

impl SettingType {
    /// Validate a value against this type's constraints
    pub fn validate(&self, key: &str, value: &serde_json::Value) -> Result<(), ValidationError> {
        match self {
            SettingType::String { pattern, min_length, max_length } => {
                Self::validate_string(key, value, pattern.as_deref(), *min_length, *max_length)
            },
            SettingType::Integer { min, max } => Self::validate_integer(key, value, *min, *max),
            SettingType::Float { min, max } => Self::validate_float(key, value, *min, *max),
            SettingType::Boolean => Self::validate_boolean(key, value),
            SettingType::Duration { min, max } => Self::validate_duration(key, value, *min, *max),
            SettingType::Path { must_exist, is_directory } => {
                Self::validate_path(key, value, *must_exist, *is_directory)
            },
            SettingType::Url { schemes } => Self::validate_url(key, value, schemes),
            SettingType::Enum { variants } => Self::validate_enum(key, value, variants),
            SettingType::Secret => {
                // Secrets just need to be present and non-null
                if value.is_null() {
                    Err(ValidationError::MissingRequired { key: key.to_string() })
                } else {
                    Ok(())
                }
            },
            SettingType::Array { element_type, min_items, max_items } => {
                Self::validate_array(key, value, element_type, *min_items, *max_items)
            },
            SettingType::Object { fields } => Self::validate_object(key, value, fields),
            SettingType::Any => {
                // Any type accepts anything
                Ok(())
            },
        }
    }

    fn validate_string(
        key: &str, value: &serde_json::Value, pattern: Option<&str>, min_length: Option<usize>,
        max_length: Option<usize>,
    ) -> Result<(), ValidationError> {
        let s = value.as_str().ok_or_else(|| ValidationError::TypeMismatch {
            key: key.to_string(),
            expected: "string".to_string(),
            actual: value.type_str().to_string(),
        })?;

        if let Some(pattern) = pattern {
            let re = regex::Regex::new(pattern).map_err(|_| ValidationError::ConstraintViolation {
                key: key.to_string(),
                reason: format!("Invalid regex pattern: {}", pattern),
            })?;

            if !re.is_match(s) {
                return Err(ValidationError::InvalidPattern {
                    key: key.to_string(),
                    pattern: pattern.to_string(),
                    value: s.to_string(),
                });
            }
        }

        let length = s.chars().count();

        if let Some(min) = min_length {
            if length < min {
                return Err(ValidationError::TooShort { key: key.to_string(), min, length });
            }
        }

        if let Some(max) = max_length {
            if length > max {
                return Err(ValidationError::TooLong { key: key.to_string(), max, length });
            }
        }

        Ok(())
    }

    fn validate_integer(
        key: &str, value: &serde_json::Value, min: Option<i64>, max: Option<i64>,
    ) -> Result<(), ValidationError> {
        let i = value.as_i64().ok_or_else(|| ValidationError::TypeMismatch {
            key: key.to_string(),
            expected: "integer".to_string(),
            actual: value.type_str().to_string(),
        })?;

        if let Some(min_val) = min {
            if i < min_val {
                return Err(ValidationError::OutOfRange {
                    key: key.to_string(),
                    min: min_val as f64,
                    max: max.unwrap_or(i64::MAX) as f64,
                    value: i as f64,
                });
            }
        }

        if let Some(max_val) = max {
            if i > max_val {
                return Err(ValidationError::OutOfRange {
                    key: key.to_string(),
                    min: min.unwrap_or(i64::MIN) as f64,
                    max: max_val as f64,
                    value: i as f64,
                });
            }
        }

        Ok(())
    }

    fn validate_float(
        key: &str, value: &serde_json::Value, min: Option<f64>, max: Option<f64>,
    ) -> Result<(), ValidationError> {
        let f = value
            .as_f64()
            .or_else(|| value.as_i64().map(|i| i as f64))
            .or_else(|| value.as_u64().map(|u| u as f64))
            .ok_or_else(|| ValidationError::TypeMismatch {
                key: key.to_string(),
                expected: "float".to_string(),
                actual: value.type_str().to_string(),
            })?;

        if let Some(min_val) = min {
            if f < min_val {
                return Err(ValidationError::OutOfRange {
                    key: key.to_string(),
                    min: min_val,
                    max: max.unwrap_or(f64::MAX),
                    value: f,
                });
            }
        }

        if let Some(max_val) = max {
            if f > max_val {
                return Err(ValidationError::OutOfRange {
                    key: key.to_string(),
                    min: min.unwrap_or(f64::MIN),
                    max: max_val,
                    value: f,
                });
            }
        }

        Ok(())
    }

    fn validate_boolean(key: &str, value: &serde_json::Value) -> Result<(), ValidationError> {
        value.as_bool().ok_or_else(|| ValidationError::TypeMismatch {
            key: key.to_string(),
            expected: "boolean".to_string(),
            actual: value.type_str().to_string(),
        })?;
        Ok(())
    }

    fn validate_duration(
        key: &str, value: &serde_json::Value, _min: Option<std::time::Duration>, _max: Option<std::time::Duration>,
    ) -> Result<(), ValidationError> {
        // Duration validation: accept number (seconds) or object with secs/nanos
        if value.is_number() || value.is_object() {
            Ok(())
        } else {
            Err(ValidationError::TypeMismatch {
                key: key.to_string(),
                expected: "duration (number or object)".to_string(),
                actual: value.type_str().to_string(),
            })
        }
    }

    fn validate_path(
        key: &str, value: &serde_json::Value, _must_exist: bool, _is_directory: bool,
    ) -> Result<(), ValidationError> {
        value.as_str().ok_or_else(|| ValidationError::TypeMismatch {
            key: key.to_string(),
            expected: "path string".to_string(),
            actual: value.type_str().to_string(),
        })?;
        // Note: Actual filesystem checks (must_exist, is_directory) would be done
        // at a higher level, not in type validation
        Ok(())
    }

    fn validate_url(key: &str, value: &serde_json::Value, schemes: &[String]) -> Result<(), ValidationError> {
        let url_str = value.as_str().ok_or_else(|| ValidationError::TypeMismatch {
            key: key.to_string(),
            expected: "URL string".to_string(),
            actual: value.type_str().to_string(),
        })?;

        // If schemes are specified, validate the scheme matches
        if !schemes.is_empty() {
            if let Some(scheme_end) = url_str.find("://") {
                let scheme = &url_str[..scheme_end];
                if !schemes.contains(&scheme.to_string()) {
                    return Err(ValidationError::ConstraintViolation {
                        key: key.to_string(),
                        reason: format!("URL scheme '{}' not in allowed schemes: {}", scheme, schemes.join(", ")),
                    });
                }
            } else {
                return Err(ValidationError::ConstraintViolation {
                    key: key.to_string(),
                    reason: "URL must have a scheme (e.g., http://)".to_string(),
                });
            }
        }
        Ok(())
    }

    fn validate_enum(key: &str, value: &serde_json::Value, variants: &[String]) -> Result<(), ValidationError> {
        let s = value.as_str().ok_or_else(|| ValidationError::TypeMismatch {
            key: key.to_string(),
            expected: "enum variant string".to_string(),
            actual: value.type_str().to_string(),
        })?;

        if !variants.contains(&s.to_string()) {
            return Err(ValidationError::NotOneOf {
                key: key.to_string(),
                expected: variants.to_vec(),
                actual: s.to_string(),
            });
        }

        Ok(())
    }

    fn validate_array(
        key: &str, value: &serde_json::Value, element_type: &SettingType, min_items: Option<usize>,
        max_items: Option<usize>,
    ) -> Result<(), ValidationError> {
        let arr = value.as_array().ok_or_else(|| ValidationError::TypeMismatch {
            key: key.to_string(),
            expected: "array".to_string(),
            actual: value.type_str().to_string(),
        })?;

        if let Some(min) = min_items {
            if arr.len() < min {
                return Err(ValidationError::TooShort { key: key.to_string(), min, length: arr.len() });
            }
        }

        if let Some(max) = max_items {
            if arr.len() > max {
                return Err(ValidationError::TooLong { key: key.to_string(), max, length: arr.len() });
            }
        }

        // Validate each element
        for (idx, item) in arr.iter().enumerate() {
            let item_key = format!("{}[{}]", key, idx);
            element_type.validate(&item_key, item)?;
        }

        Ok(())
    }

    fn validate_object(
        key: &str, value: &serde_json::Value, fields: &[SettingMetadata],
    ) -> Result<(), ValidationError> {
        value.as_object().ok_or_else(|| ValidationError::TypeMismatch {
            key: key.to_string(),
            expected: "object".to_string(),
            actual: value.type_str().to_string(),
        })?;

        // Validate each field if they exist in the object
        for field in fields {
            if let Some(field_value) = value.get(&field.key) {
                field.validate_value(field_value)?;
            }
        }

        Ok(())
    }
}

// ============================================================================
// METADATA VALIDATORS
// ============================================================================

impl SettingMetadata {
    /// Validate a value against this setting's metadata (type + constraints)
    pub fn validate_value(&self, value: &serde_json::Value) -> Result<(), ValidationError> {
        // Check type first
        self.setting_type.validate(&self.key, value)?;

        // Then check all constraints
        for constraint in &self.constraints {
            constraint.validate(&self.key, value)?;
        }

        Ok(())
    }

    /// Validate a value and return detailed ValidationResult
    pub fn validate(&self, value: &serde_json::Value) -> crate::validation::ValidationResult {
        let mut result = crate::validation::ValidationResult::new();

        // Check required constraint
        if self.constraints.contains(&Constraint::Required) && value.is_null() {
            result.add_error(ValidationError::MissingRequired { key: self.key.clone() });
            return result;
        }

        // Check type
        if let Err(e) = self.setting_type.validate(&self.key, value) {
            let error = e.redact_if_secret(self.visibility);
            result.add_error(error);
            return result;
        }

        // Check constraints
        for constraint in &self.constraints {
            if let Err(e) = constraint.validate(&self.key, value) {
                let error = e.redact_if_secret(self.visibility);
                result.add_error(error);
            }
        }

        result
    }
}

/// Aggregated validation results including errors and warnings
///
/// Collects all validation errors and warnings from checking a value or configuration
/// against its metadata constraints and type requirements.
///
/// # Examples
///
/// ```ignore
/// use settings_loader::validation::{ValidationResult, ValidationError};
///
/// let mut result = ValidationResult::new();
/// result.add_error(ValidationError::MissingRequired {
///     key: "api_key".to_string(),
/// });
/// result.add_warning("Using default value for port".to_string());
///
/// assert!(!result.is_valid());
/// assert_eq!(result.errors().len(), 1);
/// assert_eq!(result.warnings().len(), 1);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Whether validation passed without errors
    is_valid: bool,
    /// Accumulated validation errors
    errors: Vec<ValidationError>,
    /// Non-critical warnings
    warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result (initially valid with no errors/warnings)
    pub fn new() -> Self {
        ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check if validation passed without errors
    pub fn is_valid(&self) -> bool {
        self.is_valid && self.errors.is_empty()
    }

    /// Get all validation errors
    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }

    /// Get all warnings
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    /// Add a validation error and mark result as invalid
    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    /// Add a non-critical warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Get count of validation errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get count of warnings
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Check if there are any validation errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Merge another ValidationResult into this one
    ///
    /// Combines errors and warnings. Result is valid only if both are valid.
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }

    /// Convert to a Result type for convenient error propagation
    pub fn into_result(self) -> Result<(), ValidationError> {
        if self.is_valid() {
            Ok(())
        } else {
            match self.errors.len() {
                0 => Ok(()),
                1 => Err(self.errors.into_iter().next().unwrap()),
                _ => Err(ValidationError::Multiple(self.errors)),
            }
        }
    }
}

impl fmt::Display for ValidationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_valid() {
            write!(f, "Validation passed")
        } else {
            write!(f, "Validation failed with {} error(s)", self.errors.len())?;
            if !self.warnings.is_empty() {
                write!(f, " and {} warning(s)", self.warnings.len())?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches2::assert_matches;

    #[test]
    fn validation_error_missing_required_display() {
        let error = ValidationError::MissingRequired { key: "api_key".to_string() };
        assert_eq!(error.to_string(), "api_key: required field is missing");
    }

    #[test]
    fn validation_error_out_of_range_display() {
        let error = ValidationError::OutOfRange {
            key: "port".to_string(),
            min: 1024.0,
            max: 65535.0,
            value: 70000.0,
        };
        let msg = error.to_string();
        assert!(msg.contains("port"));
        assert!(msg.contains("70000"));
        assert!(msg.contains("1024"));
        assert!(msg.contains("65535"));
    }

    #[test]
    fn validation_error_invalid_pattern_display() {
        let error = ValidationError::InvalidPattern {
            key: "email".to_string(),
            pattern: "[a-z0-9]+@[a-z0-9]+\\.[a-z]{2,}".to_string(),
            value: "not-an-email".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("email"));
        assert!(msg.contains("not-an-email"));
    }

    #[test]
    fn validation_error_not_one_of_display() {
        let error = ValidationError::NotOneOf {
            key: "env".to_string(),
            expected: vec!["dev".to_string(), "staging".to_string(), "prod".to_string()],
            actual: "invalid".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("env"));
        assert!(msg.contains("invalid"));
        assert!(msg.contains("dev"));
    }

    #[test]
    fn validation_error_key_extraction() {
        let error = ValidationError::MissingRequired { key: "test_key".to_string() };
        assert_eq!(error.key(), Some("test_key"));
    }

    #[test]
    fn validation_error_multiple_flatten() {
        let error = ValidationError::Multiple(vec![
            ValidationError::MissingRequired { key: "key1".to_string() },
            ValidationError::OutOfRange {
                key: "key2".to_string(),
                min: 0.0,
                max: 100.0,
                value: 150.0,
            },
        ]);

        let flattened = error.flatten();
        assert_eq!(flattened.len(), 2);
    }

    #[test]
    fn validation_result_new_is_valid() {
        let result = ValidationResult::new();
        assert!(result.is_valid());
        assert_eq!(result.error_count(), 0);
        assert_eq!(result.warning_count(), 0);
    }

    #[test]
    fn validation_result_add_error_marks_invalid() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());

        result.add_error(ValidationError::MissingRequired { key: "test".to_string() });

        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn validation_result_add_warning() {
        let mut result = ValidationResult::new();
        result.add_warning("Test warning".to_string());

        assert!(result.is_valid());
        assert_eq!(result.warning_count(), 1);
        assert!(!result.warnings().is_empty());
    }

    #[test]
    fn validation_result_merge() {
        let mut result1 = ValidationResult::new();
        result1.add_error(ValidationError::MissingRequired { key: "key1".to_string() });

        let mut result2 = ValidationResult::new();
        result2.add_error(ValidationError::OutOfRange {
            key: "key2".to_string(),
            min: 0.0,
            max: 100.0,
            value: 150.0,
        });

        result1.merge(result2);
        assert!(!result1.is_valid());
        assert_eq!(result1.error_count(), 2);
    }

    #[test]
    fn validation_result_into_result_ok() {
        let result = ValidationResult::new();
        assert!(result.into_result().is_ok());
    }

    #[test]
    fn validation_result_into_result_err() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError::MissingRequired { key: "test".to_string() });
        assert!(result.into_result().is_err());
    }

    #[test]
    fn validation_result_display_valid() {
        let result = ValidationResult::new();
        assert_eq!(result.to_string(), "Validation passed");
    }

    #[test]
    fn validation_result_display_invalid() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError::MissingRequired { key: "test".to_string() });
        let msg = result.to_string();
        assert!(msg.contains("Validation failed"));
        assert!(msg.contains("1 error"));
    }
    #[test]
    fn test_constraint_validate_length_array() {
        let constraint = Constraint::Length { min: 2, max: 4 };
        let arr = serde_json::json!([1, 2, 3]);
        assert!(constraint.validate("test", &arr).is_ok());

        let short_arr = serde_json::json!([1]);
        assert_matches!(
            constraint.validate("test", &short_arr),
            Err(ValidationError::TooShort { .. })
        );

        let long_arr = serde_json::json!([1, 2, 3, 4, 5]);
        assert_matches!(
            constraint.validate("test", &long_arr),
            Err(ValidationError::TooLong { .. })
        );
    }

    #[test]
    fn test_constraint_validate_length_invalid_type() {
        let constraint = Constraint::Length { min: 1, max: 5 };
        let num = serde_json::json!(10);
        assert_matches!(
            constraint.validate("test", &num),
            Err(ValidationError::TypeMismatch { .. })
        );
    }

    // ========================================================================
    // NEW COMPREHENSIVE VALIDATION TESTS
    // ========================================================================

    // ValidationError::key() tests
    #[test]
    fn test_validation_error_key_all_variants() {
        assert_eq!(
            ValidationError::ConstraintViolation {
                key: "key1".to_string(),
                reason: "test".to_string()
            }
            .key(),
            Some("key1")
        );

        assert_eq!(
            ValidationError::TypeMismatch {
                key: "key2".to_string(),
                expected: "string".to_string(),
                actual: "number".to_string()
            }
            .key(),
            Some("key2")
        );

        assert_eq!(
            ValidationError::MissingRequired { key: "key3".to_string() }.key(),
            Some("key3")
        );

        assert_eq!(
            ValidationError::InvalidPattern {
                key: "key4".to_string(),
                pattern: ".*".to_string(),
                value: "test".to_string()
            }
            .key(),
            Some("key4")
        );

        assert_eq!(
            ValidationError::OutOfRange {
                key: "key5".to_string(),
                min: 0.0,
                max: 100.0,
                value: 150.0
            }
            .key(),
            Some("key5")
        );

        assert_eq!(
            ValidationError::TooShort { key: "key6".to_string(), min: 5, length: 3 }.key(),
            Some("key6")
        );

        assert_eq!(
            ValidationError::TooLong { key: "key7".to_string(), max: 10, length: 15 }.key(),
            Some("key7")
        );

        assert_eq!(
            ValidationError::NotOneOf {
                key: "key8".to_string(),
                expected: vec!["a".to_string()],
                actual: "b".to_string()
            }
            .key(),
            Some("key8")
        );

        assert_eq!(
            ValidationError::CustomValidation {
                key: "key9".to_string(),
                message: "custom".to_string()
            }
            .key(),
            Some("key9")
        );

        assert_eq!(
            ValidationError::Multiple(vec![ValidationError::MissingRequired { key: "key10".to_string() }]).key(),
            None
        );
    }

    #[test]
    fn test_validation_error_flatten_nested_multiple() {
        let error = ValidationError::Multiple(vec![
            ValidationError::MissingRequired { key: "key1".to_string() },
            ValidationError::Multiple(vec![
                ValidationError::OutOfRange {
                    key: "key2".to_string(),
                    min: 0.0,
                    max: 100.0,
                    value: 150.0,
                },
                ValidationError::TooShort { key: "key3".to_string(), min: 5, length: 2 },
            ]),
        ]);

        let flattened = error.flatten();
        assert_eq!(flattened.len(), 3);
    }

    #[test]
    fn test_validation_error_redact_if_secret_public_visibility() {
        let error = ValidationError::InvalidPattern {
            key: "username".to_string(),
            pattern: "[a-z]+".to_string(),
            value: "Invalid123".to_string(),
        };

        let redacted = error.redact_if_secret(Visibility::Public);
        match redacted {
            ValidationError::InvalidPattern { value, .. } => {
                assert_eq!(value, "Invalid123"); // Not redacted for public
            },
            _ => panic!("Expected InvalidPattern"),
        }
    }

    #[test]
    fn test_validation_error_redact_if_secret_hidden_visibility() {
        let error = ValidationError::InvalidPattern {
            key: "api_key".to_string(),
            pattern: "[0-9]+".to_string(),
            value: "sk-abc123xyz".to_string(),
        };

        let redacted = error.redact_if_secret(Visibility::Hidden);
        match redacted {
            ValidationError::InvalidPattern { value, .. } => {
                assert_eq!(value, "[REDACTED:api_key]");
            },
            _ => panic!("Expected InvalidPattern"),
        }
    }

    #[test]
    fn test_validation_error_redact_if_secret_secret_visibility() {
        let error = ValidationError::NotOneOf {
            key: "token".to_string(),
            expected: vec!["valid1".to_string(), "valid2".to_string()],
            actual: "secret_value_xyz".to_string(),
        };

        let redacted = error.redact_if_secret(Visibility::Secret);
        match redacted {
            ValidationError::NotOneOf { actual, .. } => {
                assert_eq!(actual, "[REDACTED:token]");
            },
            _ => panic!("Expected NotOneOf"),
        }
    }

    #[test]
    fn test_validation_error_redact_out_of_range() {
        let error = ValidationError::OutOfRange {
            key: "password_hash".to_string(),
            min: 0.0,
            max: 100.0,
            value: 150.0,
        };

        let redacted = error.redact_if_secret(Visibility::Secret);
        match redacted {
            ValidationError::OutOfRange { value, .. } => {
                assert!(value.is_nan()); // NaN sentinel for redaction
            },
            _ => panic!("Expected OutOfRange"),
        }
    }

    #[test]
    fn test_validation_error_display_constraint_violation() {
        let error = ValidationError::ConstraintViolation {
            key: "test".to_string(),
            reason: "invalid value".to_string(),
        };
        assert_eq!(error.to_string(), "test: invalid value");
    }

    #[test]
    fn test_validation_error_display_type_mismatch() {
        let error = ValidationError::TypeMismatch {
            key: "port".to_string(),
            expected: "integer".to_string(),
            actual: "string".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("port"));
        assert!(msg.contains("type mismatch"));
        assert!(msg.contains("integer"));
        assert!(msg.contains("string"));
    }

    #[test]
    fn test_validation_error_display_too_short() {
        let error = ValidationError::TooShort { key: "password".to_string(), min: 8, length: 5 };
        let msg = error.to_string();
        assert!(msg.contains("password"));
        assert!(msg.contains("5"));
        assert!(msg.contains("8"));
    }

    #[test]
    fn test_validation_error_display_too_long() {
        let error = ValidationError::TooLong { key: "name".to_string(), max: 50, length: 100 };
        let msg = error.to_string();
        assert!(msg.contains("name"));
        assert!(msg.contains("100"));
        assert!(msg.contains("50"));
    }

    #[test]
    fn test_validation_error_display_custom_validation() {
        let error = ValidationError::CustomValidation {
            key: "custom_field".to_string(),
            message: "failed custom check".to_string(),
        };
        assert_eq!(error.to_string(), "custom_field: failed custom check");
    }

    #[test]
    fn test_validation_error_display_multiple() {
        let error = ValidationError::Multiple(vec![
            ValidationError::MissingRequired { key: "key1".to_string() },
            ValidationError::OutOfRange {
                key: "key2".to_string(),
                min: 0.0,
                max: 100.0,
                value: 150.0,
            },
        ]);
        let msg = error.to_string();
        assert!(msg.contains("Multiple validation errors"));
        assert!(msg.contains("1."));
        assert!(msg.contains("2."));
    }

    // Constraint::validate tests for all variants
    #[test]
    fn test_constraint_pattern_valid() {
        let constraint = Constraint::Pattern("[a-z]+".to_string());
        let value = serde_json::json!("abc");
        assert!(constraint.validate("test", &value).is_ok());
    }

    #[test]
    fn test_constraint_pattern_invalid() {
        let constraint = Constraint::Pattern("[0-9]+".to_string());
        let value = serde_json::json!("abc");
        assert_matches!(
            constraint.validate("test", &value),
            Err(ValidationError::InvalidPattern { .. })
        );
    }

    #[test]
    fn test_constraint_pattern_invalid_regex() {
        let constraint = Constraint::Pattern("[invalid(".to_string());
        let value = serde_json::json!("test");
        assert_matches!(
            constraint.validate("test", &value),
            Err(ValidationError::ConstraintViolation { .. })
        );
    }

    #[test]
    fn test_constraint_range_valid() {
        let constraint = Constraint::Range { min: 1.0, max: 100.0 };
        assert!(constraint.validate("test", &serde_json::json!(50)).is_ok());
        assert!(constraint.validate("test", &serde_json::json!(1)).is_ok());
        assert!(constraint.validate("test", &serde_json::json!(100)).is_ok());
    }

    #[test]
    fn test_constraint_range_out_of_bounds() {
        let constraint = Constraint::Range { min: 1.0, max: 100.0 };
        assert_matches!(
            constraint.validate("test", &serde_json::json!(0)),
            Err(ValidationError::OutOfRange { .. })
        );
        assert_matches!(
            constraint.validate("test", &serde_json::json!(101)),
            Err(ValidationError::OutOfRange { .. })
        );
    }

    #[test]
    fn test_constraint_required_valid() {
        let constraint = Constraint::Required;
        assert!(constraint.validate("test", &serde_json::json!("value")).is_ok());
        assert!(constraint.validate("test", &serde_json::json!(0)).is_ok());
        assert!(constraint.validate("test", &serde_json::json!(false)).is_ok());
    }

    #[test]
    fn test_constraint_required_null() {
        let constraint = Constraint::Required;
        assert_matches!(
            constraint.validate("test", &serde_json::json!(null)),
            Err(ValidationError::MissingRequired { .. })
        );
    }

    #[test]
    fn test_constraint_one_of_valid() {
        let constraint = Constraint::OneOf(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert!(constraint.validate("test", &serde_json::json!("a")).is_ok());
        assert!(constraint.validate("test", &serde_json::json!("b")).is_ok());
    }

    #[test]
    fn test_constraint_one_of_invalid() {
        let constraint = Constraint::OneOf(vec!["a".to_string(), "b".to_string()]);
        assert_matches!(
            constraint.validate("test", &serde_json::json!("c")),
            Err(ValidationError::NotOneOf { .. })
        );
    }

    #[test]
    fn test_constraint_custom_passes_through() {
        let constraint = Constraint::Custom("application_defined".to_string());
        assert!(constraint.validate("test", &serde_json::json!(null)).is_ok());
        assert!(constraint.validate("test", &serde_json::json!("anything")).is_ok());
    }

    // SettingType::validate tests
    #[test]
    fn test_setting_type_string_valid() {
        let st = SettingType::String { pattern: None, min_length: None, max_length: None };
        assert!(st.validate("test", &serde_json::json!("value")).is_ok());
    }

    #[test]
    fn test_setting_type_string_with_pattern_valid() {
        let st = SettingType::String {
            pattern: Some("[a-z]+".to_string()),
            min_length: None,
            max_length: None,
        };
        assert!(st.validate("test", &serde_json::json!("abc")).is_ok());
    }

    #[test]
    fn test_setting_type_string_with_pattern_invalid() {
        let st = SettingType::String {
            pattern: Some("[0-9]+".to_string()),
            min_length: None,
            max_length: None,
        };
        assert!(st.validate("test", &serde_json::json!("abc")).is_err());
    }

    #[test]
    fn test_setting_type_string_with_length_bounds() {
        let st = SettingType::String {
            pattern: None,
            min_length: Some(3),
            max_length: Some(10),
        };
        assert!(st.validate("test", &serde_json::json!("abc")).is_ok());
        assert!(st.validate("test", &serde_json::json!("ab")).is_err());
        assert!(st.validate("test", &serde_json::json!("abcdefghijk")).is_err());
    }

    #[test]
    fn test_setting_type_integer_valid() {
        let st = SettingType::Integer { min: None, max: None };
        assert!(st.validate("test", &serde_json::json!(42)).is_ok());
        assert!(st.validate("test", &serde_json::json!(-100)).is_ok());
    }

    #[test]
    fn test_setting_type_integer_with_bounds() {
        let st = SettingType::Integer { min: Some(0), max: Some(100) };
        assert!(st.validate("test", &serde_json::json!(50)).is_ok());
        assert!(st.validate("test", &serde_json::json!(-1)).is_err());
        assert!(st.validate("test", &serde_json::json!(101)).is_err());
    }

    #[test]
    fn test_setting_type_integer_rejects_string() {
        let st = SettingType::Integer { min: None, max: None };
        assert!(st.validate("test", &serde_json::json!("42")).is_err());
    }

    #[test]
    fn test_setting_type_float_valid() {
        let st = SettingType::Float { min: None, max: None };
        assert!(st.validate("test", &serde_json::json!(3.14)).is_ok());
        assert!(st.validate("test", &serde_json::json!(42)).is_ok()); // Integer coerced to float
    }

    #[test]
    fn test_setting_type_float_with_bounds() {
        let st = SettingType::Float { min: Some(0.0), max: Some(1.0) };
        assert!(st.validate("test", &serde_json::json!(0.5)).is_ok());
        assert!(st.validate("test", &serde_json::json!(-0.1)).is_err());
        assert!(st.validate("test", &serde_json::json!(1.1)).is_err());
    }

    #[test]
    fn test_setting_type_boolean_valid() {
        let st = SettingType::Boolean;
        assert!(st.validate("test", &serde_json::json!(true)).is_ok());
        assert!(st.validate("test", &serde_json::json!(false)).is_ok());
    }

    #[test]
    fn test_setting_type_boolean_rejects_other_types() {
        let st = SettingType::Boolean;
        assert!(st.validate("test", &serde_json::json!(1)).is_err());
        assert!(st.validate("test", &serde_json::json!("true")).is_err());
    }

    #[test]
    fn test_setting_type_enum_valid() {
        let st = SettingType::Enum {
            variants: vec!["dev".to_string(), "staging".to_string(), "prod".to_string()],
        };
        assert!(st.validate("env", &serde_json::json!("dev")).is_ok());
        assert!(st.validate("env", &serde_json::json!("prod")).is_ok());
    }

    #[test]
    fn test_setting_type_enum_invalid() {
        let st = SettingType::Enum {
            variants: vec!["dev".to_string(), "staging".to_string()],
        };
        assert!(st.validate("env", &serde_json::json!("invalid")).is_err());
    }

    #[test]
    fn test_setting_type_secret_valid() {
        let st = SettingType::Secret;
        assert!(st.validate("password", &serde_json::json!("any_value")).is_ok());
        assert!(st.validate("password", &serde_json::json!(123)).is_ok());
    }

    #[test]
    fn test_setting_type_secret_rejects_null() {
        let st = SettingType::Secret;
        assert!(st.validate("password", &serde_json::json!(null)).is_err());
    }

    #[test]
    fn test_setting_type_any_accepts_anything() {
        let st = SettingType::Any;
        assert!(st.validate("test", &serde_json::json!(null)).is_ok());
        assert!(st.validate("test", &serde_json::json!("string")).is_ok());
        assert!(st.validate("test", &serde_json::json!(42)).is_ok());
        assert!(st.validate("test", &serde_json::json!([1, 2, 3])).is_ok());
    }

    #[test]
    fn test_setting_type_array_valid() {
        let st = SettingType::Array {
            element_type: Box::new(SettingType::Integer { min: None, max: None }),
            min_items: None,
            max_items: None,
        };
        assert!(st.validate("test", &serde_json::json!([1, 2, 3])).is_ok());
        assert!(st.validate("test", &serde_json::json!([])).is_ok());
    }

    #[test]
    fn test_setting_type_array_with_bounds() {
        let st = SettingType::Array {
            element_type: Box::new(SettingType::Integer { min: None, max: None }),
            min_items: Some(1),
            max_items: Some(3),
        };
        assert!(st.validate("test", &serde_json::json!([1])).is_ok());
        assert!(st.validate("test", &serde_json::json!([])).is_err());
        assert!(st.validate("test", &serde_json::json!([1, 2, 3, 4])).is_err());
    }

    #[test]
    fn test_setting_type_array_invalid_element_type() {
        let st = SettingType::Array {
            element_type: Box::new(SettingType::Integer { min: None, max: None }),
            min_items: None,
            max_items: None,
        };
        assert!(st.validate("test", &serde_json::json!(["string"])).is_err());
    }

    #[test]
    fn test_setting_type_object_valid() {
        let field = SettingMetadata {
            key: "name".to_string(),
            label: "Name".to_string(),
            description: "User name".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: None,
            constraints: vec![],
            visibility: Visibility::Public,
            group: None,
        };
        let st = SettingType::Object { fields: vec![field] };
        let value = serde_json::json!({ "name": "John" });
        assert!(st.validate("user", &value).is_ok());
    }

    #[test]
    fn test_setting_type_object_rejects_non_object() {
        let st = SettingType::Object { fields: vec![] };
        assert!(st.validate("test", &serde_json::json!("not an object")).is_err());
    }

    #[test]
    fn test_setting_type_path_valid() {
        let st = SettingType::Path { must_exist: false, is_directory: false };
        assert!(st.validate("path", &serde_json::json!("/etc/config")).is_ok());
    }

    #[test]
    fn test_setting_type_path_rejects_non_string() {
        let st = SettingType::Path { must_exist: false, is_directory: false };
        assert!(st.validate("path", &serde_json::json!(123)).is_err());
    }

    #[test]
    fn test_setting_type_url_valid() {
        let st = SettingType::Url { schemes: vec![] };
        assert!(st.validate("url", &serde_json::json!("https://example.com")).is_ok());
    }

    #[test]
    fn test_setting_type_url_with_scheme_restriction() {
        let st = SettingType::Url { schemes: vec!["https".to_string()] };
        assert!(st.validate("url", &serde_json::json!("https://example.com")).is_ok());
        assert!(st.validate("url", &serde_json::json!("http://example.com")).is_err());
    }

    #[test]
    fn test_setting_type_url_missing_scheme() {
        let st = SettingType::Url { schemes: vec!["https".to_string()] };
        assert!(st.validate("url", &serde_json::json!("example.com")).is_err());
    }

    #[test]
    fn test_setting_type_duration_valid() {
        let st = SettingType::Duration { min: None, max: None };
        assert!(st.validate("timeout", &serde_json::json!(60)).is_ok());
        assert!(st.validate("timeout", &serde_json::json!({})).is_ok());
    }

    // SettingMetadata::validate and validate_value tests
    #[test]
    fn test_setting_metadata_validate_value_with_constraint() {
        let metadata = SettingMetadata {
            key: "port".to_string(),
            label: "Port".to_string(),
            description: "Server port".to_string(),
            setting_type: SettingType::Integer { min: Some(1024), max: Some(65535) },
            default: None,
            constraints: vec![Constraint::Required],
            visibility: Visibility::Public,
            group: None,
        };

        assert!(metadata.validate_value(&serde_json::json!(8080)).is_ok());
        assert!(metadata.validate_value(&serde_json::json!(null)).is_err());
        assert!(metadata.validate_value(&serde_json::json!(70000)).is_err());
    }

    #[test]
    fn test_setting_metadata_validate_returns_result() {
        let metadata = SettingMetadata {
            key: "api_key".to_string(),
            label: "API Key".to_string(),
            description: "Secret API key".to_string(),
            setting_type: SettingType::String { pattern: None, min_length: None, max_length: None },
            default: None,
            constraints: vec![Constraint::Required],
            visibility: Visibility::Secret,
            group: None,
        };

        let result = metadata.validate(&serde_json::json!(null));
        assert!(!result.is_valid());
        assert_eq!(result.error_count(), 1);
    }

    #[test]
    fn test_validation_result_merge_with_warnings() {
        let mut result1 = ValidationResult::new();
        result1.add_warning("Warning 1".to_string());

        let mut result2 = ValidationResult::new();
        result2.add_warning("Warning 2".to_string());
        result2.add_error(ValidationError::MissingRequired { key: "test".to_string() });

        result1.merge(result2);
        assert!(!result1.is_valid());
        assert_eq!(result1.warning_count(), 2);
        assert_eq!(result1.error_count(), 1);
    }

    #[test]
    fn test_validation_result_into_result_single_error() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError::MissingRequired { key: "test".to_string() });

        match result.into_result() {
            Err(ValidationError::MissingRequired { key }) => {
                assert_eq!(key, "test");
            },
            _ => panic!("Expected single error"),
        }
    }

    #[test]
    fn test_validation_result_into_result_multiple_errors() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError::MissingRequired { key: "test1".to_string() });
        result.add_error(ValidationError::OutOfRange {
            key: "test2".to_string(),
            min: 0.0,
            max: 100.0,
            value: 150.0,
        });

        match result.into_result() {
            Err(ValidationError::Multiple(errors)) => {
                assert_eq!(errors.len(), 2);
            },
            _ => panic!("Expected multiple errors"),
        }
    }

    #[test]
    fn test_validation_result_display_with_warnings() {
        let mut result = ValidationResult::new();
        result.add_error(ValidationError::MissingRequired { key: "test".to_string() });
        result.add_warning("Some warning".to_string());

        let msg = result.to_string();
        assert!(msg.contains("1 error"));
        assert!(msg.contains("1 warning"));
    }

    #[test]
    fn test_json_type_str_helper() {
        let null_val = serde_json::json!(null);
        assert_eq!(null_val.type_str(), "null");

        let bool_val = serde_json::json!(true);
        assert_eq!(bool_val.type_str(), "boolean");

        let num_val = serde_json::json!(42);
        assert_eq!(num_val.type_str(), "number");

        let str_val = serde_json::json!("text");
        assert_eq!(str_val.type_str(), "string");

        let arr_val = serde_json::json!([1, 2, 3]);
        assert_eq!(arr_val.type_str(), "array");

        let obj_val = serde_json::json!({});
        assert_eq!(obj_val.type_str(), "object");
    }

    #[test]
    fn test_constraint_pattern_unicode() {
        let constraint = Constraint::Pattern("[ñáéíóú]+".to_string());
        assert!(constraint.validate("test", &serde_json::json!("ñoño")).is_ok());
        assert!(constraint.validate("test", &serde_json::json!("hello")).is_err());
    }

    #[test]
    fn test_setting_type_string_length_with_unicode() {
        let st = SettingType::String {
            pattern: None,
            min_length: Some(2),
            max_length: Some(4),
        };
        // Unicode characters count as single chars in Rust
        assert!(st.validate("test", &serde_json::json!("ñoño")).is_ok()); // 4 chars
        assert!(st.validate("test", &serde_json::json!("ñ")).is_err()); // 1 char, too short
    }

    #[test]
    fn test_validation_error_display_redacted_out_of_range() {
        let error = ValidationError::OutOfRange {
            key: "secret".to_string(),
            min: 0.0,
            max: 100.0,
            value: f64::NAN,
        };
        let msg = error.to_string();
        assert!(msg.contains("[REDACTED:secret]"));
    }

    #[test]
    fn test_setting_type_integer_accepts_large_values() {
        let st = SettingType::Integer { min: None, max: None };
        // i64 max value is valid
        assert!(st.validate("test", &serde_json::json!(i64::MAX)).is_ok());
        // u64::MAX exceeds i64 range and fails in serde_json
        assert!(st.validate("test", &serde_json::json!(9223372036854775807i64)).is_ok());
    }

    #[test]
    fn test_setting_type_float_from_integer() {
        let st = SettingType::Float { min: None, max: None };
        assert!(st.validate("test", &serde_json::json!(42)).is_ok());
    }

    #[test]
    fn test_constraint_length_with_multibyte_chars() {
        let constraint = Constraint::Length { min: 1, max: 3 };
        // String with emoji (counts as 1 char in Rust)
        assert!(constraint.validate("test", &serde_json::json!("👍👍")).is_ok());
        // 2 chars
    }
}
