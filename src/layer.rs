//! Explicit configuration layering API.
//!
//! Allows applications to compose configuration sources with clear, explicit
//! precedence. Later layers override earlier layers.
//!
//! # Layer Types
//!
//! - `Path` - Load from explicit file path
//! - `EnvVar` - Load from path specified in environment variable
//! - `EnvSearch` - Search for environment-specific configuration files
//! - `Secrets` - Load from secrets file
//! - `EnvVars` - Load from system environment variables with prefix
//!
//! # Example
//!
//! ```rust
//! use settings_loader::{LayerBuilder, SettingsLoader, LoadingOptions, SettingsError};
//! use std::path::PathBuf;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
//! struct MyOptions;
//!
//! impl LoadingOptions for MyOptions {
//!     type Error = SettingsError;
//!
//!     fn config_path(&self) -> Option<PathBuf> { None }
//!     fn secrets_path(&self) -> Option<PathBuf> { None }
//!     fn implicit_search_paths(&self) -> Vec<PathBuf> { Vec::new() }
//!
//!     fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
//!         builder
//!             .with_path(PathBuf::from("config.yaml"))
//!             .with_secrets(PathBuf::from("secrets.yaml"))
//!             .with_env_vars("APP", "__")
//!     }
//! }
//!
//! #[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
//! struct MySettings {
//!     app: String,
//!     secret: String,
//!     env_var: String,
//! }
//!
//! impl SettingsLoader for MySettings {
//!     type Options = MyOptions;
//! }
//!
//! std::fs::write("config.yaml", "app: my_app").unwrap();
//! std::fs::write("secrets.yaml", "secret: my_secret").unwrap();
//! std::env::set_var("APP__ENV_VAR", "my_env_var");
//!
//! let options = MyOptions;
//! let settings = MySettings::load(&options).expect("Failed to load settings");
//!
//! assert_eq!(settings.app, "my_app");
//! assert_eq!(settings.secret, "my_secret");
//! assert_eq!(settings.env_var, "my_env_var");
//!
//! std::fs::remove_file("config.yaml").unwrap();
//! std::fs::remove_file("secrets.yaml").unwrap();
//! std::env::remove_var("APP__ENV_VAR");
//! ```

use std::io;
use std::path::{Path, PathBuf};

use config::builder::DefaultState;
use config::{Config, ConfigBuilder};
use path_absolutize::Absolutize;

use crate::provenance::{SettingSource, SourceMap, SourceMetadata};
use crate::scope::ConfigScope;
use crate::{Environment, SettingsError};
use config::Source;

/// Result of resolving a configuration layer into its components.
///
/// Contains metadata for provenance tracking and dual source representations
/// for both configuration building and provenance tracking.
#[derive(Debug)]
struct LayerResolution {
    /// Metadata describing the source of this layer
    metadata: SourceMetadata,
    /// Primary source for building configuration
    config_source: Box<dyn Source + Send + Sync>,
    /// Secondary source for provenance tracking
    provenance_source: Box<dyn Source + Send + Sync>,
}

/// Result type for layer resolution.
type LayerResult = Result<LayerResolution, SettingsError>;

/// A builder for constructing a layered configuration.
type ConfigFile = config::File<config::FileSourceFile, config::FileFormat>;

/// Represents a single configuration layer/source.
///
/// Layers are applied in order (first to last), with later layers having
/// higher precedence and overriding earlier values.
#[derive(Debug, Clone)]
pub enum ConfigLayer {
    /// Load configuration from an explicit file path.
    /// Fails if file does not exist.
    Path(PathBuf),

    /// Load configuration from a path specified in an environment variable.
    /// If the env var is not set, this layer is skipped.
    EnvVar(String),

    /// Search for environment-specific configuration files.
    /// Searches for files named after the environment (e.g., "production.yaml")
    /// in the specified directories.
    EnvSearch { env: Environment, dirs: Vec<PathBuf> },

    /// Load secrets from an explicit file path.
    /// Fails if file does not exist.
    Secrets(PathBuf),

    /// Load configuration from system environment variables.
    /// Prefix determines the env var namespace (e.g., "APP" → APP_*)
    /// Separator determines path nesting (e.g., "__" → APP_DB__HOST)
    EnvVars { prefix: String, separator: String },

    /// Load configuration from a file with specific scope information.
    /// Identical to Path but carries source scope metadata.
    ScopedPath {
        path: PathBuf,
        scope: crate::scope::ConfigScope,
    },
}

impl ConfigLayer {
    /// Returns true if this is a Path layer.
    pub fn is_path(&self) -> bool {
        matches!(self, ConfigLayer::Path(_))
    }

