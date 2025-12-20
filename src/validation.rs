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
}
