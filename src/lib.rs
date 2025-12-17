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
pub use loading_options::{LoadingOptions, NoOptions};
pub use scope::ConfigScope;
pub use settings_loader::SettingsLoader;

pub mod common;
pub mod environment;
pub mod error;
mod internals;
pub mod layer;
pub mod loading_options;
pub mod scope;
pub mod settings_loader;

#[cfg(test)]
mod tracing;
