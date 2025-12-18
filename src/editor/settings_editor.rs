#![cfg(feature = "editor")]
/// Factory trait for creating layer editors with format auto-detection.
use std::path::Path;

use super::json::JsonLayerEditor;
use super::toml::TomlLayerEditor;
use super::yaml::YamlLayerEditor;
use super::{ConfigFormat, Editor, EditorError, LayerEditor};

/// Factory trait for creating layer editors with format auto-detection.
///
/// Provides methods to open existing configuration files or create new ones
/// with automatic format detection from file extensions.
///
/// # Example
///
/// ```ignore
/// use settings_loader::editor::{SettingsEditor, SettingsLoaderEditor, ConfigFormat};
/// use std::path::Path;
///
/// // Open existing file with auto-detected format
/// let editor = SettingsLoaderEditor::open(Path::new("settings.toml"))?;
///
/// // Create new file with explicit format
/// let editor = SettingsLoaderEditor::create(
///     Path::new("new_config.json"),
///     ConfigFormat::Json
/// )?;
/// ```
pub trait SettingsEditor {
    /// The concrete layer editor type this factory produces
    type Editor: LayerEditor;

    /// Open an existing configuration file with format auto-detection.
    ///
    /// Detects the format from the file extension and opens the appropriate editor.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file doesn't exist
    /// - The file extension is not recognized
    /// - The file cannot be parsed
    ///
    /// # Example
    ///
    /// ```ignore
    /// use settings_loader::editor::{SettingsEditor, SettingsLoaderEditor};
    /// use std::path::Path;
    /// let editor = SettingsLoaderEditor::open(Path::new("settings.toml"))?;
    /// ```
    fn open(path: &Path) -> Result<Self::Editor, EditorError>;

    /// Create a new configuration file with an explicit format.
    ///
    /// Creates an empty configuration file with the specified format. The file
    /// is created immediately on the filesystem.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created (permissions, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use settings_loader::editor::{SettingsEditor, SettingsLoaderEditor, ConfigFormat};
    /// use std::path::Path;
    /// let editor = SettingsLoaderEditor::create(
    ///     Path::new("config.yaml"),
    ///     ConfigFormat::Yaml
    /// )?;
    /// ```
    fn create(path: &Path, format: ConfigFormat) -> Result<Self::Editor, EditorError>;
}

/// Concrete implementation of `SettingsEditor` that acts as a factory
/// for creating and opening various `LayerEditor` types.
#[derive(Debug, Default)]
pub struct SettingsLoaderEditor;

impl SettingsEditor for SettingsLoaderEditor {
    type Editor = Editor;

    fn open(path: &Path) -> Result<Self::Editor, EditorError> {
        let format = ConfigFormat::from_path(path).ok_or(EditorError::FormatMismatch)?;

        match format {
            ConfigFormat::Toml => Ok(Editor::Toml(TomlLayerEditor::open(path)?)),
            ConfigFormat::Json => Ok(Editor::Json(JsonLayerEditor::open(path)?)),
            ConfigFormat::Yaml => Ok(Editor::Yaml(YamlLayerEditor::open(path)?)),
            // _ => Err(EditorError::FormatMismatch), // Hjson and Ron are not yet supported for editing
        }
    }

    fn create(path: &Path, format: ConfigFormat) -> Result<Self::Editor, EditorError> {
        match format {
            ConfigFormat::Toml => Ok(Editor::Toml(TomlLayerEditor::create(path)?)),
            ConfigFormat::Json => Ok(Editor::Json(JsonLayerEditor::create(path)?)),
            ConfigFormat::Yaml => Ok(Editor::Yaml(YamlLayerEditor::create(path)?)),
            // _ => Err(EditorError::FormatMismatch), // Hjson and Ron are not yet supported for editing
        }
    }
}
