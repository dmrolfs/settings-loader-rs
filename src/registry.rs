//! Settings Registry
//!
//! This module provides a centralized registry for configuration metadata.
//! The SettingsRegistry allows applications to define and store metadata about
//! their settings (types, descriptions, defaults, constraints) which can then
//! be used for validation, introspection, and UI generation.

use crate::introspection::SettingsIntrospection;
use crate::metadata::{ConfigSchema, SettingGroup, SettingMetadata};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A centralized registry for configuration metadata
///
/// The SettingsRegistry stores metadata about configuration settings, allowing
/// for runtime discovery and validation. It supports grouping settings and
/// generating a complete ConfigSchema.
#[derive(Debug, Clone, Default)]
pub struct SettingsRegistry {
    name: String,
    version: String,
    settings: HashMap<String, SettingMetadata>,
    groups: HashMap<String, SettingGroup>,
}

impl SettingsRegistry {
    /// Create a new, empty settings registry
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            settings: HashMap::new(),
            groups: HashMap::new(),
        }
    }

    /// Register metadata for a single setting
    pub fn register(&mut self, metadata: SettingMetadata) {
        self.settings.insert(metadata.key.clone(), metadata);
    }

    /// Register multiple settings
    pub fn register_many(&mut self, settings: impl IntoIterator<Item = SettingMetadata>) {
        for setting in settings {
            self.register(setting);
        }
    }

    /// Register a group of settings
    pub fn register_group(&mut self, group: SettingGroup) {
        self.groups.insert(group.name.clone(), group);
    }

    /// Get metadata for a specific setting by key
    pub fn metadata(&self, key: &str) -> Option<&SettingMetadata> {
        self.settings.get(key)
    }

    /// Generate a complete ConfigSchema from the registered metadata
    pub fn generate_schema(&self) -> ConfigSchema {
        ConfigSchema {
            name: self.name.clone(),
            version: self.version.clone(),
            settings: self.settings.values().cloned().collect(),
            groups: self.groups.values().cloned().collect(),
        }
    }
}

impl SettingsIntrospection for SettingsRegistry {
    fn schema(&self) -> ConfigSchema {
        self.generate_schema()
    }
}

/// Global settings registry for the application
static GLOBAL_REGISTRY: Lazy<Arc<RwLock<SettingsRegistry>>> =
    Lazy::new(|| Arc::new(RwLock::new(SettingsRegistry::new("default", "0.1.0"))));

/// Initialize the global settings registry with name and version
pub fn init_global_registry(name: &str, version: &str) {
    let mut registry = GLOBAL_REGISTRY.write().unwrap();
    registry.name = name.to_string();
    registry.version = version.to_string();
}

/// Register a setting in the global registry
pub fn register_setting(metadata: SettingMetadata) {
    let mut registry = GLOBAL_REGISTRY.write().unwrap();
    registry.register(metadata);
}

/// Get a copy of the global schema
pub fn global_schema() -> ConfigSchema {
    let registry = GLOBAL_REGISTRY.read().unwrap();
    registry.generate_schema()
}
