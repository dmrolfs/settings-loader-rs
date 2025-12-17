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

use crate::{Environment, SettingsError};

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
                self = self.with_path(path);
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
    pub fn build(self) -> Result<ConfigBuilder<DefaultState>, SettingsError> {
        let mut config = Config::builder();

        for layer in self.layers {
            config = match layer {
                ConfigLayer::Path(path) => {
                    let abs_path = path.absolutize()?;
                    // Validate file exists before adding to config
                    std::fs::metadata(&abs_path).map_err(|_e| {
                        SettingsError::from(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("config file not found: {}", abs_path.display()),
                        ))
                    })?;
                    config.add_source(ConfigFile::from(abs_path.as_ref()).required(true))
                },
                ConfigLayer::EnvVar(var_name) => {
                    match std::env::var(&var_name) {
                        Ok(env_path) => {
                            let abs_path = Path::new(&env_path).absolutize()?;
                            // Validate file exists before adding to config
                            std::fs::metadata(&abs_path).map_err(|_e| {
                                SettingsError::from(io::Error::new(
                                    io::ErrorKind::NotFound,
                                    format!("config file not found: {}", abs_path.display()),
                                ))
                            })?;
                            config.add_source(ConfigFile::from(abs_path.as_ref()).required(true))
                        },
                        Err(std::env::VarError::NotPresent) => {
                            // Skip gracefully - no error
                            config
                        },
                        Err(e) => return Err(e.into()),
                    }
                },
                ConfigLayer::EnvSearch { env: _env, dirs: _dirs } => {
                    // TODO: Implement EnvSearch in Phase 3
                    // For now, skip this layer
                    config
                },
                ConfigLayer::Secrets(path) => {
                    let abs_path = path.absolutize()?;
                    // Validate file exists before adding to config
                    std::fs::metadata(&abs_path).map_err(|_e| {
                        SettingsError::from(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("secrets file not found: {}", abs_path.display()),
                        ))
                    })?;
                    config.add_source(ConfigFile::from(abs_path.as_ref()).required(true))
                },
                ConfigLayer::EnvVars { prefix, separator } => config.add_source(
                    config::Environment::default()
                        .prefix(&prefix)
                        .separator(&separator)
                        .try_parsing(true),
                ),
            };
        }

        Ok(config)
    }
}

impl Default for LayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}
