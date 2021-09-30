#[macro_use]
extern crate enum_display_derive;

pub use environment::Environment;
pub use error::SettingsError;
pub use settings_loader::SettingsLoader;
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
