//! Configuration format detection and representation.

use std::path::Path;

/// Supported configuration file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    /// TOML format (.toml extension)
    Toml,
    /// JSON format (.json extension)
    Json,
    /// YAML format (.yaml or .yml extensions)
    Yaml,
}

impl ConfigFormat {
    /// Detect configuration format from file path extension.
    ///
    /// Returns the detected format based on file extension, or `None` if the
    /// extension is not recognized or the path has no extension.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::path::Path;
    /// use settings_loader::editor::ConfigFormat;
    ///
    /// assert_eq!(
    ///     ConfigFormat::from_path(Path::new("settings.toml")),
    ///     Some(ConfigFormat::Toml)
    /// );
    ///
    /// assert_eq!(
    ///     ConfigFormat::from_path(Path::new("config.json")),
    ///     Some(ConfigFormat::Json)
    /// );
    ///
    /// assert_eq!(
    ///     ConfigFormat::from_path(Path::new("settings.yaml")),
    ///     Some(ConfigFormat::Yaml)
    /// );
    ///
    /// assert_eq!(
    ///     ConfigFormat::from_path(Path::new("settings.txt")),
    ///     None
    /// );
    /// ```
    pub fn from_path(path: &Path) -> Option<ConfigFormat> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext_str| match ext_str.to_lowercase().as_str() {
                "toml" => Some(ConfigFormat::Toml),
                "json" => Some(ConfigFormat::Json),
                "yaml" | "yml" => Some(ConfigFormat::Yaml),
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_path_toml() {
        assert_eq!(
            ConfigFormat::from_path(Path::new("settings.toml")),
            Some(ConfigFormat::Toml)
        );
    }

    #[test]
    fn test_from_path_json() {
        assert_eq!(
            ConfigFormat::from_path(Path::new("settings.json")),
            Some(ConfigFormat::Json)
        );
    }

    #[test]
    fn test_from_path_yaml() {
        assert_eq!(
            ConfigFormat::from_path(Path::new("settings.yaml")),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::from_path(Path::new("settings.yml")),
            Some(ConfigFormat::Yaml)
        );
    }

    #[test]
    fn test_from_path_case_insensitive() {
        assert_eq!(
            ConfigFormat::from_path(Path::new("settings.TOML")),
            Some(ConfigFormat::Toml)
        );
        assert_eq!(
            ConfigFormat::from_path(Path::new("settings.YaMl")),
            Some(ConfigFormat::Yaml)
        );
    }

    #[test]
    fn test_from_path_unknown_extension() {
        assert_eq!(ConfigFormat::from_path(Path::new("settings.txt")), None);
        assert_eq!(ConfigFormat::from_path(Path::new("settings.ini")), None);
    }

    #[test]
    fn test_from_path_no_extension() {
        assert_eq!(ConfigFormat::from_path(Path::new("settings")), None);
    }
}
