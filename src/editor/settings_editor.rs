//! Settings editor factory trait for creating layer editors.

use std::path::Path;

use super::{ConfigFormat, EditorError, LayerEditor};

/// Factory trait for creating layer editors with format auto-detection.
///
/// Provides methods to open existing configuration files or create new ones
/// with automatic format detection from file extensions.
///
/// # Example
///
/// ```ignore
/// use settings_loader::editor::SettingsEditor;
/// use std::path::Path;
///
/// // Open existing file with auto-detected format
/// let editor = SettingsEditor::open(Path::new("settings.toml"))?;
///
/// // Create new file with explicit format
/// let editor = SettingsEditor::create(
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
    /// let editor = SettingsEditor::open(Path::new("settings.toml"))?;
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
    /// use settings_loader::editor::ConfigFormat;
    ///
    /// let editor = SettingsEditor::create(
    ///     Path::new("config.yaml"),
    ///     ConfigFormat::Yaml
    /// )?;
    /// ```
    fn create(path: &Path, format: ConfigFormat) -> Result<Self::Editor, EditorError>;
}
