//! YAML-specific configuration layer editor.

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};
use serde_yaml::Value;

use super::{EditorError, LayerEditor};

/// YAML-specific implementation of the `LayerEditor` trait.
#[derive(Debug)]
pub struct YamlLayerEditor {
    path: PathBuf,
    document: RwLock<Value>,
    dirty: RwLock<bool>,
}

impl YamlLayerEditor {
    /// Opens an existing YAML file and parses it into a `YamlLayerEditor`.
    pub fn open(path: &Path) -> Result<Self, EditorError> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read YAML file: {}", path.display()))
            .map_err(|e| EditorError::IoError(io::Error::other(e.to_string())))?;

        let document: Value = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse YAML file: {}", path.display()))
            .map_err(|e| EditorError::parse_error(e.to_string()))?;

        Ok(Self {
            path: path.to_path_buf(),
            document: RwLock::new(document),
            dirty: RwLock::new(false),
        })
    }

    /// Creates a new, empty YAML file editor.
    pub fn create(path: &Path) -> Result<Self, EditorError> {
        let document = Value::Mapping(serde_yaml::Mapping::new());
        let mut editor = Self {
            path: path.to_path_buf(),
            document: RwLock::new(document),
            dirty: RwLock::new(true), // New file is dirty until saved
        };
        // Create the file on the filesystem
        editor.save()?;
        Ok(editor)
    }

    // Helper to navigate to a value in the YAML document using a dotted path.
    fn get_value_mut<'a>(doc: &'a mut Value, key: &str) -> Result<&'a mut Value, EditorError> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current_value = doc;

        for (i, part) in parts.iter().enumerate() {
            let part_value = Value::String(part.to_string());
            if i == parts.len() - 1 {
                return Ok(current_value
                    .as_mapping_mut()
                    .ok_or_else(|| {
                        EditorError::InvalidPath(format!("Path segment '{}' is not a mapping", parts[i - 1]))
                    })?
                    .entry(part_value)
                    .or_insert(Value::Null));
            } else {
                current_value = current_value
                    .as_mapping_mut()
                    .ok_or_else(|| EditorError::InvalidPath(format!("Path segment '{}' is not a mapping", part)))?
                    .entry(part_value)
                    .or_insert(Value::Mapping(serde_yaml::Mapping::new()));
            }
        }
        Ok(current_value)
    }
}

impl LayerEditor for YamlLayerEditor {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let doc = self.document.read();
        let parts: Vec<&str> = key.split('.').collect();
        let mut current_value: &Value = &doc;

        for part in parts {
            current_value = current_value.as_mapping()?.get(Value::String(part.to_string()))?;
        }

