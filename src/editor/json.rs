//! JSON-specific configuration layer editor.

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};

use super::{EditorError, LayerEditor};

/// JSON-specific implementation of the `LayerEditor` trait.
#[derive(Debug)]
pub struct JsonLayerEditor {
    path: PathBuf,
    document: RwLock<Value>,
    dirty: RwLock<bool>,
}

impl JsonLayerEditor {
    /// Opens an existing JSON file and parses it into a `JsonLayerEditor`.
    pub fn open(path: &Path) -> Result<Self, EditorError> {
        let content = fs::read_to_string(path).map_err(EditorError::IoError)?;

        let document: Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON file: {}", path.display()))
            .map_err(|e| EditorError::parse_error(e.to_string()))?;

        Ok(Self {
            path: path.to_path_buf(),
            document: RwLock::new(document),
            dirty: RwLock::new(false),
        })
    }

    /// Creates a new, empty JSON file editor.
    pub fn create(path: &Path) -> Result<Self, EditorError> {
        let document = Value::Object(Map::new());
        let mut editor = Self {
            path: path.to_path_buf(),
            document: RwLock::new(document),
            dirty: RwLock::new(true), // New file is dirty until saved
        };
        // Create the file on the filesystem
        editor.save()?;
        Ok(editor)
    }

    // Helper to navigate to a value in the JSON document using a dotted path.
    fn get_value_mut<'a>(doc: &'a mut Value, key: &str) -> Result<&'a mut Value, EditorError> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current_value = doc;

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                return Ok(current_value
                    .as_object_mut()
                    .ok_or_else(|| {
                        EditorError::InvalidPath(format!("Path segment '{}' is not an object", parts[i - 1]))
                    })?
                    .entry(*part)
                    .or_insert(Value::Null));
            } else {
                current_value = current_value
                    .as_object_mut()
                    .ok_or_else(|| EditorError::InvalidPath(format!("Path segment '{}' is not an object", part)))?
                    .entry(*part)
                    .or_insert(Value::Object(Map::new()));
            }
        }
        Ok(current_value)
    }
}

impl LayerEditor for JsonLayerEditor {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let doc = self.document.read();
        let parts: Vec<&str> = key.split('.').collect();
        let mut current_value: &Value = &doc;

        for part in parts {
            current_value = current_value.as_object()?.get(part)?;
        }

