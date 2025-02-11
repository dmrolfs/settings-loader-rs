use std::fmt::Debug;
use std::path::{Path, PathBuf};

use config::builder::DefaultState;
use config::ConfigBuilder;
use path_absolutize::*;
use serde::de::DeserializeOwned;

use crate::{Environment, LoadingOptions, SettingsError};

type ConfigFile = config::File<config::FileSourceFile, config::FileFormat>;

/// The main driver for loading application settings from multiple sources.
///
/// The `SettingsLoader` trait defines the core logic for composing configuration
/// sources into a unified representation for an application. It integrates settings
/// from configuration files, environment variables, secrets files, and command-line
/// arguments.
///
/// This trait is typically implemented for a settings structure that represents
/// the application's full configuration. The `load()` function orchestrates the
/// loading process, using `LoadingOptions` to determine available sources and
/// their precedence.
///
/// # Configuration Sources
///
/// The `SettingsLoader` trait loads settings from multiple sources, in the following
/// order of precedence (highest to lowest):
///
/// 1. **CLI Option Overrides** – Explicit settings provided via command-line arguments.
/// 2. **Environment Variables** – Overrides defined in the system environment.
/// 3. **Secrets File** – A dedicated configuration file for storing sensitive values.
/// 4. **Explicit Configuration File** – A user-defined settings file.
/// 5. **Implicit Configuration Files** – Default configurations inferred based on environment.
///
/// # Example Usage
///
/// ```rust,ignore
/// use serde::Deserialize;
/// use settings_loader::SettingsLoader;
///
/// #[derive(Debug, Deserialize)]
/// struct Settings {
///     pub http_api: HttpApiSettings,
///     pub database: DatabaseSettings,
/// }
///
/// impl SettingsLoader for Settings {
///     type Options = CliOptions;
/// }
///
/// let options = CliOptions::parse();
/// let settings = Settings::load(&options).expect("Failed to load settings");
/// println!("{:?}", settings);
/// ```
///
pub trait SettingsLoader: Debug + Sized {
    /// The options type that specifies how settings are loaded.
    type Options: LoadingOptions + Debug;

    /// Returns the default directory where configuration resources are stored.
    ///
    /// By default, this function returns `"resources"`. Implementations may override this method
    /// to provide an application-specific configuration directory.
    fn resources_home() -> PathBuf {
        PathBuf::from("resources")
    }