        serde_yaml::from_value(current_value.clone()).ok()
    }

    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), EditorError> {
        let mut doc = self.document.write();
        let target_value = YamlLayerEditor::get_value_mut(&mut doc, key)?;

        *target_value = serde_yaml::to_value(value)
            .map_err(|e| EditorError::serialization_error(format!("Failed to serialize value: {}", e)))?;

        *self.dirty.write() = true;
        Ok(())
    }

    fn unset(&mut self, key: &str) -> Result<(), EditorError> {
        let mut doc = self.document.write();
        let parts: Vec<&str> = key.split('.').collect();

        if parts.is_empty() {
            return Err(EditorError::InvalidPath("Empty key provided for unset".to_string()));
        }

        let target_key = parts.last().unwrap();
        let parent_path_str = parts[..parts.len() - 1].join(".");

        let parent_value: &mut Value = if parent_path_str.is_empty() {
            &mut doc
        } else {
            YamlLayerEditor::get_value_mut(&mut doc, &parent_path_str)
                .map_err(|_| EditorError::key_not_found(&parent_path_str))?
        };

        if let Some(mapping) = parent_value.as_mapping_mut() {
            if mapping.remove(Value::String(target_key.to_string())).is_some() {
                *self.dirty.write() = true;
                Ok(())
            } else {
                Err(EditorError::key_not_found(key))
            }
        } else {
            Err(EditorError::KeyNotFound(format!(
                "Parent path '{}' is not a mapping",
                parent_path_str
            )))
        }
    }

    fn keys(&self) -> Vec<String> {
        let doc = self.document.read();
        doc.as_mapping()
            .map(|mapping| mapping.keys().filter_map(|k| k.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default()
    }

    fn is_dirty(&self) -> bool {
        *self.dirty.read()
    }

    fn save(&mut self) -> Result<(), EditorError> {
        let path = &self.path;
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("Invalid path: no parent directory"))
            .map_err(EditorError::IoError)?;
        let temp_file_name = format!(".{}.tmp", path.file_name().unwrap().to_string_lossy());
        let temp_path = parent.join(temp_file_name);

        let content = serde_yaml::to_string(&*self.document.read())
            .map_err(|e| EditorError::serialization_error(format!("Failed to serialize document: {}", e)))?;

        let mut file = fs::File::create(&temp_path)
            .with_context(|| format!("Failed to create temporary file: {}", temp_path.display()))
            .map_err(|e| EditorError::IoError(io::Error::other(e.to_string())))?;
        file.write_all(content.as_bytes())
            .with_context(|| format!("Failed to write to temporary file: {}", temp_path.display()))
            .map_err(|e| EditorError::IoError(io::Error::other(e.to_string())))?;
        file.sync_all()
            .with_context(|| format!("Failed to sync temporary file: {}", temp_path.display()))
            .map_err(|e| EditorError::IoError(io::Error::other(e.to_string())))?;

        fs::rename(&temp_path, path)
            .with_context(|| format!("Failed to rename temporary file to: {}", path.display()))
            .map_err(|e| EditorError::IoError(io::Error::other(e.to_string())))?;

        *self.dirty.write() = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_yaml_editor_open_and_get() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            database:
                host: localhost
                port: 5432
                enabled: true
            server:
                port: 8080
            "#,
        )
        .unwrap();

        let editor = YamlLayerEditor::open(path).unwrap();

        assert_eq!(editor.get::<String>("database.host"), Some("localhost".to_string()));
        assert_eq!(editor.get::<u16>("database.port"), Some(5432));
        assert_eq!(editor.get::<bool>("database.enabled"), Some(true));
        assert_eq!(editor.get::<u16>("server.port"), Some(8080));
        assert_eq!(editor.get::<String>("nonexistent.key"), None);
    }

    #[test]
    fn test_yaml_editor_set_and_save() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            database:
                host: localhost
                port: 5432
                enabled: true
            "#,
        )
        .unwrap();

        let mut editor = YamlLayerEditor::open(path).unwrap();
        assert!(!editor.is_dirty());

        editor.set("database.host", "new_host.com").unwrap();
        editor.set("database.port", 1234).unwrap();
        editor.set("server.timeout", 300_u32).unwrap(); // Add new key
        assert!(editor.is_dirty());

        editor.save().unwrap();
        assert!(!editor.is_dirty());

        let new_content = fs::read_to_string(path).unwrap();
        assert!(new_content.contains("host: new_host.com"));
        assert!(new_content.contains("port: 1234"));
        assert!(new_content.contains("server:"));
        assert!(new_content.contains("timeout: 300"));

        let editor_reopened = YamlLayerEditor::open(path).unwrap();
        assert_eq!(
            editor_reopened.get::<String>("database.host"),
            Some("new_host.com".to_string())
        );
        assert_eq!(editor_reopened.get::<u16>("database.port"), Some(1234));
        assert_eq!(editor_reopened.get::<u32>("server.timeout"), Some(300));
    }

    #[test]
    fn test_yaml_editor_create() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::remove_file(path).unwrap();

        let editor = YamlLayerEditor::create(path).unwrap();
        assert!(!editor.is_dirty());

        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "{}\n"); // Initially empty YAML mapping with newline
    }

    #[test]
    fn test_yaml_editor_unset() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            debug: true
            version: "1.0"
            database:
                host: localhost
                port: 5432
                ssl: true
            "#,
        )
        .unwrap();

        let mut editor = YamlLayerEditor::open(path).unwrap();
        assert!(!editor.is_dirty());

        editor.unset("debug").unwrap();
        assert!(editor.is_dirty());
        assert_eq!(editor.get::<bool>("debug"), None);

        editor.unset("database.port").unwrap();
        assert!(editor.is_dirty());
        assert_eq!(editor.get::<u16>("database.port"), None);

        assert!(editor.unset("nonexistent_key").is_err());
        assert!(editor.unset("database.nonexistent_key").is_err());

        editor.save().unwrap();
        assert!(!editor.is_dirty());

        let new_content = fs::read_to_string(path).unwrap();
        assert!(!new_content.contains("debug: true"));
        assert!(!new_content.contains("port: 5432"));
        assert!(new_content.contains("host: localhost"));
        assert!(new_content.contains("ssl: true"));
    }

    #[test]
    fn test_yaml_editor_keys() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            key1: value1
            section1:
                key2: value2
            section2:
                key3: value3
            "#,
        )
        .unwrap();

        let editor = YamlLayerEditor::open(path).unwrap();
        let keys = editor.keys();
        let mut sorted_keys = keys;
        sorted_keys.sort();
        assert_eq!(sorted_keys, vec!["key1", "section1", "section2"]);
    }
}
