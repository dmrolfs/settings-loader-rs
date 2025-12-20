//! Configuration editing and writing support
//!
//! This module provides bidirectional configuration management with format-specific
//! backends. Enables reading, modifying, and saving configuration files while
//! preserving format-specific semantics (e.g., TOML comments).
//!
//! # Features
//!
//! - **Bidirectional**: Read configuration, modify values, save changes
//! - **Format Support**: TOML (with comment preservation), JSON, YAML
//! - **Type Safety**: Typed get/set operations with serde
//! - **Dotted Paths**: Navigate nested structures with "database.host" notation
//! - **Atomic Writes**: Safe write operations prevent corruption
//! - **Dirty Tracking**: Know when unsaved changes exist
//!
//! # Example
//!
//! ```ignore
//! use settings_loader::editor::{LayerEditor, ConfigFormat};
//! use std::path::Path;
//!
//! // Open a configuration file
//! let mut editor = LayerEditor::open(Path::new("settings.toml"))?;
//!
//! // Get a value (type safe)
//! let debug: bool = editor.get("debug").unwrap_or(false);
//!
//! // Set a value
//! editor.set("debug", true)?;
//!
//! // Save changes (atomic write)
//! editor.save()?;
//! ```
//!
//! # Feature Flag
//!
//! This module requires the `editor` feature flag:
//!
//! ```toml
//! [dependencies]
//! settings_loader = { version = "0.15", features = ["editor"] }
//! ```

pub use self::error::EditorError;
pub use self::format::ConfigFormat;
use self::json::JsonLayerEditor;
pub use self::layer_editor::LayerEditor;
pub use self::settings_editor::{SettingsEditor, SettingsLoaderEditor};
use self::toml::TomlLayerEditor;
use self::yaml::YamlLayerEditor;
use serde::{de::DeserializeOwned, Serialize};

pub mod config_editor;
pub mod error;
pub mod format;
pub mod json;
pub mod layer_editor;
pub mod settings_editor;
pub mod toml;
pub mod yaml;

pub use self::config_editor::ConfigEditor;

/// An enum that represents a concrete `LayerEditor` for a specific format.
/// This allows for a single, unified interface when working with different
/// configuration file types.
#[derive(Debug)]
pub enum Editor {
    Toml(TomlLayerEditor),
    Json(JsonLayerEditor),
    Yaml(YamlLayerEditor),
}

impl LayerEditor for Editor {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        match self {
            Editor::Toml(editor) => editor.get(key),
            Editor::Json(editor) => editor.get(key),
            Editor::Yaml(editor) => editor.get(key),
        }
    }

    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), EditorError> {
        match self {
            Editor::Toml(editor) => editor.set(key, value),
            Editor::Json(editor) => editor.set(key, value),
            Editor::Yaml(editor) => editor.set(key, value),
        }
    }

    fn unset(&mut self, key: &str) -> Result<(), EditorError> {
        match self {
            Editor::Toml(editor) => editor.unset(key),
            Editor::Json(editor) => editor.unset(key),
            Editor::Yaml(editor) => editor.unset(key),
        }
    }

    fn keys(&self) -> Vec<String> {
        match self {
            Editor::Toml(editor) => editor.keys(),
            Editor::Json(editor) => editor.keys(),
            Editor::Yaml(editor) => editor.keys(),
        }
    }

    fn is_dirty(&self) -> bool {
        match self {
            Editor::Toml(editor) => editor.is_dirty(),
            Editor::Json(editor) => editor.is_dirty(),
            Editor::Yaml(editor) => editor.is_dirty(),
        }
    }

    fn save(&mut self) -> Result<(), EditorError> {
        match self {
            Editor::Toml(editor) => editor.save(),
            Editor::Json(editor) => editor.save(),
            Editor::Yaml(editor) => editor.save(),
        }
    }
}
