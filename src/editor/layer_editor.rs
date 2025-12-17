//! Layer editor trait for modifying configuration files.

use serde::{Deserialize, Serialize};

use super::EditorError;

/// Edit a single configuration layer (file).
///
/// Provides type-safe get/set operations on configuration files with support for
/// nested values via dotted paths, dirty flag tracking, and atomic saves.
///
/// # Example
///
/// ```ignore
/// use settings_loader::editor::LayerEditor;
/// use std::path::Path;
///
/// // Open a configuration file
/// let mut editor = todo!(); // Open implementation
///
/// // Get a value (type safe)
/// if let Some(host) = editor.get::<String>("database.host") {
///     println!("Current host: {}", host);
/// }
///
/// // Set a value
/// editor.set("database.host", "db.example.com")?;
///
/// // Check for unsaved changes
/// if editor.is_dirty() {
///     editor.save()?;
/// }
/// ```
pub trait LayerEditor: Send + Sync {
    /// Get a setting value by key, supporting dotted paths for nested keys.
    ///
    /// Returns `Some(value)` if the key exists and can be deserialized to type `T`,
    /// or `None` if the key doesn't exist or type conversion fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let host: Option<String> = editor.get("database.host");
    /// let port: Option<u16> = editor.get("database.port");
    /// ```
    fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T>;

    /// Set a setting value by key, supporting dotted paths for nested keys.
    ///
    /// If the key doesn't exist, it is created along with any missing parent
    /// structures. Marks the editor as dirty (unsaved changes).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The value cannot be serialized to the file format
    /// - The key path is invalid
    ///
    /// # Example
    ///
    /// ```ignore
    /// editor.set("database.host", "db.example.com")?;
    /// editor.set("database.port", 5432)?;
    /// ```
    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), EditorError>;

    /// Remove a setting key from the configuration layer.
    ///
    /// Deletes the specified key from the configuration file. Supports nested keys
    /// via dotted paths (e.g., `"database.port"`). The key must exist or an error
    /// is returned.
    ///
    /// This method marks the editor as dirty; changes are not persisted until
    /// `save()` is called.
    ///
    /// # Behavior
    ///
    /// - Removes only the specified key, leaving other keys intact
    /// - Works with nested structures via dotted paths
    /// - Does NOT recursively delete parent structures if they become empty
    /// - Parent structures are preserved even if all their children are deleted
    /// - Comments on the deleted line are removed (TOML, YAML)
    ///
    /// # Format-Specific Behavior
    ///
    /// - **TOML**: Comments for surrounding keys are preserved; deleted key's
    ///   comments are removed; formatting preserved
    /// - **JSON**: Simple key deletion; array indices not supported
    /// - **YAML**: Key and associated comments removed; formatting preserved
    ///
    /// # Errors
    ///
    /// Returns `EditorError::KeyNotFound` if:
    /// - The key does not exist in the configuration
    /// - A parent structure in a dotted path doesn't exist (e.g., trying to delete
    ///   `"database.port"` when `[database]` section doesn't exist)
    ///
    /// # Examples
    ///
    /// Delete a top-level key:
    ///
    /// ```ignore
    /// // Before: { "debug": true, "version": "1.0" }
    /// editor.unset("debug")?;
    /// // After:  { "version": "1.0" }
    /// ```
    ///
    /// Delete a nested key via dotted path:
    ///
    /// ```ignore
    /// // Before: [database]
    /// //         host = "localhost"
    /// //         port = 5432         ← will be deleted
    /// //         ssl = true
    /// editor.unset("database.port")?;
    /// // After:  [database]
    /// //         host = "localhost"
    /// //         ssl = true
    /// ```
    ///
    /// Error case - key doesn't exist:
    ///
    /// ```ignore
    /// editor.unset("nonexistent_key")?;  // ❌ Returns KeyNotFound error
    /// ```
    ///
    /// Persisting deletions:
    ///
    /// ```ignore
    /// editor.unset("old_setting")?;
    /// assert!(editor.is_dirty());        // True - changes not saved yet
    /// editor.save()?;                    // Persists deletion to file
    /// assert!(!editor.is_dirty());       // False - changes saved
    /// ```
    ///
    /// # Marks as Dirty
    ///
    /// After calling `unset()`, `is_dirty()` returns `true` until `save()` is called.
    ///
    /// # See Also
    ///
    /// - `set()` - Add or modify a configuration key
    /// - `get()` - Retrieve a configuration value
    /// - `is_dirty()` - Check for unsaved changes
    /// - `save()` - Persist changes to file
    fn unset(&mut self, key: &str) -> Result<(), EditorError>;

    /// Get all available keys in this configuration layer.
    ///
    /// Returns a list of all top-level keys (not including nested keys in their entirety).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let keys = editor.keys();
    /// println!("Available settings: {:?}", keys);
    /// ```
    fn keys(&self) -> Vec<String>;

    /// Check if this editor has unsaved changes.
    ///
    /// Returns `true` if any set/unset operations were performed since opening
    /// or since the last successful save().
    ///
    /// # Example
    ///
    /// ```ignore
    /// if editor.is_dirty() {
    ///     editor.save()?;
    /// }
    /// ```
    fn is_dirty(&self) -> bool;

    /// Save changes back to the configuration file.
    ///
    /// Atomically writes changes to a temporary file, then renames it to the
    /// original file location. This ensures that the file is either fully updated
    /// or unchanged - never partially written.
    ///
    /// Clears the dirty flag after successful save.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be written (permissions, disk full, etc.)
    /// - Format-specific serialization fails
    ///
    /// # Format-Specific Behavior
    ///
    /// - **TOML**: Comments and formatting are preserved
    /// - **JSON**: No comment preservation (JSON spec limitation)
    /// - **YAML**: Comments may not be preserved (YAML limitation)
    ///
    /// # Example
    ///
    /// ```ignore
    /// editor.set("debug", true)?;
    /// editor.save()?;  // Changes now persisted
    /// ```
    fn save(&mut self) -> Result<(), EditorError>;
}
