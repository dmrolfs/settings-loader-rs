#[macro_use]
extern crate enum_display_derive;

pub use crate::settings_loader::SettingsLoader;
use config::builder::DefaultState;
use config::ConfigBuilder;
pub use environment::Environment;
pub use error::SettingsError;
use std::fmt::Debug;
use std::path::PathBuf;

pub mod common;
pub mod environment;
pub mod error;
pub mod settings_loader;

pub trait LoadingOptions: Debug {
    fn config_path(&self) -> Option<PathBuf>;
    fn secrets_path(&self) -> Option<PathBuf>;
}

pub trait OptionOverrides {
    type Error: std::error::Error + Sync + Send + 'static;
    fn load_overrides(self, config: ConfigBuilder<DefaultState>) -> Result<ConfigBuilder<DefaultState>, Self::Error>;
}
