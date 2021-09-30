use crate::{Environment, LoadingOptions, SettingsError};
use config::builder::DefaultState;
use config::ConfigBuilder;
use serde::de::DeserializeOwned;
use std::convert::TryInto;
use std::fmt::Debug;
use std::path::PathBuf;

pub trait SettingsLoader: Debug + Sized {
    type Options: LoadingOptions;

    fn env_app_environment() -> &'static str {
        "APP_ENVIRONMENT"
    }
    fn resources_dir() -> PathBuf {
        PathBuf::from("resources")
    }
    fn app_config_basename() -> &'static str {
        "application"
    }

    fn environment_prefix() -> &'static str {
        "app"
    }
    fn environment_path_separator() -> &'static str {
        "__"
    }

    #[tracing::instrument(level = "info")]
    fn load(options: Self::Options) -> Result<Self, SettingsError>
    where
        Self: DeserializeOwned,
    {
        tracing::info!(?options, "loading common based on CLI options.");
        let mut config_builder = config::Config::builder();
        config_builder = Self::load_configuration(config_builder, options.config_path())?;
        config_builder = Self::load_secrets(config_builder, options.secrets_path());
        config_builder = Self::load_environment(config_builder);
        let config = config_builder.build()?;
        tracing::info!(?config, "configuration loaded");
        let settings = config.try_into()?;
        tracing::info!(?settings, "common built for application.");
        Ok(settings)
    }

    #[tracing::instrument(level = "info", skip(config,))]
    fn load_configuration(
        config: ConfigBuilder<DefaultState>, specific_config_path: Option<PathBuf>,
    ) -> Result<ConfigBuilder<DefaultState>, SettingsError> {
        match specific_config_path {
            Some(explicit_path) => {
                let config = config.add_source(config::File::from(explicit_path).required(true));
                Ok(config)
            }

            None => {
                let resources_path = std::env::current_dir()?.join(Self::resources_dir());
                let config_path = resources_path.join(Self::app_config_basename());
                tracing::debug!(
                    "looking for {} config in: {:?}",
                    Self::app_config_basename(),
                    resources_path
                );
                let config =
                    config.add_source(config::File::with_name(config_path.to_string_lossy().as_ref()).required(true));

                match std::env::var(Self::env_app_environment()) {
                    Ok(rep) => {
                        let environment: Environment = rep.try_into()?;
                        let env_config_path = resources_path.join(environment.as_ref());
                        tracing::debug!("looking for {} config in: {:?}", environment, resources_path);
                        let config = config.add_source(
                            config::File::with_name(env_config_path.to_string_lossy().as_ref()).required(true),
                        );
                        Ok(config)
                    }

                    Err(std::env::VarError::NotPresent) => {
                        tracing::warn!(
                            "no environment variable override on common specified at env var, {}",
                            Self::env_app_environment()
                        );
                        Ok(config)
                    }

                    Err(err) => Err(err.into()),
                }
            }
        }
    }

    #[tracing::instrument(level = "info", skip(config))]
    fn load_secrets(config: ConfigBuilder<DefaultState>, secrets_path: Option<PathBuf>) -> ConfigBuilder<DefaultState> {
        if let Some(path) = secrets_path {
            tracing::debug!(
                "looking for secrets configuration at: {:?} -- exists:{}",
                path,
                path.as_path().exists()
            );
            if path.as_path().exists() {
                tracing::info!("adding secrets override configuration source from {:?}", path);
            } else {
                tracing::error!("cannot find secrets override configuration at {:?}", path);
            }
            config.add_source(config::File::from(path).required(true))
        } else {
            config
        }
    }

    #[tracing::instrument(level = "info", skip(config))]
    fn load_environment(config: ConfigBuilder<DefaultState>) -> ConfigBuilder<DefaultState> {
        let config_env =
            config::Environment::with_prefix(Self::environment_prefix()).separator(Self::environment_path_separator());
        tracing::info!("loading environment properties with prefix: {:?}", config_env);
        config.add_source(config_env)
    }
}