    /// Returns true if this is an EnvVar layer.
    pub fn is_env_var(&self) -> bool {
        matches!(self, ConfigLayer::EnvVar(_))
    }

    /// Returns true if this is an EnvSearch layer.
    pub fn is_env_search(&self) -> bool {
        matches!(self, ConfigLayer::EnvSearch { .. })
    }

    /// Returns true if this is a Secrets layer.
    pub fn is_secrets(&self) -> bool {
        matches!(self, ConfigLayer::Secrets(_))
    }

    /// Returns true if this is an EnvVars layer.
    pub fn is_env_vars(&self) -> bool {
        matches!(self, ConfigLayer::EnvVars { .. })
    }

    /// Returns true if this is a ScopedPath layer.
    pub fn is_scoped_path(&self) -> bool {
        matches!(self, ConfigLayer::ScopedPath { .. })
    }
}

/// Builder for constructing an ordered sequence of configuration layers.
///
/// Layers are applied in order, with later layers having higher precedence
/// and overriding earlier layers.
#[derive(Debug, Clone)]
pub struct LayerBuilder {
    layers: Vec<ConfigLayer>,
}

impl LayerBuilder {
    /// Create a new empty layer builder.
    pub fn new() -> Self {
        LayerBuilder { layers: Vec::new() }
    }

    /// Add a path-based configuration layer.
    ///
    /// The file at the path must exist or building will fail.
    pub fn with_path(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.layers.push(ConfigLayer::Path(path.as_ref().to_path_buf()));
        self
    }

    /// Add a path-based configuration layer by discovering the file format.
    ///
    /// Searches the specified directory for a file with the given basename and any
    /// supported extension (yaml, yml, toml, json, ron, hjson, json5). The first
    /// matching file found (in that order) will be used.
    ///
    /// This method performs explicit file discovery rather than delegating to `config-rs`
    /// because it provides clearer error messages and explicit file selection.
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory to search in
    /// * `basename` - Base name of the config file (without extension)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Searches for ./config/application.{yaml,yml,toml,json,ron,hjson,json5}
    /// let builder = LayerBuilder::new()
    ///     .with_path_in_dir("config", "application");
    /// ```
    ///
    /// The file must exist or building will fail.
    pub fn with_path_in_dir(mut self, dir: impl AsRef<std::path::Path>, basename: impl AsRef<str>) -> Self {
        let dir = dir.as_ref();
        let basename = basename.as_ref();

        // Search for file with supported extensions in order of preference
        let extensions = ["yaml", "yml", "toml", "json", "ron", "hjson", "json5"];

        for ext in &extensions {
            let path = dir.join(format!("{}.{}", basename, ext));
            if path.exists() {
                self.layers.push(ConfigLayer::Path(path));
                return self;
            }
        }

        // If no file found, still add a Path layer - it will fail during build()
        // This provides better error messages
        let attempted_path = dir.join(format!("{}.yaml", basename));
        self.layers.push(ConfigLayer::Path(attempted_path));
        self
    }

    /// Add an environment variable configuration layer.
    ///
    /// If the env var is not set, this layer is skipped gracefully.
    pub fn with_env_var(mut self, var_name: &str) -> Self {
        self.layers.push(ConfigLayer::EnvVar(var_name.to_string()));
        self
    }

    /// Add an environment-directed search layer.
    ///
    /// Searches for files named after the environment (e.g., "production.yaml")
    /// in the specified directories.
    pub fn with_env_search(mut self, env: Environment, dirs: impl IntoIterator<Item = PathBuf>) -> Self {
        self.layers
            .push(ConfigLayer::EnvSearch { env, dirs: dirs.into_iter().collect() });
        self
    }

    /// Add a secrets file layer.
    ///
    /// The file must exist or building will fail.
    pub fn with_secrets(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.layers.push(ConfigLayer::Secrets(path.as_ref().to_path_buf()));
        self
    }

    /// Add environment variable configuration layer.
    ///
    /// Prefix determines the env var namespace (e.g., "APP" → APP_*)
    /// Separator determines path nesting (e.g., "__" → APP_DB__HOST)
    pub fn with_env_vars(mut self, prefix: &str, separator: &str) -> Self {
        self.layers.push(ConfigLayer::EnvVars {
            prefix: prefix.to_string(),
            separator: separator.to_string(),
        });
        self
    }

