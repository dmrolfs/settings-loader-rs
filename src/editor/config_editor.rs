#![cfg(feature = "editor")]
//! Core configuration editor that coordinates multi-layer edits.
//!
//! `ConfigEditor` uses provenance metadata (`SourceMap`) to determine which
//! source file provided each configuration value. This allows targeted edits
//! that persist changes back to the original source while preserving comments
//! and formatting (for supported formats like TOML).

use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{Editor, EditorError, LayerEditor, SettingsEditor, SettingsLoaderEditor};
use crate::provenance::{SettingSource, SourceMap};

/// Orchestrates multi-layer configuration editing using provenance data.
#[derive(Debug, Default)]
pub struct ConfigEditor {
    /// Tracking map of key -> source metadata
    source_map: SourceMap,
    /// Cache of open layer editors, keyed by their absolute file path
    editors: HashMap<PathBuf, Editor>,
    /// Default target file for new keys
    default_target: Option<PathBuf>,
}

impl ConfigEditor {
    /// Create a new `ConfigEditor` with the given source map.
    pub fn new(source_map: SourceMap) -> Self {
        Self {
            source_map,
            editors: HashMap::new(),
            default_target: None,
        }
    }

    /// Set the default target file for new keys.
    ///
    /// When setting a key that doesn't exist in any source, the `ConfigEditor`
    /// will use this file as the target.
    pub fn set_default_target(&mut self, path: PathBuf) {
        self.default_target = Some(path);
    }

    /// Get the source map used by this editor.
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    /// Get a value for a specific key.
    ///
    /// This retrieves the value from the current state of the editors. If the
    /// editor for the source of this key is not yet open, it will be opened.
    pub fn get<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, EditorError> {
        let path = {
            let metadata = match self.source_map.source_of(key) {
                Some(meta) => meta,
                None => return Ok(None),
            };

            match &metadata.source {
                SettingSource::File { path, .. } => path.clone(),
                _ => return Ok(None),
            }
        };

        let editor = self.get_or_open_editor(&path)?;
        Ok(editor.get(key))
    }

    /// Set a value for a specific key.
    ///
    /// If the key exists, it will be updated in its original source file.
    /// If the key is new, it will be added to the default target file.
    ///
    /// # Errors
    ///
    /// Returns `EditorError::KeyNotFound` if the key is new and no default target is set.
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), EditorError> {
        let path = if let Some(meta) = self.source_map.source_of(key) {
            match &meta.source {
                SettingSource::File { path, .. } => path.clone(),
                other => {
                    return Err(EditorError::InvalidPath(format!(
                        "Source for key '{}' is not a file: {:?}",
                        key, other
                    )));
                },
            }
        } else if let Some(target) = &self.default_target {
            target.clone()
        } else {
            return Err(EditorError::KeyNotFound(format!(
                "Key '{}' not found and no default target set",
                key
            )));
        };

        let editor = self.get_or_open_editor(&path)?;
        editor.set(key, value)?;
        Ok(())
    }

    /// Unset a key.
    ///
    /// Removes the key from its original source file.
    pub fn unset(&mut self, key: &str) -> Result<(), EditorError> {
        let path = {
            let metadata = self
                .source_map
                .source_of(key)
                .ok_or_else(|| EditorError::KeyNotFound(key.to_string()))?;

            match &metadata.source {
                SettingSource::File { path, .. } => path.clone(),
                _ => {
                    return Err(EditorError::InvalidPath(format!(
                        "Source for key '{}' is not a file",
                        key
                    )));
                },
            }
        };

        let editor = self.get_or_open_editor(&path)?;
        editor.unset(key)
    }

    /// Save all changes back to their respective files.
    ///
    /// This performs atomic writes for all "dirty" layer editors.
    pub fn save(&mut self) -> Result<(), EditorError> {
        for editor in self.editors.values_mut() {
            if editor.is_dirty() {
                editor.save()?;
            }
        }
        Ok(())
    }

    /// Check if any of the open editors have unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.editors.values().any(|e| e.is_dirty())
    }

    /// Get a list of all files that have been modified.
    pub fn dirty_files(&self) -> Vec<PathBuf> {
        self.editors
            .iter()
            .filter(|(_, e)| e.is_dirty())
            .map(|(p, _)| p.clone())
            .collect()
    }

    /// Helper to get an editor from cache or open it.
    fn get_or_open_editor(&mut self, path: &Path) -> Result<&mut Editor, EditorError> {
        if !self.editors.contains_key(path) {
            let editor = if path.exists() {
                SettingsLoaderEditor::open(path)?
            } else {
                // Determine format from extension
                let format = super::ConfigFormat::from_path(path).ok_or(EditorError::FormatMismatch)?;
                SettingsLoaderEditor::create(path, format)?
            };
            self.editors.insert(path.to_path_buf(), editor);
        }

        Ok(self.editors.get_mut(path).unwrap())
    }
}
