//! Source provenance tracking for configuration settings.
//!
//! This module provides the structures and logic to track where each configuration
//! value originated (e.g., specific file, environment variable, default value).
//! This is essential for:
//! - Multi-scope configuration (knowing if a value is UserGlobal or ProjectLocal)
//! - Layer-aware editing (knowing which file to update)
//! - UI visualization (showing the user the source of a setting)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::scope::ConfigScope;

/// Identifies the kind of source that provided a configuration value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceType {
    /// Default values provided by the application code
    Default,
    /// Configuration loaded from a file
    File,
    /// Configuration loaded from environment variables
    Environment,
    /// Configuration loaded from a secrets file
    Secrets,
    /// Configuration provided by CLI arguments or runtime overrides
    Override,
}

/// Metadata describing the origin of a configuration value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMetadata {
    /// Unique identifier for the source (e.g., "file:settings.toml", "env:APP_PORT")
    pub id: String,
    /// The type of source
    pub source_type: SourceType,
    /// The file path, if applicable (Standardized absolute path)
    pub path: Option<PathBuf>,
    /// The configuration scope, if applicable
    pub scope: Option<ConfigScope>,
}

impl SourceMetadata {
    /// Create metadata for a file source
    pub fn file(path: PathBuf, scope: Option<ConfigScope>) -> Self {
        let id = format!("file:{}", path.display());
        Self {
            id,
            source_type: SourceType::File,
            path: Some(path),
            scope,
        }
    }

    /// Create metadata for an environment variable source
    pub fn env(var_name: String) -> Self {
        Self {
            id: format!("env:{}", var_name),
            source_type: SourceType::Environment,
            path: None,
            scope: Some(ConfigScope::Runtime),
        }
    }
}

impl Default for SourceMetadata {
    fn default() -> Self {
        Self {
            id: "<none>".to_string(),
            source_type: SourceType::Default,
            path: None,
            scope: None,
        }
    }
}

/// A map tracking the source of every configuration key.
///
/// Keys are dotted paths (e.g., "database.host").
/// Values are the metadata of the source that provided the *winning* value for that key.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceMap {
    /// Map of setting key -> SourceMetadata
    entries: HashMap<String, SourceMetadata>,
}

impl SourceMap {
    /// Create a new empty source map
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    /// Record a source for a given key
    pub fn insert(&mut self, key: String, metadata: SourceMetadata) {
        self.entries.insert(key, metadata);
    }

    /// Get the source metadata for a specific key
    pub fn source_of(&self, key: &str) -> Option<&SourceMetadata> {
        self.entries.get(key)
    }

    /// Get all keys provided by a specific source type
    pub fn keys_from(&self, source_type: SourceType) -> Vec<String> {
        self.entries
            .iter()
            .filter(|(_, meta)| meta.source_type == source_type)
            .map(|(key, _)| key.clone())
            .collect()
    }
}
