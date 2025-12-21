//! TOML-specific configuration layer editor.

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::Context;
use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Serialize};
use toml_edit::{DocumentMut, Item, Table, Value};

use super::{EditorError, LayerEditor};
use toml;

/// TOML-specific implementation of the `LayerEditor` trait.
#[derive(Debug)]
pub struct TomlLayerEditor {
    path: PathBuf,
    document: RwLock<DocumentMut>, // Use RwLock for interior mutability
    dirty: RwLock<bool>,
}

impl TomlLayerEditor {
    /// Opens an existing TOML file and parses it into a `TomlLayerEditor`.
    pub fn open(path: &Path) -> Result<Self, EditorError> {
        let content = fs::read_to_string(path).map_err(EditorError::IoError)?;

        let document = content
            .parse::<DocumentMut>()
            .with_context(|| format!("Failed to parse TOML file: {}", path.display()))
            .map_err(|e| EditorError::parse_error(e.to_string()))?;

        Ok(Self {
            path: path.to_path_buf(),
            document: RwLock::new(document),
            dirty: RwLock::new(false),
        })
    }

    /// Creates a new, empty TOML file editor.
    pub fn create(path: &Path) -> Result<Self, EditorError> {
        let document = DocumentMut::new();
        let mut editor = Self {
            path: path.to_path_buf(),
            document: RwLock::new(document),
            dirty: RwLock::new(true), // New file is dirty until saved
        };
        // Create the file on the filesystem
        editor.save()?;
        Ok(editor)
    }

    // Helper to get an item from the document using a dotted path
    fn get_item<'a>(&self, doc: &'a DocumentMut, key: &str) -> Option<&'a Item> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current_item: Option<&Item> = Some(doc.as_item());

        for (i, part) in parts.iter().enumerate() {
            if let Some(Item::Table(table)) = current_item {
                current_item = table.get(part);
            } else if i < parts.len() {
                // If we expect more parts but current_item is not a table, then key path is invalid
                return None;
            }
        }
        current_item
    }

    // Helper to get a mutable item from the document using a dotted path, creating tables if necessary
    fn get_item_mut<'a>(doc: &'a mut DocumentMut, key: &str) -> Result<&'a mut Item, EditorError> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current_item: &mut Item = doc.as_item_mut();

        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part, this is the item we want to return
                if current_item.is_table() || current_item.is_table_like() {
                    // It's a table, so we can insert a new item or get an existing one
                    let table = current_item
                        .as_table_mut()
                        .ok_or_else(|| EditorError::InvalidPath(format!("Path segment '{}' is not a table", key)))?;
                    return Ok(table.entry(part).or_insert(Item::None));
                } else {
                    // Not a table, can't descend further
                    return Err(EditorError::InvalidPath(format!(
                        "Path segment '{}' is not a table",
                        parts[i - 1]
                    )));
                }
            } else {
                // Not the last part, ensure it's a table and descend
                let table = current_item.as_table_mut().ok_or_else(|| {
                    EditorError::InvalidPath(format!("Path segment '{}' is not a table", parts[i - 1]))
                })?;
                current_item = table.entry(part).or_insert(Item::Table(Table::new()));
                if !current_item.is_table() && !current_item.is_table_like() {
                    return Err(EditorError::InvalidPath(format!(
                        "Path segment '{}' is not a table",
                        part
                    )));
                }
            }
        }
        Ok(current_item)
    }

    // Helper to remove an item from the document using a dotted path
    fn remove_item(&mut self, key: &str) -> Result<(), EditorError> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut doc = self.document.write();

        if parts.is_empty() || parts.iter().any(|p| p.is_empty()) {
            return Err(EditorError::InvalidPath("Empty key provided for unset".to_string()));
        }

        let target_key = parts.last().unwrap();
        let parent_path = parts[..parts.len() - 1].join(".");

        let current_table = if parent_path.is_empty() {
            doc.as_table_mut()
        } else {
            match Self::get_item_mut(&mut doc, &parent_path) {
                Ok(item) => item
                    .as_table_mut()
                    .ok_or_else(|| EditorError::InvalidPath(format!("Parent path '{}' is not a table", parent_path)))?,
                Err(e) => return Err(e),
            }
        };

        if current_table.remove(target_key).is_some() {
            *self.dirty.write() = true;
            Ok(())
        } else {
            Err(EditorError::key_not_found(key))
        }
    }
}

