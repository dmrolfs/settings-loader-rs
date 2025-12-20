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
use std::fmt;
use std::path::PathBuf;

use crate::scope::ConfigScope;
pub use crate::scope::ConfigScope as SourceScope;

/// Identifies the specific source that provided a configuration value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SettingSource {
    /// Default values provided by the application code or metadata
    Default,
    /// Configuration loaded from a file
    File {
        /// Standardized absolute path to the file
        path: PathBuf,
        /// The configuration scope (e.g., UserGlobal, ProjectLocal)
        #[serde(skip_serializing_if = "Option::is_none")]
        scope: Option<ConfigScope>,
    },
    /// Configuration loaded from a specific environment variable
    EnvVar {
        /// Name of the environment variable
        name: String,
    },
    /// Configuration loaded from a set of environment variables (e.g., APP_*)
    EnvVars {
        /// Prefix used for the search
        prefix: String,
    },
    /// Configuration loaded from a secrets file
    Secrets {
        /// Path to the secrets file
        path: PathBuf,
    },
    /// Configuration provided by CLI arguments or runtime overrides
    Override {
        /// Name/identifier of the override
        name: String,
    },
}

impl fmt::Display for SettingSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::File { path, scope } => {
                let scope_str = scope.map(|s| format!(" ({:?})", s)).unwrap_or_default();
                write!(f, "file:{}{}", path.display(), scope_str)
            },
            Self::EnvVar { name } => write!(f, "env:{}", name),
            Self::EnvVars { prefix } => write!(f, "env_vars:{}*", prefix),
            Self::Secrets { path } => write!(f, "secrets:{}", path.display()),
            Self::Override { name } => write!(f, "override:{}", name),
        }
    }
}

/// Metadata describing the origin and precedence of a configuration value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMetadata {
    /// The specific source origin
    pub source: SettingSource,
    /// Precedence layer index (higher values mean higher precedence)
    pub layer_index: usize,
}

impl SourceMetadata {
    /// Create metadata for a file source
    pub fn file(path: PathBuf, scope: Option<ConfigScope>, layer_index: usize) -> Self {
        Self {
            source: SettingSource::File { path, scope },
            layer_index,
        }
    }

    /// Create metadata for an environment variable source
    pub fn env(name: String, layer_index: usize) -> Self {
        Self {
            source: SettingSource::EnvVar { name },
            layer_index,
        }
    }

    /// Create metadata for a default source
    pub fn default(layer_index: usize) -> Self {
        Self { source: SettingSource::Default, layer_index }
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

    /// Record a source for a given key.
    ///
    /// If a key already exists, it is only updated if the new metadata has
    /// a higher or equal layer index (higher precedence).
    pub fn insert(&mut self, key: String, metadata: SourceMetadata) {
        if let Some(existing) = self.entries.get(&key) {
            if metadata.layer_index >= existing.layer_index {
                self.entries.insert(key, metadata);
            }
        } else {
            self.entries.insert(key, metadata);
        }
    }

    /// Get the source metadata for a specific key
    pub fn source_of(&self, key: &str) -> Option<&SourceMetadata> {
        self.entries.get(key)
    }

    /// Get all entries in the source map
    pub fn entries(&self) -> &HashMap<String, SourceMetadata> {
        &self.entries
    }

    /// Generate a structured audit report of all configuration sources.
    /// Inserts all keys from a source into the map, only if they have higher precedence (higher layer_index).
    pub fn insert_layer(&mut self, metadata: SourceMetadata, props: HashMap<String, config::Value>) {
        for key in props.keys() {
            let should_insert = match self.entries.get(key) {
                Some(existing) => metadata.layer_index >= existing.layer_index,
                None => true,
            };

            if should_insert {
                self.entries.insert(key.clone(), metadata.clone());
            }
        }
    }

    pub fn audit_report(&self) -> String {
        let mut report = String::from("Configuration Audit Report\n");
        report.push_str("==========================\n\n");

        let mut sorted_keys: Vec<_> = self.entries.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            let meta = &self.entries[key];
            report.push_str(&format!("{:<30} -> Layer {}: {}\n", key, meta.layer_index, meta.source));
        }

        report
    }
}