        serde_json::from_value(current_value.clone()).ok()
    }

    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), EditorError> {
        let mut doc = self.document.write();
        let target_value = JsonLayerEditor::get_value_mut(&mut doc, key)?;

        *target_value = serde_json::to_value(value)
            .map_err(|e| EditorError::serialization_error(format!("Failed to serialize value: {}", e)))?;

        *self.dirty.write() = true;
        Ok(())
    }

    fn unset(&mut self, key: &str) -> Result<(), EditorError> {
        let mut doc = self.document.write();
        let parts: Vec<&str> = key.split('.').collect();

        if parts.is_empty() || parts.iter().any(|p| p.is_empty()) {
            return Err(EditorError::InvalidPath("Empty key provided for unset".to_string()));
        }

        let target_key = parts.last().unwrap();
        let parent_path_str = parts[..parts.len() - 1].join(".");

        let parent_value: &mut Value = if parent_path_str.is_empty() {
            &mut doc
        } else {
            JsonLayerEditor::get_value_mut(&mut doc, &parent_path_str)
                .map_err(|_| EditorError::key_not_found(&parent_path_str))?
        };

        if let Some(obj) = parent_value.as_object_mut() {
            if obj.remove(*target_key).is_some() {
                *self.dirty.write() = true;
                Ok(())
            } else {
                Err(EditorError::key_not_found(key))
            }
        } else {
            Err(EditorError::InvalidPath(format!(
                "Parent path '{}' is not an object",
                parent_path_str
            )))
        }
    }

    fn keys(&self) -> Vec<String> {
        let doc = self.document.read();
        doc.as_object()
            .map(|obj| obj.keys().map(|k| k.to_string()).collect())
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

        let content = serde_json::to_string_pretty(&*self.document.read())
            .map_err(|e| EditorError::serialization_error(format!("Failed to serialize document: {}", e)))?;

        let mut file = fs::File::create(&temp_path).map_err(EditorError::IoError)?;
        file.write_all(content.as_bytes()).map_err(EditorError::IoError)?;
        file.sync_all().map_err(EditorError::IoError)?;

        fs::rename(&temp_path, path).map_err(EditorError::IoError)?;

        *self.dirty.write() = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_editor_open_and_get() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"{
                "database": {
                    "host": "localhost",
                    "port": 5432,
                    "enabled": true
                },
                "server": {
                    "port": 8080
                }
            }"#,
        )
        .unwrap();

        let editor = JsonLayerEditor::open(path).unwrap();

        assert_eq!(editor.get::<String>("database.host"), Some("localhost".to_string()));
        assert_eq!(editor.get::<u16>("database.port"), Some(5432));
        assert_eq!(editor.get::<bool>("database.enabled"), Some(true));
        assert_eq!(editor.get::<u16>("server.port"), Some(8080));
        assert_eq!(editor.get::<String>("nonexistent.key"), None);
    }

    #[test]
    fn test_json_editor_set_and_save() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"{
                "database": {
                    "host": "localhost",
                    "port": 5432,
                    "enabled": true
                }
            }"#,
        )
        .unwrap();

        let mut editor = JsonLayerEditor::open(path).unwrap();
        assert!(!editor.is_dirty());

        editor.set("database.host", "new_host.com").unwrap();
        editor.set("database.port", 1234).unwrap();
        editor.set("server.timeout", 300_u32).unwrap(); // Add new key
        assert!(editor.is_dirty());

        editor.save().unwrap();
        assert!(!editor.is_dirty());

        let new_content = fs::read_to_string(path).unwrap();
        let new_json: serde_json::Value = serde_json::from_str(&new_content).unwrap();

        assert_eq!(
            new_json["database"]["host"],
            serde_json::Value::String("new_host.com".to_string())
        );
        assert_eq!(new_json["database"]["port"], serde_json::Value::from(1234));
        assert_eq!(new_json["server"]["timeout"], serde_json::Value::from(300));

        let editor_reopened = JsonLayerEditor::open(path).unwrap();
        assert_eq!(
            editor_reopened.get::<String>("database.host"),
            Some("new_host.com".to_string())
        );
        assert_eq!(editor_reopened.get::<u16>("database.port"), Some(1234));
        assert_eq!(editor_reopened.get::<u32>("server.timeout"), Some(300));
    }

    #[test]
    fn test_json_editor_create() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::remove_file(path).unwrap();

        let editor = JsonLayerEditor::create(path).unwrap();
        assert!(!editor.is_dirty());

        let content = fs::read_to_string(path).unwrap();
        let created_json: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(created_json.is_object());
        assert!(created_json.as_object().unwrap().is_empty()); // Empty JSON object with newline
    }

    #[test]
    fn test_json_editor_unset() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"{
                "debug": true,
                "version": "1.0",
                "database": {
                    "host": "localhost",
                    "port": 5432,
                    "ssl": true
                }
            }"#,
        )
        .unwrap();

        let mut editor = JsonLayerEditor::open(path).unwrap();
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
        let new_json: serde_json::Value = serde_json::from_str(&new_content).unwrap();

        assert!(!new_json["debug"].is_boolean()); // `debug` should be removed
        assert!(new_json["version"] == "1.0");

        assert!(new_json["database"]["host"] == "localhost");
        assert!(!new_json["database"]["port"].is_number()); // `port` should be removed
        assert!(new_json["database"]["ssl"] == true);
    }

    #[test]
    fn test_json_editor_keys() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"{
                "key1": "value1",
                "section1": {
                    "key2": "value2"
                },
                "section2": {
                    "key3": "value3"
                }
            }"#,
        )
        .unwrap();

        let editor = JsonLayerEditor::open(path).unwrap();
        let keys = editor.keys();
        let mut sorted_keys = keys;
        sorted_keys.sort();
        assert_eq!(sorted_keys, vec!["key1", "section1", "section2"]);
    }

    #[test]
    fn test_json_editor_invalid_path() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(path, r#"{"a": "not_an_object", "b": {"c": "value"}}"#).unwrap();

        let mut editor = JsonLayerEditor::open(path).unwrap();

        // Test setting a nested key on a non-object value
        let res = editor.set("a.b", 123);
        assert!(res.is_err());
        if let Err(EditorError::InvalidPath(msg)) = res {
            assert!(msg.contains("Path segment 'a' is not an object"));
        } else {
            panic!("Expected InvalidPath error, got {:?}", res);
        }

        // Test unsetting a key whose parent is not an object
        let res = editor.unset("b.c.d");
        assert!(res.is_err());
        if let Err(EditorError::InvalidPath(msg)) = res {
            assert!(msg.contains("Parent path 'b.c' is not an object"));
        } else {
            panic!("Expected InvalidPath error for non-object parent, got {:?}", res);
        }
    }

    #[test]
    fn test_json_editor_unset_empty_key() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(path, r#"{"key": "value"}"#).unwrap();

        let mut editor = JsonLayerEditor::open(path).unwrap();
        let res = editor.unset("");
        assert!(res.is_err());
        if let Err(EditorError::InvalidPath(msg)) = res {
            assert!(msg.contains("Empty key provided for unset"));
        } else {
            panic!("Expected InvalidPath error for empty key, got {:?}", res);
        }
    }

    #[test]
    fn test_json_editor_get_value_mut_root_key() {
        let mut doc = serde_json::from_str(r#"{"key1": "value1"}"#).unwrap();
        let key_ref = JsonLayerEditor::get_value_mut(&mut doc, "key1").unwrap();
        assert_eq!(key_ref, &mut serde_json::Value::String("value1".to_string()));
    }

    #[test]
    fn test_json_editor_get_value_mut_new_root_key() {
        let mut doc = serde_json::Value::Object(serde_json::Map::new());
        let key_ref = JsonLayerEditor::get_value_mut(&mut doc, "new_key").unwrap();
        assert_eq!(key_ref, &mut serde_json::Value::Null);
    }

    #[test]
    fn test_json_editor_is_dirty_persistence() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(path, r#"{}"#).unwrap();

        let mut editor = JsonLayerEditor::open(path).unwrap();
        assert!(!editor.is_dirty());

        editor.set("key", "val").unwrap();
        assert!(editor.is_dirty());

        editor.save().unwrap();
        assert!(!editor.is_dirty());

        editor.set("key", "val").unwrap(); // Setting same value still marks dirty in current implementation
        assert!(editor.is_dirty());
    }
}