impl LayerEditor for TomlLayerEditor {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let doc = self.document.read();
        self.get_item(&doc, key).and_then(|item| {
            let item_as_str = item.to_string();
            // toml::from_str works well for values, but for tables it needs to be valid TOML.
            // If item is a table, we should use its string representation which is valid TOML.
            let toml_string = if item.is_table() {
                item_as_str
            } else {
                // If it is a value, we need to wrap it in a key-value pair to be valid TOML for parsing.
                format!("key = {}", item_as_str)
            };

            let wrapped_val: Result<toml::Value, _> = toml::from_str(&toml_string);

            wrapped_val.ok().and_then(|v| {
                if item.is_table() {
                    v.try_into().ok()
                } else {
                    v.get("key").and_then(|v_inner| v_inner.clone().try_into().ok())
                }
            })
        })
    }

    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), EditorError> {
        let mut doc = self.document.write();
        let item = Self::get_item_mut(&mut doc, key)?;

        let toml_value = toml::Value::try_from(value)
            .map_err(|e| EditorError::serialization_error(format!("Failed to serialize value: {}", e)))?;

        let mut new_value_item = toml_value.to_string().parse::<Value>().map_err(|e| {
            EditorError::serialization_error(format!(
                "Failed to parse serialized value back to toml_edit::Value: {}",
                e
            ))
        })?;

        if let Some(v) = item.as_value() {
            new_value_item.decor_mut().clone_from(v.decor());
        }

        *item = Item::Value(new_value_item);

        *self.dirty.write() = true;
        Ok(())
    }

    fn unset(&mut self, key: &str) -> Result<(), EditorError> {
        self.remove_item(key)
    }

    fn keys(&self) -> Vec<String> {
        let doc = self.document.read();
        doc.as_table().iter().map(|(k, _)| k.to_string()).collect()
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

        let content = self.document.read().to_string();

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
    fn test_toml_editor_open_and_get() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            [database]
            host = "localhost"
            port = 5432
            enabled = true

            [server]
            port = 8080
            "#,
        )
        .unwrap();

        let editor = TomlLayerEditor::open(path).unwrap();

        assert_eq!(editor.get::<String>("database.host"), Some("localhost".to_string()));
        assert_eq!(editor.get::<u16>("database.port"), Some(5432));
        assert_eq!(editor.get::<bool>("database.enabled"), Some(true));
        assert_eq!(editor.get::<u16>("server.port"), Some(8080));
        assert_eq!(editor.get::<String>("nonexistent.key"), None);
    }

    #[test]
    fn test_toml_editor_set_and_save() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            [database]
            host = "localhost"
            port = 5432 # This is a comment
            enabled = true
            "#,
        )
        .unwrap();

        let mut editor = TomlLayerEditor::open(path).unwrap();
        assert!(!editor.is_dirty());

        editor.set("database.host", "new_host.com").unwrap();
        editor.set("database.port", 1234).unwrap();
        editor.set("server.timeout", 300_u32).unwrap(); // Add new key
        assert!(editor.is_dirty());

        editor.save().unwrap();
        assert!(!editor.is_dirty());

        let new_content = fs::read_to_string(path).unwrap();
        assert!(new_content.contains("host = \"new_host.com\""));
        assert!(new_content.contains("port = 1234"));
        assert!(new_content.contains("# This is a comment"));
        assert!(new_content.contains("[server]"));
        assert!(new_content.contains("timeout = 300"));

        let editor_reopened = TomlLayerEditor::open(path).unwrap();
        assert_eq!(
            editor_reopened.get::<String>("database.host"),
            Some("new_host.com".to_string())
        );
        assert_eq!(editor_reopened.get::<u16>("database.port"), Some(1234));
        assert_eq!(editor_reopened.get::<u32>("server.timeout"), Some(300));
    }

    #[test]
    fn test_toml_editor_create() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        // Delete the file created by NamedTempFile to ensure create works from scratch
        fs::remove_file(path).unwrap();

        let editor = TomlLayerEditor::create(path).unwrap();
        assert!(!editor.is_dirty()); // Should be false after initial save

        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, ""); // Initially empty
    }

    #[test]
    fn test_toml_editor_unset() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            debug = true
            # Top-level comment
            version = "1.0"

            [database] # Database config
            host = "localhost"
            port = 5432
            ssl = true
            "#,
        )
        .unwrap();

        let mut editor = TomlLayerEditor::open(path).unwrap();
        assert!(!editor.is_dirty());

        // Unset top-level key
        editor.unset("debug").unwrap();
        assert!(editor.is_dirty());
        assert_eq!(editor.get::<bool>("debug"), None);

        // Unset nested key
        editor.unset("database.port").unwrap();
        assert!(editor.is_dirty());
        assert_eq!(editor.get::<u16>("database.port"), None);

        // Try to unset non-existent key
        assert!(editor.unset("nonexistent_key").is_err());
        assert!(editor.unset("database.nonexistent_key").is_err());

        editor.save().unwrap();
        assert!(!editor.is_dirty());

        let new_content = fs::read_to_string(path).unwrap();
        // Verify 'debug' is gone
        assert!(!new_content.contains("debug = true"));
        // Verify 'port' is gone, but other database settings and comment preserved
        assert!(!new_content.contains("port = 5432"));
        assert!(new_content.contains("host = \"localhost\""));
        assert!(new_content.contains("ssl = true"));
        assert!(new_content.contains("# Database config")); // Parent comment preserved
        assert!(new_content.contains("# Top-level comment")); // Top-level comment preserved
    }

    #[test]
    fn test_toml_editor_keys() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            key1 = "value1"
            [section1]
            key2 = "value2"
            [section2]
            key3 = "value3"
            "#,
        )
        .unwrap();

        let editor = TomlLayerEditor::open(path).unwrap();
        let keys = editor.keys();
        let mut sorted_keys = keys;
        sorted_keys.sort();
        assert_eq!(sorted_keys, vec!["key1", "section1", "section2"]);
    }

    #[test]
    fn test_toml_editor_invalid_path() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(path, r#"a = "not_a_table""#).unwrap();

        let mut editor = TomlLayerEditor::open(path).unwrap();
        let res = editor.set("a.b", 123);
        assert!(res.is_err());
        if let Err(EditorError::InvalidPath(msg)) = res {
            assert!(msg.contains("Path segment 'a' is not a table"));
        } else {
            panic!("Expected InvalidPath error, got {:?}", res);
        }
    }

    #[test]
    fn test_toml_editor_unset_empty_key() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(path, r#"key = "value""#).unwrap();

        let mut editor = TomlLayerEditor::open(path).unwrap();
        let res = editor.unset("");
        assert!(res.is_err());
        if let Err(EditorError::InvalidPath(msg)) = res {
            assert!(msg.contains("Empty key provided for unset"));
        } else {
            panic!("Expected InvalidPath error for empty key, got {:?}", res);
        }
    }

    #[test]
    fn test_toml_editor_unset_parent_not_table() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            [section]
            key = "value"
            non_table = 123
        "#,
        )
        .unwrap();

        let mut editor = TomlLayerEditor::open(path).unwrap();
        // Attempt to unset a key inside a non-table value
        let res = editor.unset("section.non_table.nested");
        assert!(res.is_err());
        if let Err(EditorError::InvalidPath(msg)) = res {
            assert!(msg.contains("is not a table"), "Got: {}", msg);
        } else {
            panic!("Expected InvalidPath error for non-table parent, got {:?}", res);
        }
    }

    #[test]
    fn test_get_item_mut_intermediate_non_table() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            [a]
            b = "string_value"
        "#,
        )
        .unwrap();
        let mut editor = TomlLayerEditor::open(path).unwrap();
        // Trying to get a mutable item where 'b' is a string, not a table
        let res = editor.set("a.b.c", 1);
        assert!(res.is_err());
        if let Err(EditorError::InvalidPath(msg)) = res {
            assert!(msg.contains("Path segment 'b' is not a table"));
        } else {
            panic!("Expected InvalidPath error for intermediate non-table, got {:?}", res);
        }
    }

    #[test]
    fn test_get_item_non_table_intermediate() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            [x]
            y = "string_value"
        "#,
        )
        .unwrap();
        let editor = TomlLayerEditor::open(path).unwrap();
        // Trying to get a nested item where 'y' is a string, not a table
        let doc = editor.document.read();
        let item = editor.get_item(&doc, "x.y.z");
        assert!(item.is_none());
    }

    #[test]
    fn test_get_table_as_primitive() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            [db]
            host = "localhost"
        "#,
        )
        .unwrap();
        let editor = TomlLayerEditor::open(path).unwrap();
        let result: Option<String> = editor.get("db"); // Trying to get a table as a string
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_primitive_as_table() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(
            path,
            r#"
            value = "test"
        "#,
        )
        .unwrap();
        let editor = TomlLayerEditor::open(path).unwrap();
        let result: Option<String> = editor.get("value"); // Trying to get a string value
        assert_eq!(result, Some("test".to_string()));
    }

    #[test]
    fn test_toml_editor_is_dirty_persistence() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();
        fs::write(path, r#""#).unwrap();

        let mut editor = TomlLayerEditor::open(path).unwrap();
        assert!(!editor.is_dirty());

        editor.set("key", "val").unwrap();
        assert!(editor.is_dirty());

        editor.save().unwrap();
        assert!(!editor.is_dirty());

        editor.set("key", "val").unwrap();
        assert!(editor.is_dirty());
    }
}
