#![allow(clippy::multiple_crate_versions)]

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

pub use environment::Environment;
pub use error::SettingsError;
pub use layer::{ConfigLayer, LayerBuilder};
pub use loading_options::{LoadingOptions, MultiScopeConfig, NoOptions};
pub use scope::ConfigScope;
pub use settings_loader::SettingsLoader;

pub mod common;
pub mod environment;
pub mod error;
mod internals;
pub mod layer;
pub mod loading_options;
pub mod provenance;
pub mod scope;
pub mod settings_loader;

pub use provenance::{SourceMap, SourceMetadata, SourceType};

#[cfg(feature = "editor")]
pub mod editor;
#[cfg(feature = "editor")]
pub use crate::editor::{
    ConfigEditor, ConfigFormat, Editor, EditorError, LayerEditor, SettingsEditor, SettingsLoaderEditor,
};

#[cfg(feature = "metadata")]
pub mod metadata;

#[cfg(feature = "metadata")]
pub mod introspection;

#[cfg(feature = "metadata")]
pub mod validation;

#[cfg(feature = "metadata")]
pub mod registry;

#[cfg(feature = "metadata")]
pub use registry::{global_schema, init_global_registry, register_setting, SettingsRegistry};

#[cfg(test)]
mod tracing;