    /// Add multiple scopes to the layer builder using MultiScopeConfig.
    ///
    /// Automatically resolves paths for each scope based on platform conventions
    /// and adds them as Path layers in the specified order. Missing or non-existent
    /// files are skipped gracefully.
    ///
    /// # Arguments
    ///
    /// * `scopes` - An iterable of ConfigScope values to load
    ///
    /// # Example
    ///
    /// ```ignore
    /// use settings_loader::{LayerBuilder, ConfigScope, MultiScopeConfig};
    ///
    /// struct AppConfig;
    ///
    /// impl LoadingOptions for AppConfig { ... }
    ///
    /// impl MultiScopeConfig for AppConfig {
    ///     const APP_NAME: &'static str = "my-app";
    ///     // ... other required methods ...
    /// }
    ///
    /// let builder = LayerBuilder::new()
    ///     .with_scopes::<AppConfig>(AppConfig::default_scopes());
    /// ```
    pub fn with_scopes<T: crate::MultiScopeConfig>(
        mut self, scopes: impl IntoIterator<Item = crate::ConfigScope>,
    ) -> Self {
        for scope in scopes {
            if let Some(path) = T::resolve_path(scope) {
                // Use ScopedPath to preserve scope info for provenance
                self.layers.push(ConfigLayer::ScopedPath { path, scope });
            }
        }
        self
    }

    /// Returns a slice of all layers added to this builder.
    pub fn layers(&self) -> &[ConfigLayer] {
        &self.layers
    }

    /// Returns the number of layers in this builder.
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Returns true if no layers have been added.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Returns true if any layers have been added (opposite of is_empty).
    pub fn has_layers(&self) -> bool {
        !self.is_empty()
    }

    /// Returns true if builder contains at least one Path layer.
    pub fn has_path_layer(&self) -> bool {
        self.layers.iter().any(|l| matches!(l, ConfigLayer::Path(_)))
    }

    /// Returns true if builder contains at least one EnvVar layer with the given name.
    pub fn has_env_var_layer(&self, name: &str) -> bool {
        self.layers
            .iter()
            .any(|l| matches!(l, ConfigLayer::EnvVar(ref var_name) if var_name == name))
    }

    /// Returns true if builder contains at least one Secrets layer.
    pub fn has_secrets_layer(&self) -> bool {
        self.layers.iter().any(|l| matches!(l, ConfigLayer::Secrets(_)))
    }

    /// Returns true if builder contains an EnvVars layer with the given prefix and separator.
    pub fn has_env_vars_layer(&self, prefix: &str, sep: &str) -> bool {
        self.layers.iter().any(|l| {
            matches!(l, ConfigLayer::EnvVars { prefix: ref p, separator: ref s }
                if p == prefix && s == sep)
        })
    }

    /// Build the configuration from accumulated layers.
    ///
    /// # Returns
    /// A ConfigBuilder with all layers added in order.
    ///
    /// # Errors
    /// Returns SettingsError if layer validation fails (missing files, path absolutization, etc).
    pub fn build(mut self) -> Result<ConfigBuilder<DefaultState>, SettingsError> {
        let mut config = Config::builder();

        let layers = std::mem::take(&mut self.layers);
        for (idx, layer) in layers.into_iter().enumerate() {
            let resolution = self.resolve_layer_sources(&layer, idx)?;
            config = config.add_source(vec![resolution.config_source]);
        }

        Ok(config)
    }

    /// Build the configuration and return the source map (provenance).
    ///
    /// Identical to `build()`, but also tracks which source provided which value.
    pub fn build_with_provenance(mut self) -> Result<(ConfigBuilder<DefaultState>, SourceMap), SettingsError> {
        let mut config = Config::builder();
        let mut source_map = SourceMap::new();

        let layers = std::mem::take(&mut self.layers);
        for (idx, layer) in layers.into_iter().enumerate() {
            let resolution = self.resolve_layer_sources(&layer, idx)?;
            config = config.add_source(vec![resolution.config_source]);
            source_map.insert_layer(resolution.metadata, resolution.provenance_source.collect()?);
        }

        Ok((config, source_map))
    }

    /// Helper to resolve a single layer into metadata and dual sources.
    fn resolve_layer_sources(&self, layer: &ConfigLayer, idx: usize) -> LayerResult {
        match layer {
            ConfigLayer::Path(path) | ConfigLayer::ScopedPath { path, .. } => {
                let scope = if let ConfigLayer::ScopedPath { scope, .. } = layer {
                    Some(*scope)
                } else {
                    None
                };
                self.resolve_file_layer(path, scope, idx)
            },
            ConfigLayer::EnvVar(var_name) => self.resolve_env_var_layer(var_name, idx),
            ConfigLayer::EnvSearch { env, dirs } => self.resolve_env_search_layer(env, dirs, idx),
            ConfigLayer::Secrets(path) => self.resolve_secrets_layer(path, idx),
            ConfigLayer::EnvVars { prefix, separator } => self.resolve_env_vars_layer(prefix, separator, idx),
        }
    }

