//! Configuration editing and writing support (Phase 4)
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

pub use self::format::ConfigFormat;
pub use self::error::EditorError;
pub use self::layer_editor::LayerEditor;
pub use self::settings_editor::SettingsEditor;

pub mod format;
pub mod error;
pub mod layer_editor;
pub mod settings_editor;

// Format-specific backends will be in separate modules
// pub mod toml;
// pub mod json;
// pub mod yaml;
