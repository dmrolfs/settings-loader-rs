//! Error types for configuration editing operations.

use std::io;

/// Error type for configuration editing operations.
#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    /// IO error (file not found, permission denied, etc.)
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Configuration file parsing error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Value serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Configuration key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Configuration format mismatch
    #[error("Format mismatch: expected format differs from file")]
    FormatMismatch,

    /// Type mismatch when getting/setting values
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    /// Invalid dotted path format
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

impl EditorError {
    /// Create a KeyNotFound error with the given key
    pub fn key_not_found(key: impl Into<String>) -> Self {
        EditorError::KeyNotFound(key.into())
    }

    /// Create a ParseError with the given message
    pub fn parse_error(msg: impl Into<String>) -> Self {
        EditorError::ParseError(msg.into())
    }

    /// Create a SerializationError with the given message
    pub fn serialization_error(msg: impl Into<String>) -> Self {
        EditorError::SerializationError(msg.into())
    }

    /// Create a TypeMismatch error
    pub fn type_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        EditorError::TypeMismatch { expected: expected.into(), actual: actual.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_error_io_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = EditorError::from(io_err);
        assert!(err.to_string().contains("IO error"));
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_editor_error_parse_error() {
        let err = EditorError::parse_error("invalid TOML format");
        assert!(err.to_string().contains("Parse error: invalid TOML format"));
    }

    #[test]
    fn test_editor_error_serialization_error() {
        let err = EditorError::serialization_error("failed to serialize data");
        assert!(err.to_string().contains("Serialization error: failed to serialize data"));
    }

    #[test]
    fn test_editor_error_key_not_found() {
        let err = EditorError::key_not_found("database.host");
        assert!(err.to_string().contains("Key not found: database.host"));
    }

    #[test]
    fn test_editor_error_format_mismatch() {
        let err = EditorError::FormatMismatch;
        assert!(err
            .to_string()
            .contains("Format mismatch: expected format differs from file"));
    }

    #[test]
    fn test_editor_error_type_mismatch() {
        let err = EditorError::type_mismatch("integer", "string");
        assert!(err.to_string().contains("Type mismatch: expected integer, got string"));
    }

    #[test]
    fn test_editor_error_invalid_path() {
        let err = EditorError::InvalidPath("invalid.nested.key".to_string());
        assert!(err.to_string().contains("Invalid path: invalid.nested.key"));
    }
}