    fn resolve_file_layer(&self, path: &Path, scope: Option<ConfigScope>, idx: usize) -> LayerResult {
        let abs_path = path.absolutize()?;
        if !abs_path.exists() {
            return Err(SettingsError::from(io::Error::new(
                io::ErrorKind::NotFound,
                format!("config file not found: {}", abs_path.display()),
            )));
        }

        let meta = SourceMetadata::file(abs_path.to_path_buf(), scope, idx);
        let s1 = ConfigFile::from(abs_path.as_ref()).required(true);
        let s2 = ConfigFile::from(abs_path.as_ref()).required(true);
        Ok(LayerResolution {
            metadata: meta,
            config_source: Box::new(s1),
            provenance_source: Box::new(s2),
        })
    }

    fn resolve_env_var_layer(&self, var_name: &str, idx: usize) -> LayerResult {
        let meta = SourceMetadata::env(var_name.to_string(), idx);
        match std::env::var(var_name) {
            Ok(env_path) => {
                let abs_path = Path::new(&env_path).absolutize()?;
                if !abs_path.exists() {
                    return Err(SettingsError::from(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!(
                            "config file from env var {} not found: {}",
                            var_name,
                            abs_path.display()
                        ),
                    )));
                }
                let s1 = ConfigFile::from(abs_path.as_ref()).required(true);
                let s2 = ConfigFile::from(abs_path.as_ref()).required(true);
                Ok(LayerResolution {
                    metadata: meta,
                    config_source: Box::new(s1),
                    provenance_source: Box::new(s2),
                })
            },
            Err(_) => {
                let s1 = config::File::from_str("{}", config::FileFormat::Json);
                let s2 = config::File::from_str("{}", config::FileFormat::Json);
                Ok(LayerResolution {
                    metadata: meta,
                    config_source: Box::new(s1),
                    provenance_source: Box::new(s2),
                })
            },
        }
    }

    fn resolve_env_search_layer(&self, env: &Environment, dirs: &[PathBuf], idx: usize) -> LayerResult {
        let env_name = env.as_ref();
        let extensions = ["toml", "yaml", "yml", "json", "ron", "hjson", "json5"];

        for dir in dirs {
            for ext in &extensions {
                let path = dir.join(format!("{}.{}", env_name, ext));
                if path.exists() {
                    let abs_path = path.absolutize()?;
                    let meta = SourceMetadata::file(abs_path.to_path_buf(), None, idx);
                    let s1 = ConfigFile::from(abs_path.as_ref()).required(true);
                    let s2 = ConfigFile::from(abs_path.as_ref()).required(true);
                    return Ok(LayerResolution {
                        metadata: meta,
                        config_source: Box::new(s1),
                        provenance_source: Box::new(s2),
                    });
                }
            }
        }

        // No file found, return empty
        let meta = SourceMetadata {
            source: SettingSource::Override { name: format!("env_search:{}", env) },
            layer_index: idx,
        };
        let s1 = config::File::from_str("{}", config::FileFormat::Json);
        let s2 = config::File::from_str("{}", config::FileFormat::Json);
        Ok(LayerResolution {
            metadata: meta,
            config_source: Box::new(s1),
            provenance_source: Box::new(s2),
        })
    }

    fn resolve_secrets_layer(&self, path: &Path, idx: usize) -> LayerResult {
        let abs_path = path.absolutize()?;
        if !abs_path.exists() {
            return Err(SettingsError::from(io::Error::new(
                io::ErrorKind::NotFound,
                format!("secrets file not found: {}", abs_path.display()),
            )));
        }

        let meta = SourceMetadata {
            source: SettingSource::Secrets { path: abs_path.to_path_buf() },
            layer_index: idx,
        };
        let s1 = ConfigFile::from(abs_path.as_ref()).required(true);
        let s2 = ConfigFile::from(abs_path.as_ref()).required(true);
        Ok(LayerResolution {
            metadata: meta,
            config_source: Box::new(s1),
            provenance_source: Box::new(s2),
        })
    }

    fn resolve_env_vars_layer(&self, prefix: &str, separator: &str, idx: usize) -> LayerResult {
        let meta = SourceMetadata {
            source: SettingSource::EnvVars { prefix: prefix.to_string() },
            layer_index: idx,
        };
        let s1 = config::Environment::default()
            .prefix(prefix)
            .separator(separator)
            .try_parsing(true);
        let s2 = config::Environment::default()
            .prefix(prefix)
            .separator(separator)
            .try_parsing(true);
        Ok(LayerResolution {
            metadata: meta,
            config_source: Box::new(s1),
            provenance_source: Box::new(s2),
        })
    }
}

impl Default for LayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
