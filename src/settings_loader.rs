use std::fmt::Debug;
use std::path::PathBuf;

use config::builder::DefaultState;
use config::ConfigBuilder;
use serde::de::DeserializeOwned;

use crate::{Environment, LoadingOptions, SettingsError};

pub trait SettingsLoader: Debug + Sized {
    type Options: LoadingOptions + Debug;

    fn resources() -> PathBuf {
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
    fn load(options: &Self::Options) -> Result<Self, SettingsError>
    where
        Self: DeserializeOwned,
    {
        tracing::info!(?options, "loading common based on CLI options.");
        let resources = options
            .resources_path()
            .unwrap_or(std::env::current_dir()?.join(Self::resources()));

        let mut builder = config::Config::builder();
        match options.config_path() {
            Some(ref path) => {
                builder = builder.add_source(Self::make_explicit_config_source(path));
            },
            None => {
                builder = builder.add_source(Self::make_implicit_app_config_sources(
                    Self::app_config_basename(),
                    &resources,
                ));

                if let Some(env) = options.environment() {
                    builder = builder.add_source(Self::make_app_environment_source(env, &resources));
                }
            },
        }

        if let Some(ref secrets) = options.secrets_path() {
            builder = builder.add_source(Self::make_secrets_source(secrets));
        }

        builder = builder.add_source(Self::make_environment_variables_source());

        builder = options
            .load_overrides(builder)
            .map_err(|err| SettingsError::CliOptionError(err.into()))?;

        let config = builder.build()?;
        tracing::info!(?config, "configuration loaded");
        let settings = config.try_into()?;
        tracing::info!(?settings, "common built for application.");
        Ok(settings)
    }

    fn make_explicit_config_source(path: &PathBuf) -> config::File<config::FileSourceFile> {
        config::File::from(path.as_path()).required(true)
    }

    fn make_implicit_app_config_sources(basename: &str, resources: &PathBuf) -> config::File<config::FileSourceFile> {
        tracing::debug!("looking for {} config in: {:?}", basename, resources);
        let path = resources.join(basename);
        config::File::from(path).required(true)
    }

    fn make_app_environment_source(
        environment: Environment, resources: &PathBuf,
    ) -> config::File<config::FileSourceFile> {
        tracing::debug!("looking for {} config at {:?}", environment, resources);
        let env_path = resources.join(environment.as_ref());
        config::File::from(env_path).required(false)
    }

    fn make_secrets_source(secrets_path: &PathBuf) -> config::File<config::FileSourceFile> {
        if secrets_path.as_path().exists() {
            tracing::info!("adding secrets override configuration source from {:?}", secrets_path);
        } else {
            tracing::error!("cannot find secrets override configuration at {:?}", secrets_path);
        }
        config::File::from(secrets_path.as_path()).required(true)
    }

    fn make_environment_variables_source() -> config::Environment {
        let config_env =
            config::Environment::with_prefix(Self::environment_prefix()).separator(Self::environment_path_separator());
        tracing::info!("loading environment variables with: {:?}", config_env);
        config_env
    }

    // #[tracing::instrument(level = "info", skip(config,))]
    // fn add_configuration_source(
    //     config: ConfigBuilder<DefaultState>, specific_config_path: Option<PathBuf>,
    // ) -> Result<ConfigBuilder<DefaultState>, SettingsError> {
    //     match specific_config_path {
    //         Some(explicit_path) => {
    //             let config = config.add_source(config::File::from(explicit_path).required(true));
    //             Ok(config)
    //         }
    //
    //         None => {
    //             let resources_path = std::env::current_dir()?.join(Self::resources());
    //             let config_path = resources_path.join(Self::app_config_basename());
    //             tracing::debug!(
    //                 "looking for {} config in: {:?}",
    //                 Self::app_config_basename(),
    //                 resources_path
    //             );
    //             let config = config.add_source(config::File::from(config_path).required(true));
    //
    //             match std::env::var(CliOptions::env_app_environment()) {
    //                 Ok(rep) => {
    //                     let environment: Environment = rep.try_into()?;
    //                     Ok(Self::add_app_environment_source(config, environment, &resources_path))
    //                 }
    //
    //                 Err(std::env::VarError::NotPresent) => {
    //                     tracing::warn!(
    //                         "no environment variable override on common specified at env var, {}",
    //                         Self::env_app_environment()
    //                     );
    //                     Ok(config)
    //                 }
    //
    //                 Err(err) => Err(err.into()),
    //             }
    //         }
    //     }
    // }

    #[tracing::instrument(level = "info", skip(config))]
    fn add_app_environment_source(
        config: ConfigBuilder<DefaultState>, env: Environment, resources_path: &PathBuf,
    ) -> ConfigBuilder<DefaultState> {
        let env_config_path = resources_path.join(env.as_ref());
        tracing::debug!("looking for {} configu at {:?}", env, resources_path);
        config.add_source(config::File::from(env_config_path).required(false))
    }

    #[tracing::instrument(level = "info", skip(config))]
    fn add_secrets_source(
        config: ConfigBuilder<DefaultState>, secrets_path: Option<PathBuf>,
    ) -> ConfigBuilder<DefaultState> {
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
    fn add_environment_variables_source(config: ConfigBuilder<DefaultState>) -> ConfigBuilder<DefaultState> {
        let config_env =
            config::Environment::with_prefix(Self::environment_prefix()).separator(Self::environment_path_separator());
        tracing::info!("loading environment variables with prefix: {:?}", config_env);
        config.add_source(config_env)
    }
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use config::{Config, FileFormat};
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};
    use serde_with::{serde_as, DisplayFromStr};

    use super::*;
    use crate::{NoOptions, APP_ENVIRONMENT};

    #[derive(Debug, PartialEq, Eq)]
    struct TestOptions(String, Option<Environment>);

    impl LoadingOptions for TestOptions {
        type Error = SettingsError;

        fn config_path(&self) -> Option<PathBuf> {
            None
        }

        fn secrets_path(&self) -> Option<PathBuf> {
            Some(PathBuf::from("./resources/secrets.yaml"))
        }

        fn environment_override(&self) -> Option<Environment> {
            self.1
        }

        #[tracing::instrument(level = "info", skip(config))]
        fn load_overrides(
            &self, config: ConfigBuilder<DefaultState>,
        ) -> Result<ConfigBuilder<DefaultState>, Self::Error> {
            Ok(config.set_override("foo", self.0.as_str())?)
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
            vec![(APP_ENVIRONMENT, None)], // Some("local"))],
            || {
                eprintln!("+ test_load_string_settings");
                Lazy::force(&TEST_TRACING);
                let main_span = tracing::info_span!("test_load_string_settings");
                let _ = main_span.enter();

                assert_err!(std::env::var(APP_ENVIRONMENT));
                tracing::info!("envar: {} = {:?}", APP_ENVIRONMENT, std::env::var(APP_ENVIRONMENT));

                let config = Config::builder().add_source(config::File::from_str(
                    r###"
                    | application:
                    |   port: 8000
                    |   host: 10.1.2.57
                    |   base_url: "http://10.1.2.57"
                    | database:
                    |   username: postgres
                    |   password: password
                    |   port: 5432
                    |   host: "localhost"
                    |   database_name: "propensity"
                    |   require_ssl: true
                "###
                    .trim_margin_with("| ")
                    .unwrap()
                    .as_str(),
                    FileFormat::Yaml,
                ));

                let options = TestOptions("bar".to_string(), Some(Environment::Local));
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
                Lazy::force(&TEST_TRACING);
                let main_span = tracing::info_span!("test_settings_load_w_options");
                let _ = main_span.enter();

                assert_eq!(assert_ok!(std::env::var(APP_ENVIRONMENT)), "local");
                tracing::info!("envar: {} = {:?}", APP_ENVIRONMENT, std::env::var(APP_ENVIRONMENT));

                let actual = assert_ok!(TestSettings::load(&TestOptions("zed".to_string(), None)));

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
        Lazy::force(&TEST_TRACING);
        let main_span = tracing::info_span!("test_settings_load_w_no_options");
        let _ = main_span.enter();

        with_env_vars(
            "test_settings_load_w_no_options",
            vec![(APP_ENVIRONMENT, Some("production"))],
            || {
                assert_eq!(assert_ok!(std::env::var(APP_ENVIRONMENT)), "production");
                tracing::info!("envar: {} = {:?}", APP_ENVIRONMENT, std::env::var(APP_ENVIRONMENT));

                let actual = assert_ok!(TestSettingsNoOpts::load(&()));

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

    use std::env::VarError;
    use std::panic::{RefUnwindSafe, UnwindSafe};
    use std::sync::Mutex;
    use std::{env, panic};

    use once_cell::sync::Lazy;
    use trim_margin::MarginTrimmable;

    use crate::tracing::TEST_TRACING;

    static SERIAL_TEST: Lazy<Mutex<()>> = Lazy::new(|| Default::default());

    /// Sets environment variables to the given value for the duration of the closure.
    /// Restores the previous values when the closure completes or panics, before unwinding the
    /// panic.
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
            },
            Err(err) => {
                eprintln!("W_END[{}]: Err - resetting env to: {:?}", label, old_kvs);
                for (k, v) in old_kvs {
                    reset_env(k, v);
                }
                drop(guard);
                panic::resume_unwind(err);
            },
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
