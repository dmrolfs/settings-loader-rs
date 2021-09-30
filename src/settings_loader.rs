use crate::{Environment, LoadingOptions, SettingsError};
use config::builder::DefaultState;
use config::ConfigBuilder;
use serde::de::DeserializeOwned;
use std::convert::TryInto;
use std::fmt::Debug;
use std::path::PathBuf;

const APP_ENVIRONMENT: &'static str = "APP_ENVIRONMENT";

pub trait SettingsLoader: Debug + Sized {
    type Options: LoadingOptions + Debug;

    fn env_app_environment() -> &'static str {
        APP_ENVIRONMENT
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
        config_builder = options
            .load_overrides(config_builder)
            .map_err(|err| SettingsError::CliOptionError(err.into()))?;
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

#[cfg(test)]
mod tests {
    use claim::assert_ok;
    use config::{Config, FileFormat};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::NoOptions;
    use serde::{Deserialize, Serialize};
    use serde_with::{serde_as, DisplayFromStr};

    #[derive(Debug, PartialEq, Eq)]
    struct TestOptions(String);

    impl LoadingOptions for TestOptions {
        type Error = SettingsError;

        fn config_path(&self) -> Option<PathBuf> {
            None
        }

        fn secrets_path(&self) -> Option<PathBuf> {
            Some(PathBuf::from("./resources/secrets.yaml"))
        }

