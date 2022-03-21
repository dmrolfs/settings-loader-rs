#![forbid(unsafe_code)]
#![warn(
clippy::cargo,
// missing_docs,
clippy::nursery,
// clippy::pedantic,
future_incompatible,
rust_2018_idioms
)]

use std::path::PathBuf;

use config::builder::DefaultState;
use config::ConfigBuilder;
pub use environment::Environment;
pub use error::SettingsError;

pub use crate::settings_loader::SettingsLoader;

pub mod common;
pub mod environment;
pub mod error;
mod internals;
pub mod settings_loader;
mod tracing;

const APP_ENVIRONMENT: &str = "APP_ENVIRONMENT";

pub trait LoadingOptions: Sized {
    type Error: std::error::Error + Sync + Send + 'static;

    fn config_path(&self) -> Option<PathBuf>;

    fn secrets_path(&self) -> Option<PathBuf>;

    fn resources_path(&self) -> Option<PathBuf> {
        None
    }

    fn load_overrides(&self, config: ConfigBuilder<DefaultState>) -> Result<ConfigBuilder<DefaultState>, Self::Error> {
        Ok(config)
    }

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

    fn environment_override(&self) -> Option<Environment> {
        None
    }

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
}
