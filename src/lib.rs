#[macro_use]
extern crate enum_display_derive;

pub use crate::settings_loader::SettingsLoader;
use config::builder::DefaultState;
use config::ConfigBuilder;
pub use environment::Environment;
pub use error::SettingsError;
use std::path::PathBuf;

pub mod common;
pub mod environment;
pub mod error;
pub mod settings_loader;
mod tracing;

pub trait LoadingOptions: Sized {
    type Error: std::error::Error + Sync + Send + 'static;
    fn config_path(&self) -> Option<PathBuf>;
    fn secrets_path(&self) -> Option<PathBuf>;
    fn load_overrides(self, config: ConfigBuilder<DefaultState>) -> Result<ConfigBuilder<DefaultState>, Self::Error> {
        Ok(config)
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
}