        #[tracing::instrument(level = "info", skip(config))]
        fn load_overrides(
            self, config: ConfigBuilder<DefaultState>,
        ) -> Result<ConfigBuilder<DefaultState>, Self::Error> {
            Ok(config.set_override("foo", self.0)?)
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestSettings {
        pub application: TestHttpSettings,
        pub database: TestDbSettings,
        pub foo: String,
    }

    impl SettingsLoader for TestSettings {
        type Options = TestOptions;
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestSettingsNoOpts {
        pub application: TestHttpSettings,
        pub database: TestDbSettings,
        pub foo: String,
    }

    impl SettingsLoader for TestSettingsNoOpts {
        type Options = NoOptions;
    }

    #[serde_as]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestDbSettings {
        pub username: String,
        pub password: String,
        #[serde_as(as = "DisplayFromStr")]
        pub port: u16,
        pub host: String,
        pub database_name: String,
        pub require_ssl: bool,
    }

    #[serde_as]
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestHttpSettings {
        pub host: String,
        #[serde_as(as = "DisplayFromStr")]
        pub port: u16,
    }

    #[test]
    fn test_load_string_settings() -> anyhow::Result<()> {
        with_env_vars(
            "test_load_string_settings",
            vec![(APP_ENVIRONMENT, Some("local"))],
            || {
                eprintln!("+ test_load_string_settings");
                lazy_static::initialize(&crate::tracing::TEST_TRACING);
                let main_span = tracing::info_span!("test_load_string_settings");
                let _ = main_span.enter();

                assert_eq!(assert_ok!(std::env::var(APP_ENVIRONMENT)), "local");
                tracing::info!("envar: {} = {:?}", APP_ENVIRONMENT, std::env::var(APP_ENVIRONMENT));

                let config = Config::builder().add_source(config::File::from_str(
                    r###"
application:
  port: 8000
  host: 10.1.2.57
  base_url: "http://10.1.2.57"
database:
  username: postgres
  password: password
  port: 5432
  host: "localhost"
  database_name: "propensity"
  require_ssl: true
                "###,
                    FileFormat::Yaml,
                ));

                let options = TestOptions("bar".to_string());
                let config = assert_ok!(options.load_overrides(config));

                let config = assert_ok!(config.build());
                tracing::info!(?config, "eligibility config loaded.");

                let actual: TestSettings = assert_ok!(config.try_into());
                assert_eq!(
                    actual,
                    TestSettings {
                        application: TestHttpSettings {
                            port: 8000,
                            host: "10.1.2.57".to_string(),
                            // base_url: "http://10.1.2.57".to_string(),
                        },
                        database: TestDbSettings {
                            username: "postgres".to_string(),
                            password: "password".to_string(),
                            port: 5432,
                            host: "localhost".to_string(),
                            database_name: "propensity".to_string(),
                            require_ssl: true,
                        },
                        foo: "bar".to_string(),
                    }
                );
            },
        );
        eprintln!("- test_load_string_settings");
        Ok(())
    }

    #[test]
    fn test_settings_load_w_options() -> anyhow::Result<()> {
        with_env_vars(
            "test_settings_load_w_options",
            vec![(APP_ENVIRONMENT, Some("local"))],
            || {
                eprintln!("+ test_settings_load_w_options");
                lazy_static::initialize(&crate::tracing::TEST_TRACING);
                let main_span = tracing::info_span!("test_settings_load_w_options");
                let _ = main_span.enter();

                assert_eq!(assert_ok!(std::env::var(APP_ENVIRONMENT)), "local");
                tracing::info!("envar: {} = {:?}", APP_ENVIRONMENT, std::env::var(APP_ENVIRONMENT));

                let actual = assert_ok!(TestSettings::load(TestOptions("zed".to_string())));

                let expected: TestSettings = TestSettings {
                    application: TestHttpSettings {
                        port: 8000,
                        host: "127.0.0.1".to_string(),
                        // base_url: "http://127.0.0.1".to_string(),
                    },
                    database: TestDbSettings {
                        username: "postgres".to_string(),
                        password: "password".to_string(),
                        port: 5432,
                        host: "localhost".to_string(),
                        database_name: "propensity".to_string(),
                        require_ssl: false,
                    },
                    foo: "zed".to_string(),
                };

                assert_eq!(actual, expected);
            },
        );
        eprintln!("- test_settings_load_w_options");
        Ok(())
    }

    #[test]
    fn test_settings_load_w_no_options() -> anyhow::Result<()> {
        eprintln!("+ test_settings_load_w_no_options");
        lazy_static::initialize(&crate::tracing::TEST_TRACING);
        let main_span = tracing::info_span!("test_settings_load_w_no_options");
        let _ = main_span.enter();

        with_env_vars(
            "test_settings_load_w_no_options",
            vec![(APP_ENVIRONMENT, Some("production"))],
            || {
                assert_eq!(assert_ok!(std::env::var(APP_ENVIRONMENT)), "production");
                tracing::info!("envar: {} = {:?}", APP_ENVIRONMENT, std::env::var(APP_ENVIRONMENT));

                let actual = assert_ok!(TestSettingsNoOpts::load(()));

                let expected = TestSettingsNoOpts {
                    application: TestHttpSettings {
                        port: 8000,
                        host: "0.0.0.0".to_string(),
                        // base_url: "http://127.0.0.1".to_string(),
                    },
                    database: TestDbSettings {
                        username: "postgres".to_string(),
                        password: "password".to_string(),
                        port: 5432,
                        host: "localhost".to_string(),
                        database_name: "propensity".to_string(),
                        require_ssl: false,
                    },
                    foo: "without_options".to_string(),
                };

                assert_eq!(actual, expected);
            },
        );
        eprintln!("- test_settings_load_w_no_options");
        Ok(())
    }

    use lazy_static::lazy_static;
    use std::env::VarError;
    use std::panic::{RefUnwindSafe, UnwindSafe};
    use std::sync::Mutex;
    use std::{env, panic};

    lazy_static! {
        static ref SERIAL_TEST: Mutex<()> = Default::default();
    }

    /// Sets environment variables to the given value for the duration of the closure.
    /// Restores the previous values when the closure completes or panics, before unwinding the panic.
    pub fn with_env_vars<F>(label: &str, kvs: Vec<(&str, Option<&str>)>, closure: F)
    where
        F: Fn() + UnwindSafe + RefUnwindSafe,
    {
        let guard = SERIAL_TEST.lock().unwrap();
        let mut old_kvs: Vec<(&str, Result<String, VarError>)> = Vec::new();
        for (k, v) in kvs {
            let old_v = env::var(k);
            old_kvs.push((k, old_v));
            match v {
                None => env::remove_var(k),
                Some(v) => env::set_var(k, v),
            }
        }
        eprintln!("W_ENV[{}]: OLD_KVS: {:?}", label, old_kvs);
        let old_kvs_2 = old_kvs.clone();

        match panic::catch_unwind(|| {
            closure();
        }) {
            Ok(_) => {
                eprintln!("W_END[{}]: OK - resetting env to: {:?}", label, old_kvs);
                for (k, v) in old_kvs {
                    reset_env(k, v);
                }
            }
            Err(err) => {
                eprintln!("W_END[{}]: Err - resetting env to: {:?}", label, old_kvs);
                for (k, v) in old_kvs {
                    reset_env(k, v);
                }
                drop(guard);
                panic::resume_unwind(err);
            }
        };
        for (k, v) in old_kvs_2 {
            eprintln!(
                "W_END[{}] RESET ACTUAL: {:?}:{:?} expected:{:?}",
                label,
                k,
                env::var(k),
                v
            );
        }
    }

    fn reset_env(k: &str, old: Result<String, VarError>) {
        if let Ok(v) = old {
            env::set_var(k, v);
        } else {
            env::remove_var(k);
        }
    }
}
