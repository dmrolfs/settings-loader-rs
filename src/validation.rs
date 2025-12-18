//! Phase 5.3: Validation System - Core Types
//!
//! This module provides the validation framework for constraint-based value validation
//! using metadata from Phase 5.1 and introspection from Phase 5.2.
//!
//! # Core Types
//!
//! - [`ValidationError`] - Detailed validation failure information
//! - [`ValidationResult`] - Aggregation of validation results and warnings

use std::fmt;

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
            ValidationError::Multiple(errors) => {
                errors
                    .iter()
                    .flat_map(|e| e.flatten())
                    .collect()
            }
            _ => vec![self],
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::ConstraintViolation { key, reason } => {
                write!(f, "{}: {}", key, reason)
            }
            ValidationError::TypeMismatch {
                key,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "{}: type mismatch - expected {}, got {}",
                    key, expected, actual
                )
            }
            ValidationError::MissingRequired { key } => {
                write!(f, "{}: required field is missing", key)
            }
            ValidationError::InvalidPattern {
                key,
                pattern,
                value,
            } => {
                write!(
                    f,
                    "{}: value '{}' does not match required pattern '{}'",
                    key, value, pattern
                )
            }
            ValidationError::OutOfRange {
                key,
                min,
                max,
                value,
            } => {
                write!(
                    f,
                    "{}: value {} is outside allowed range [{}, {}]",
                    key, value, min, max
                )
            }
            ValidationError::TooShort { key, min, length } => {
                write!(
                    f,
                    "{}: length {} is shorter than minimum {}",
                    key, length, min
                )
            }
            ValidationError::TooLong { key, max, length } => {
                write!(
                    f,
                    "{}: length {} exceeds maximum {}",
                    key, length, max
                )
            }
            ValidationError::NotOneOf {
                key,
                expected,
                actual,
            } => {
                let expected_str = expected.join(", ");
                write!(
                    f,
                    "{}: '{}' is not one of allowed values: {}",
                    key, actual, expected_str
                )
            }
            ValidationError::CustomValidation { key, message } => {
                write!(f, "{}: {}", key, message)
            }
            ValidationError::Multiple(errors) => {
                writeln!(f, "Multiple validation errors:")?;
                for (idx, error) in errors.iter().enumerate() {
                    write!(f, "  {}. {}", idx + 1, error)?;
                    if idx < errors.len() - 1 {
                        writeln!(f)?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ValidationError {}

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
        let error = ValidationError::MissingRequired {
            key: "api_key".to_string(),
        };
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
            expected: vec![
                "dev".to_string(),
                "staging".to_string(),
                "prod".to_string(),
            ],
            actual: "invalid".to_string(),
        };
        let msg = error.to_string();
        assert!(msg.contains("env"));
        assert!(msg.contains("invalid"));
        assert!(msg.contains("dev"));
    }

    #[test]
    fn validation_error_key_extraction() {
        let error = ValidationError::MissingRequired {
            key: "test_key".to_string(),
        };
        assert_eq!(error.key(), Some("test_key"));
    }

    #[test]
    fn validation_error_multiple_flatten() {
        let error = ValidationError::Multiple(vec![
            ValidationError::MissingRequired {
                key: "key1".to_string(),
            },
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

        result.add_error(ValidationError::MissingRequired {
            key: "test".to_string(),
        });

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
        result1.add_error(ValidationError::MissingRequired {
            key: "key1".to_string(),
        });

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
        result.add_error(ValidationError::MissingRequired {
            key: "test".to_string(),
        });
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
        result.add_error(ValidationError::MissingRequired {
            key: "test".to_string(),
        });
        let msg = result.to_string();
        assert!(msg.contains("Validation failed"));
        assert!(msg.contains("1 error"));
    }
}