    /// Returns the base name for the application's primary configuration file.
    ///
    /// Defaults to `"application"`, meaning configuration files are expected to follow a naming
    /// pattern like `application.yaml`, `application.json`, etc.
    fn app_config_basename() -> &'static str {
        "application"
    }

    /// Returns the prefix used for environment variable-based configuration.
    ///
    /// Defaults to `"app"`, meaning environment variables should be prefixed with `APP_`
    /// (e.g., `APP_DATABASE_HOST`).
    fn environment_prefix() -> &'static str {
        "app"
    }

    /// Returns the separator used in environment variable keys.
    ///
    /// Defaults to `"__"`, meaning nested settings should be represented as
    /// `APP_DATABASE__HOST` instead of `APP_DATABASE_HOST`.
    fn environment_path_separator() -> &'static str {
        "__"
    }

    /// Loads application settings by composing multiple configuration sources.
    ///
    /// This function orchestrates the loading process, combining various sources
    /// in a defined order of precedence:
    ///     1. CLI option overrides,
    ///     2. environment variables,
    ///     3. secrets file
    ///     4. explicit application configuration file or implicitly loaded application
    ///        configuration with environment file overrides.
    ///
    /// If an explicit configuration file is provided via CLI options, it is loaded directly.
    /// Otherwise, implicit search paths and environment-based configurations are used.
    ///
    /// # Errors
    /// Returns a `SettingsError` if any part of the configuration loading process fails.
    ///
    /// # Instrumentation
    /// This function includes `tracing::instrument` logging to track the settings loading process.
    #[tracing::instrument(level = "info")]
    fn load(options: &Self::Options) -> Result<Self, SettingsError>
    where
        Self: DeserializeOwned,
    {
        let mut builder = config::Config::builder();
        match options.config_path() {
            Some(ref path) => {
                builder = builder.add_source(Self::make_explicit_config_source(path));
            },
            None => {
                tracing::info!(?options, "loading settings based on CLI options and environment.");
                let mut resource_dirs = Vec::default();
                for dir in options.implicit_search_paths() {
                    resource_dirs.push(dir.absolutize()?.into_owned());
                }
                if resource_dirs.is_empty() {
                    tracing::info!("no resource directories specified, using default.");
                    resource_dirs.push(Self::default_resource_path());
                }

                builder = builder.add_source(Self::make_implicit_config_source(
                    Self::app_config_basename(),
                    &resource_dirs,
                ));

                if let Some(env) = options.environment() {
                    for source in Self::make_environment_sources(env, &resource_dirs) {
                        builder = builder.add_source(source);
                    }
                }
            },
        }

        if let Some(ref secrets) = options.secrets_path() {
            let abs_secrets = secrets.absolutize()?;
            builder = builder.add_source(Self::make_secrets_source(&abs_secrets));
        }

        builder = builder.add_source(Self::make_environment_variables_source());

        builder = options
            .load_overrides(builder)
            .map_err(|err| SettingsError::CliOption(err.into()))?;

        let config = builder.build()?;
        tracing::info!(?config, "configuration loaded");
        let settings = config.try_deserialize()?;
        tracing::info!(?settings, "settings built for application.");
        Ok(settings)
    }

    /// Returns the default path to the resources directory.
    fn default_resource_path() -> PathBuf {
        let current_dir = std::env::current_dir().expect("failed to get current directory");
        current_dir.join(Self::resources_home())
    }

    /// Constructs a glob walker for searching configuration files in a directory.
    ///
    /// If the walker fails to initialize, an error is logged, and `None` is returned.
    fn make_glob_walker(base_dir: impl AsRef<Path>, pattern: impl AsRef<str>) -> Option<globwalk::GlobWalker> {
        globwalk::GlobWalkerBuilder::new(base_dir.as_ref(), pattern.as_ref())
            .build()
            .map_err(|err| {
                tracing::warn!(
                    error=?err,
                    "failed to build glob walker for base-directory:{:?} pattern:{}",
                    base_dir.as_ref(), pattern.as_ref()
                );
                err
            })
            .ok()
    }

    /// Searches for a configuration resource file in the given directories.
    ///
    /// Returns the first directory where the resource file is found.
    fn find_resource_dir(resource: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
        for d in dirs.iter() {
            let walker = Self::make_glob_walker(d, format!("{}.*", resource));
            let walker = match walker {
                Some(w) => w,
                None => continue,
            };

            let found = walker
                .into_iter()
                .filter_map(|entry| {
                    tracing::info!("found settings entry: {entry:?}");
                    Result::ok(entry)
                })
                .next()
                .is_some();
            if found {
                tracing::info!("found settings {resource} file in base-directory:{d:?}");
                return Some(d.clone());
            }
        }

        tracing::warn!("no settings resource found for {resource} in {dirs:?}");
        None
    }

    /// Creates a configuration source for an explicitly specified settings file.
    ///
    /// This method is used when the user provides a configuration file path via CLI.
    fn make_explicit_config_source(path: &Path) -> ConfigFile {
        ConfigFile::from(path).required(true)
    }

    /// Creates a configuration source for implicitly loaded application settings.
    ///
    /// The source file is determined by searching the provided directories for a file
    /// matching the `app_config_basename()` (e.g., `application.yaml`).
    fn make_implicit_config_source(basename: &str, dir_paths: &[PathBuf]) -> ConfigFile {
        let source_dir = Self::find_resource_dir(basename, dir_paths)
            // .cloned()
            .unwrap_or_else(Self::default_resource_path);

        let path = source_dir.join(basename);
        ConfigFile::from(path).required(true)
    }

    /// Generates a list of environment-specific configuration sources.
    ///
    /// This function looks for files named after the environment (e.g., `production.yaml`)
    /// in the provided search directories.
    fn make_environment_sources(environment: Environment, dir_paths: &[PathBuf]) -> Vec<ConfigFile> {
        dir_paths
            .iter()
            .rev()
            .map(|dir| Self::make_app_environment_source(&environment, dir))
            .collect()
    }

    /// Creates a configuration source for an environment-specific settings file.
    fn make_app_environment_source(environment: &Environment, resources: &Path) -> ConfigFile {
        tracing::info!("creating application {environment} settings source at {:?}", resources);
        let env_path = resources.join(environment.as_ref());
        ConfigFile::from(env_path).required(false)
    }

    // Creates a configuration source for a secrets file.
    ///
    /// The secrets file contains sensitive credentials such as database passwords.
    fn make_secrets_source(secrets_path: &Path) -> ConfigFile {
        if secrets_path.exists() {
            tracing::info!("adding secrets override configuration source at {:?}", secrets_path);
        } else {
            tracing::error!("cannot find secrets override configuration at {:?}", secrets_path);
        }
        ConfigFile::from(secrets_path).required(true)
    }

    /// Creates a configuration source that pulls settings from environment variables.
    ///
    /// The environment variables must follow the prefix and separator rules defined
    /// by `environment_prefix()` and `environment_path_separator()`.
    fn make_environment_variables_source() -> config::Environment {
        let prefix = Self::environment_prefix();
        let delim = Self::environment_path_separator();
        let config_env = config::Environment::with_prefix(prefix).separator(delim);
        tracing::info!("loading environment variables with: {:?}", config_env);
        config_env
    }

    /// Adds an environment-specific configuration source to the settings builder.
    #[tracing::instrument(level = "info", skip(config))]
    fn add_app_environment_source(
        config: ConfigBuilder<DefaultState>, env: Environment, resources_path: &Path,
    ) -> ConfigBuilder<DefaultState> {
        let env_config_path = resources_path.join(env.as_ref());
        tracing::debug!("looking for {} configu at {:?}", env, resources_path);
        config.add_source(config::File::from(env_config_path).required(false))
    }

    /// Adds a secrets file source to the settings builder.
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

    /// Adds environment variables as a source to the settings builder.
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
    use super::*;
    use crate::{environment, NoOptions, APP_ENVIRONMENT};
    use assert_matches2::{assert_let, assert_matches};
    use config::{Config, FileFormat};
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};
    use serde_with::{serde_as, DisplayFromStr};

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
            self.1.clone()
        }

        #[tracing::instrument(level = "info", skip(config))]
        fn load_overrides(
            &self, config: ConfigBuilder<DefaultState>,
        ) -> Result<ConfigBuilder<DefaultState>, Self::Error> {
            Ok(config.set_override("foo", self.0.as_str())?)
        }

        fn implicit_search_paths(&self) -> Vec<PathBuf> {
            vec!["./tests/override".into(), "resources".into()]
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
        #[serde(rename = "name")]
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

                assert_matches!(env::var(APP_ENVIRONMENT), Err(_));
                tracing::info!("envar: {} = {:?}", APP_ENVIRONMENT, env::var(APP_ENVIRONMENT));

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
                    |   name: "propensity"
                    |   require_ssl: true
                "###
                    .trim_margin_with("| ")
                    .unwrap()
                    .as_str(),
                    FileFormat::Yaml,
                ));

                let options = TestOptions("bar".to_string(), Some(environment::LOCAL.clone()));

                assert_let!(Ok(config) = options.load_overrides(config));
                assert_let!(Ok(config) = config.build());
                tracing::info!(?config, "eligibility config loaded.");

                assert_let!(Ok(actual) = config.try_deserialize::<TestSettings>());
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
            vec![
                (APP_ENVIRONMENT, Some("local")),
                ("APP__APPLICATION__PORT", Some("80")),
                ("APP__DATABASE__PASSWORD", Some("my voice is my password")),
                ("APP__DATABASE__PORT", Some("1111")),
            ],
            || {
                eprintln!("+ test_settings_load_w_options");
                Lazy::force(&TEST_TRACING);
                let main_span = tracing::info_span!("test_settings_load_w_options");
                let _ = main_span.enter();

                assert_matches!(env::var(APP_ENVIRONMENT), Ok(actual));
                assert_eq!(actual, "local");
                tracing::info!("envar: {} = {:?}", APP_ENVIRONMENT, std::env::var(APP_ENVIRONMENT));

                assert_let!(Ok(actual) = TestSettings::load(&TestOptions("zed".to_string(), None)));

                let expected: TestSettings = TestSettings {
                    application: TestHttpSettings {
                        port: 80,
                        host: "127.0.0.1".to_string(),
                        // base_url: "http://127.0.0.1".to_string(),
                    },
                    database: TestDbSettings {
                        username: "postgres".to_string(),
                        password: "my voice is my password".to_string(),
                        port: 1111,
                        host: "localhost".to_string(),
                        database_name: "local_db".to_string(),
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
                assert_matches!(env::var(APP_ENVIRONMENT), Ok(actual));
                assert_eq!(actual, "production");
                tracing::info!("envar: {} = {:?}", APP_ENVIRONMENT, std::env::var(APP_ENVIRONMENT));

                assert_let!(Ok(actual) = TestSettingsNoOpts::load(&()));

                let expected = TestSettingsNoOpts {
                    application: TestHttpSettings {
                        port: 8000,
                        host: "0.0.0.0".to_string(),
                        // base_url: "http://127.0.0.1".to_string(),
                    },
                    database: TestDbSettings {
                        username: "postgres".to_string(),
                        password: "resources".to_string(),
                        port: 5432,
                        host: "localhost".to_string(),
                        database_name: "default_db".to_string(),
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

    #[test]
    fn test_settings_load_w_override() -> anyhow::Result<()> {
        eprintln!("+ test_settings_load_w_override");
        Lazy::force(&TEST_TRACING);
        let main_span = tracing::info_span!("test_settings_load_w_override");
        let _ = main_span.enter();

        with_env_vars(
            "test_settings_load_w_override",
            vec![(APP_ENVIRONMENT, Some("production"))],
            || {
                assert_matches!(env::var(APP_ENVIRONMENT), Ok(actual));
                assert_eq!(actual, "production");
                tracing::info!("envar: {} = {:?}", APP_ENVIRONMENT, std::env::var(APP_ENVIRONMENT));

                assert_let!(Ok(actual) = TestSettings::load(&TestOptions("zed".to_string(), None)));

                let expected = TestSettings {
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
                        database_name: "override".to_string(),
                        require_ssl: false,
                    },
                    foo: "zed".to_string(),
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

    static SERIAL_TEST: Lazy<Mutex<()>> = Lazy::new(Default::default);

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
