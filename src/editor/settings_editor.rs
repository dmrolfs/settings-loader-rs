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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::LayerEditor;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_settings_loader_editor_open_toml() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"key = \"value\"").unwrap();
        file.flush().unwrap();
        let path = file.path().with_extension("toml");
        fs::rename(file.path(), &path).unwrap();

        let editor = SettingsLoaderEditor::open(&path).unwrap();
        if let Editor::Toml(toml_editor) = editor {
            assert_eq!(toml_editor.get::<String>("key"), Some("value".to_string()));
        } else {
            panic!("Expected Toml editor");
        }
    }

    #[test]
    fn test_settings_loader_editor_open_json() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"{\"key\": \"value\"}").unwrap();
        file.flush().unwrap();
        let path = file.path().with_extension("json");
        fs::rename(file.path(), &path).unwrap();

        let editor = SettingsLoaderEditor::open(&path).unwrap();
        if let Editor::Json(json_editor) = editor {
            assert_eq!(json_editor.get::<String>("key"), Some("value".to_string()));
        } else {
            panic!("Expected Json editor");
        }
    }

    #[test]
    fn test_settings_loader_editor_open_yaml() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"key: value").unwrap();
        file.flush().unwrap();
        let path = file.path().with_extension("yaml");
        fs::rename(file.path(), &path).unwrap();

        let editor = SettingsLoaderEditor::open(&path).unwrap();
        if let Editor::Yaml(yaml_editor) = editor {
            assert_eq!(yaml_editor.get::<String>("key"), Some("value".to_string()));
        } else {
            panic!("Expected Yaml editor");
        }
    }

    #[test]
    fn test_settings_loader_editor_open_unrecognized_format() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().with_extension("txt"); // Unrecognized extension
        std::fs::write(&path, b"some content").unwrap();

        let res = SettingsLoaderEditor::open(&path);
        assert!(res.is_err());
        if let Err(EditorError::FormatMismatch) = res {
            // Expected error
        } else {
            panic!("Expected FormatMismatch error, got {:?}", res);
        }
    }

    #[test]
    fn test_settings_loader_editor_open_non_existent_file() {
        let path = PathBuf::from("non_existent_file.toml");
        let res = SettingsLoaderEditor::open(&path);
        assert!(res.is_err());
        if let Err(EditorError::IoError(_)) = res {
            // Expected error
        } else {
            panic!("Expected IoError, got {:?}", res);
        }
    }

    #[test]
    fn test_settings_loader_editor_open_invalid_content() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"key = \"value").unwrap(); // Invalid TOML
        file.flush().unwrap();
        let path = file.path().with_extension("toml");
        fs::rename(file.path(), &path).unwrap();

        let res = SettingsLoaderEditor::open(&path);
        assert!(res.is_err());
        if let Err(EditorError::ParseError(_)) = res {
            // Expected error
        } else {
            panic!("Expected ParseError, got {:?}", res);
        }
    }

    #[test]
    fn test_settings_loader_editor_create_toml() {
        use tempfile::tempdir;
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("test.toml");

        let editor = SettingsLoaderEditor::create(&path, ConfigFormat::Toml).unwrap();
        if let Editor::Toml(toml_editor) = editor {
            assert!(!toml_editor.is_dirty()); // Should be saved
            let content = fs::read_to_string(&path).unwrap();
            assert_eq!(content, ""); // Empty TOML doc
        } else {
            panic!("Expected Toml editor");
        }
    }

    #[test]
    fn test_settings_loader_editor_create_json() {
        use tempfile::tempdir;
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("test.json");

        let editor = SettingsLoaderEditor::create(&path, ConfigFormat::Json).unwrap();
        if let Editor::Json(json_editor) = editor {
            assert!(!json_editor.is_dirty()); // Should be saved
            let content = fs::read_to_string(&path).unwrap();
            assert_eq!(content, "{}"); // Empty JSON object
        } else {
            panic!("Expected Json editor");
        }
    }

    #[test]
    fn test_settings_loader_editor_create_yaml() {
        use tempfile::tempdir;
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("test.yaml");

        let editor = SettingsLoaderEditor::create(&path, ConfigFormat::Yaml).unwrap();
        if let Editor::Yaml(yaml_editor) = editor {
            assert!(!yaml_editor.is_dirty()); // Should be saved
            let content = fs::read_to_string(&path).unwrap();
            assert_eq!(content, "{}\n"); // Empty YAML mapping
        } else {
            panic!("Expected Yaml editor");
        }
    }

    #[test]
    fn test_settings_loader_editor_create_permissions_error() {
        // Create a directory that we don't have write permissions to (not easily portable)
        // For testing purposes, we can try to create a file in a non-existent subdirectory
        // which will result in an IoError::NotFound or similar.
        let path = PathBuf::from("/nonexistent_dir/test.toml");
        let res = SettingsLoaderEditor::create(&path, ConfigFormat::Toml);
        assert!(res.is_err());
        if let Err(EditorError::IoError(e)) = res {
            assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
        } else {
            panic!("Expected IoError (NotFound), got {:?}", res);
        }
    }
}
