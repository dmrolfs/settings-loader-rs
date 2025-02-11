//! A library for managing and loading application settings from multiple sources.
//!
//! This library provides functionality to load and merge configuration values from
//! various sources such as configuration files (JSON, TOML, YAML, HJSON, RON),
//! environment variables, command-line arguments, and secret management systems.
//!
//! # Features
//! - Supports multiple configuration file formats (JSON, TOML, YAML, HJSON, RON).
//! - Merges configuration from multiple sources, with precedence rules (CLI > Env Vars > File).
//! - Strongly typed access to configuration values.
//! - Easily extendable to add more configuration sources.
//!
//! # Usage
//! ```rust, ignore
//! use std::path::PathBuf;
//! use serde::{Serialize, Deserialize};
//! use clap::Parser;
//! use settings_loader::{Environment, SettingsLoader, LoadingOptions, SettingsError};
//! use settings_loader::common::database::DatabaseSettings;
//!
//! pub struct ApplicationSettings { ... }
//!
//! #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
//! struct MySettings {
//!     pub application: ApplicationSettings,
//!     pub database: DatabaseSettings,
//! }
//!
//! impl SettingsLoader for MySettings {
//!     type Options = CliOptions;
//!  }
//!
//! struct CliOptions {
//!     config: Option<PathBuf>,
//!     secrets: Option<PathBuf>,
//!     environment: Option<Environment>,
//! }
//!
//! impl LoadingOptions for CliOptions {
//!     type Error = SettingsError;
//!
//!     fn config_path(&self) -> Option<PathBuf> {
//!         self.config.clone()
//!     }
//!
//!     fn secrets_path(&self) -> Option<PathBuf> {
//!         self.secrets.clone()
//!     }
//!
//!     fn implicit_search_paths(&self) -> Vec<PathBuf> {
//!         vec![PathBuf::from("./config")]
//!     }
//! }
//!
//! fn main() -> anyhow::Result<()> {
//!     ...
//!     let options = CliOptions::parse()?;
//!     let settings = MySettings::load(&options)?;
//!
//!     // Use settings...
//! }
//! ```

use std::path::PathBuf;

use config::builder::DefaultState;
use config::ConfigBuilder;

pub use environment::Environment;
pub use error::SettingsError;
pub use settings_loader::SettingsLoader;

pub mod common;
pub mod environment;
pub mod error;
mod internals;
pub mod settings_loader;
mod tracing;

const APP_ENVIRONMENT: &str = "APP_ENVIRONMENT";

/// Defines the contract for specifying how configuration settings are loaded.
///
/// The `LoadingOptions` trait provides a flexible way to define where and how configuration
/// settings should be retrieved. It enables applications to specify explicit configuration files,
/// secrets files, search paths, and environment overrides, ensuring that configurations can be
/// dynamically loaded from various sources.
///
/// The `SettingsLoader::load()` function serves as the primary driver of the settings loading
/// process, but it relies on `LoadingOptions` to determine the details of where and how to fetch
/// configuration values.
///
/// Implementing this trait allows users to:
/// - Specify an explicit configuration file.
/// - Define the location of a secrets file.
/// - Configure search paths for implicit configuration loading.
/// - Apply additional override mechanisms.
/// - Resolve the active environment.
///
/// ## Usage Example
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use settings_loader::{Environment, LoadingOptions, SettingsError};
///
/// struct CliOptions {
///     config: Option<PathBuf>,
///     secrets: Option<PathBuf>,
///     environment: Option<Environment>,
/// }
///
/// impl LoadingOptions for CliOptions {
///     type Error = SettingsError;
///
///     fn config_path(&self) -> Option<PathBuf> {
///         self.config.clone()
///     }
///
///     fn secrets_path(&self) -> Option<PathBuf> {
///         self.secrets.clone()
///     }
///
///     fn implicit_search_paths(&self) -> Vec<PathBuf> {
///         vec![PathBuf::from("./config")]
///     }
/// }
/// ```
pub trait LoadingOptions: Sized {
    /// The error type that can be returned from configuration operations.
    type Error: std::error::Error + From<error::SettingsError> + Sync + Send + 'static;

    /// Returns the path to an explicit configuration file, if provided.
    ///
    /// If a configuration file is specified via the command-line or other means,
    /// this function should return its path. If `None` is returned, the system will
    /// attempt to infer configuration from default locations.
    fn config_path(&self) -> Option<PathBuf>;

    /// Returns the path to a secrets file, if specified.
    ///
    /// This is useful for separating sensitive credentials (e.g., database passwords)
    /// from the main configuration files.
    fn secrets_path(&self) -> Option<PathBuf>;

    /// Returns a list of directories to search for configuration files.
    ///
    /// If an explicit configuration file is not provided, this function determines
    /// which directories will be scanned for inferred configuration files.
    fn implicit_search_paths(&self) -> Vec<PathBuf>;

    /// Allows customization of the configuration before finalization.
    ///
    /// This method provides an opportunity to apply additional runtime overrides,
    /// such as modifying individual settings dynamically.
    ///
    /// The default implementation returns the configuration unchanged.
    fn load_overrides(&self, config: ConfigBuilder<DefaultState>) -> Result<ConfigBuilder<DefaultState>, Self::Error> {
        Ok(config)
    }

    /// Determines the active environment for configuration resolution.
    ///
    /// This function checks for an explicit environment override, falling back to
    /// an environment variable lookup (`APP_ENVIRONMENT`). If no value is found,
    /// it logs a warning and defaults to `None`.
    fn environment(&self) -> Option<Environment> {
        let env: Option<Environment> = self
            .environment_override()
            .map(Result::<_, std::env::VarError>::Ok)
            .or_else(|| match std::env::var(Self::env_app_environment()) {
                Ok(env_rep) => Some(Ok(env_rep.into())),
                Err(std::env::VarError::NotPresent) => {
                    ::tracing::warn!(
                        "no environment variable override set at env var, {}",
                        Self::env_app_environment()
                    );

                    None
                },
                Err(err) => Some(Err(err)),
            })
            .transpose()
            .expect("failed to pull application environment");

        ::tracing::info!("loading settings for environment: {env:?}");
        env
    }

    /// Provides an optional explicit override for the environment.
    ///
    /// This can be used to manually specify the environment without relying
    /// on environment variables.
    fn environment_override(&self) -> Option<Environment> {
        None
    }

    /// Returns the environment variable key used to determine the application environment.
    ///
    /// Defaults to `"APP_ENVIRONMENT"`.
    fn env_app_environment() -> &'static str {
        APP_ENVIRONMENT
    }
}

pub type NoOptions = ();

impl LoadingOptions for () {
    type Error = SettingsError;

    fn config_path(&self) -> Option<PathBuf> {
        None
    }

    fn secrets_path(&self) -> Option<PathBuf> {
        None
    }

    fn implicit_search_paths(&self) -> Vec<PathBuf> {
        Vec::default()
    }
}
