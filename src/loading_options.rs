use std::path::PathBuf;

use config::builder::DefaultState;
use config::ConfigBuilder;

use crate::environment::Environment;
use crate::error::SettingsError;
use crate::layer::LayerBuilder;

pub const APP_ENVIRONMENT: &str = "APP_ENVIRONMENT";

/// Defines the contract for specifying how configuration settings are loaded.
///
/// The `LoadingOptions` trait provides a flexible way to define where and how configuration
/// settings should be retrieved. It enables applications to specify explicit configuration files,
/// secrets files, search paths, and environment overrides, ensuring that configurations can be
/// dynamically loaded from various sources.
///
/// The `SettingsLoader::load()` function serves as the primary driver of the settings loading
/// process, but it relies on `LoadingOptions` to determine the details of where and how to fetch
/// configuration values.
///
/// Implementing this trait allows users to:
/// - Specify an explicit configuration file.
/// - Define the location of a secrets file.
/// - Configure search paths for implicit configuration loading.
/// - Apply additional override mechanisms.
/// - Resolve the active environment.
///
/// ## Usage Example
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use settings_loader::{Environment, LoadingOptions, SettingsError};
///
/// struct CliOptions {
///     config: Option<PathBuf>,
///     secrets: Option<PathBuf>,
///     environment: Option<Environment>,
/// }
///
/// impl LoadingOptions for CliOptions {
///     type Error = SettingsError;
///
///     fn config_path(&self) -> Option<PathBuf> {
///         self.config.clone()
///     }
///
///     fn secrets_path(&self) -> Option<PathBuf> {
///         self.secrets.clone()
///     }
///
///     fn implicit_search_paths(&self) -> Vec<PathBuf> {
///         vec![PathBuf::from("./config")]
///     }
/// }
/// ```
pub trait LoadingOptions: Sized {
    /// The error type that can be returned from configuration operations.
    type Error: std::error::Error + From<SettingsError> + Sync + Send + 'static;

    /// Returns the path to an explicit configuration file, if provided.
    ///
    /// If a configuration file is specified via the command-line or other means,
    /// this function should return its path. If `None` is returned, the system will
    /// attempt to infer configuration from default locations.
    fn config_path(&self) -> Option<PathBuf>;

    /// Returns the path to a secrets file, if specified.
    ///
    /// This is useful for separating sensitive credentials (e.g., database passwords)
    /// from the main configuration files.
    fn secrets_path(&self) -> Option<PathBuf>;

    /// Returns a list of directories to search for configuration files.
    ///
    /// If an explicit configuration file is not provided, this function determines
    /// which directories will be scanned for inferred configuration files.
    fn implicit_search_paths(&self) -> Vec<PathBuf>;

    /// Allows customization of the configuration before finalization.
    ///
    /// This method provides an opportunity to apply additional runtime overrides,
    /// such as modifying individual settings dynamically.
    ///
    /// The default implementation returns the configuration unchanged.
    fn load_overrides(&self, config: ConfigBuilder<DefaultState>) -> Result<ConfigBuilder<DefaultState>, Self::Error> {
        Ok(config)
    }

    /// Determines the active environment for configuration resolution.
    ///
    /// This function checks for an explicit environment override, falling back to
    /// an environment variable lookup (`APP_ENVIRONMENT`). If no value is found,
    /// it logs a warning and defaults to `None`.
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

    /// Provides an optional explicit override for the environment.
    ///
    /// This can be used to manually specify the environment without relying
    /// on environment variables.
    fn environment_override(&self) -> Option<Environment> {
        None
    }

    /// Returns the environment variable key used to determine the application environment.
    ///
    /// Defaults to `"APP_ENVIRONMENT"`.
    fn env_app_environment() -> &'static str {
        APP_ENVIRONMENT
    }

    /// Returns the prefix for environment variables.
    ///
    /// Default prefix is `"APP"`. Override to customize environment variable naming convention.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use settings_loader::LoadingOptions;
    ///
    /// struct TurtleOptions;
    ///
    /// impl LoadingOptions for TurtleOptions {
    ///     type Error = settings_loader::SettingsError;
    ///     
    ///     fn config_path(&self) -> Option<PathBuf> { None }
    ///     fn secrets_path(&self) -> Option<PathBuf> { None }
    ///     fn implicit_search_paths(&self) -> Vec<PathBuf> { Vec::new() }
    ///     
    ///     fn env_prefix() -> &'static str {
    ///         "TURTLE"  // Override to use TURTLE__* convention
    ///     }
    /// }
    /// ```
    fn env_prefix() -> &'static str {
        "APP"
    }

    /// Returns the separator for nested environment variable keys.
    ///
    /// Default separator is `"__"` (double underscore). Override to customize how nested
    /// configuration keys map to environment variable names.
    ///
    /// # Example
    ///
    /// With default separator `"__"`:
    /// - `APP__DATABASE__HOST` maps to `database.host`
    ///
    /// With custom separator `"_"`:
    /// - `APP_DATABASE_HOST` maps to `database.host`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use settings_loader::LoadingOptions;
    ///
    /// struct CustomOptions;
    ///
    /// impl LoadingOptions for CustomOptions {
    ///     type Error = settings_loader::SettingsError;
    ///     
    ///     fn config_path(&self) -> Option<PathBuf> { None }
    ///     fn secrets_path(&self) -> Option<PathBuf> { None }
    ///     fn implicit_search_paths(&self) -> Vec<PathBuf> { Vec::new() }
    ///     
    ///     fn env_separator() -> &'static str {
    ///         "_"  // Override to use single underscore
    ///     }
    /// }
    /// ```
    fn env_separator() -> &'static str {
        "__"
    }

    /// Build explicit configuration layers.
    ///
    /// Default implementation returns builder unchanged (backward compatible).
    /// Override to define explicit layer composition.
    fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
        builder
    }
}

/// Trait for multi-scope configuration support with automatic path resolution.
///
/// This trait extends `LoadingOptions` to support loading configuration from multiple
/// scopes (System, UserGlobal, ProjectLocal, Runtime) with automatic, platform-specific
/// path resolution.
///
/// # Scopes
///
/// - `System` - Immutable system-wide defaults (platform-specific location)
/// - `UserGlobal` - User preferences that apply everywhere (platform-specific location)
/// - `ProjectLocal` - Project-specific overrides (current directory)
/// - `Runtime` - Dynamic configuration from environment variables and CLI
///
/// # Constants
///
/// - `APP_NAME` - Required: The application name (e.g., "my-app")
/// - `ORG_NAME` - Optional: Organization name for platform path resolution
/// - `CONFIG_BASENAME` - Optional: Base name for config files (default: "settings")
///
/// # Platform Path Resolution
///
/// UserGlobal paths use platform conventions:
/// - **Linux**: `~/.config/{APP_NAME}/settings.{ext}` (XDG Base Directory spec)
/// - **macOS**: `~/Library/Application Support/{APP_NAME}/settings.{ext}`
/// - **Windows**: `%APPDATA%/{APP_NAME}/settings.{ext}`
///
/// # Example
///
/// ```ignore
/// use settings_loader::{LoadingOptions, MultiScopeConfig, ConfigScope};
/// use std::path::{Path, PathBuf};
///
/// struct TurtleOptions;
///
/// impl LoadingOptions for TurtleOptions {
///     type Error = settings_loader::SettingsError;
///     fn config_path(&self) -> Option<PathBuf> { None }
///     fn secrets_path(&self) -> Option<PathBuf> { None }
///     fn implicit_search_paths(&self) -> Vec<PathBuf> { Vec::new() }
/// }
///
/// impl MultiScopeConfig for TurtleOptions {
///     const APP_NAME: &'static str = "spark-turtle";
///     const ORG_NAME: &'static str = "spark-turtle";
///     const CONFIG_BASENAME: &'static str = "settings";
///
///     fn find_config_in(dir: &Path) -> Option<PathBuf> {
///         crate::scope::find_config_in(dir)
///     }
/// }
/// ```
pub trait MultiScopeConfig: LoadingOptions {
    /// The application name used for path resolution (e.g., "my-app")
    const APP_NAME: &'static str;

    /// Optional organization name for platform-specific paths
    const ORG_NAME: &'static str = "";

    /// Base name for configuration files (default: "settings")
    const CONFIG_BASENAME: &'static str = "settings";

    /// Resolve a configuration path for the given scope.
    ///
    /// Returns the resolved path if the configuration file exists for that scope,
    /// or `None` if the file doesn't exist or the scope is not file-based (Runtime).
    ///
    /// # Default Behavior
    ///
    /// The default implementation resolves paths based on scope:
    /// - `System` - Platform-specific system path
    /// - `UserGlobal` - Platform-specific user path using `directories` crate
    /// - `ProjectLocal` - Current directory search
    /// - `Runtime` - None (not file-based)
    fn resolve_path(scope: crate::ConfigScope) -> Option<PathBuf> {
        use crate::ConfigScope;

        match scope {
            ConfigScope::Preferences => Self::preferences_path(),
            ConfigScope::UserGlobal => Self::user_global_path(),
            ConfigScope::ProjectLocal => Self::project_local_path(),
            ConfigScope::LocalData => Self::local_data_path(),
            ConfigScope::PersistentData => Self::persistent_data_path(),
            ConfigScope::Runtime => None,
        }
    }

    /// Resolve the preferences configuration path.
    ///
    /// Platform-specific implementations using `directories` crate:
    /// - **Linux**: `~/.config/APP_NAME/settings.{ext}` (or XDG_CONFIG_HOME/APP_NAME)
    /// - **macOS**: `~/Library/Preferences/APP_NAME/settings.{ext}`
    /// - **Windows**: `%APPDATA%/APP_NAME/settings.{ext}`
    ///
    /// # Requires Feature Flag
    ///
    /// This method requires the `multi-scope` feature (which enables the `directories` crate).
    ///
    /// # Returns
    ///
    /// `Some(PathBuf)` if configuration file exists in preferences path, `None` otherwise.
    #[cfg(feature = "multi-scope")]
    fn preferences_path() -> Option<PathBuf> {
        use directories::BaseDirs;

        let dirs = BaseDirs::new()?;
        let pref_dir = dirs.preference_dir();
        let app_dir = pref_dir.join(Self::APP_NAME);
        Self::find_config_in(&app_dir)
    }

    #[cfg(not(feature = "multi-scope"))]
    fn preferences_path() -> Option<PathBuf> {
        None
    }

    /// Resolve the user global configuration path.
    ///
    /// Platform-specific implementations:
    /// - **Linux**: `~/.config/{APP_NAME}/settings.{ext}` (XDG_CONFIG_HOME)
    /// - **macOS**: `~/Library/Application Support/{APP_NAME}/settings.{ext}`
    /// - **Windows**: `%APPDATA%/{APP_NAME}/settings.{ext}`
    ///
    /// # Requires Feature Flag
    ///
    /// This method requires the `multi-scope` feature (which enables the `directories` crate).
    ///
    /// # Returns
    ///
    /// `Some(PathBuf)` if configuration file exists in user path, `None` otherwise.
    #[cfg(feature = "multi-scope")]
    fn user_global_path() -> Option<PathBuf> {
        use directories::ProjectDirs;

        let proj = ProjectDirs::from(Self::ORG_NAME, Self::ORG_NAME, Self::APP_NAME)?;
        let config_dir = proj.config_dir();
        Self::find_config_in(config_dir)
    }

    #[cfg(not(feature = "multi-scope"))]
    fn user_global_path() -> Option<PathBuf> {
        // Without directories crate, we can't reliably resolve platform paths
        None
    }

    /// Resolve the project-local configuration path.
    ///
    /// Searches for configuration files in the current directory.
    /// Files are searched in order of format preference: TOML > YAML > JSON > etc.
    ///
    /// # Returns
    ///
    /// `Some(PathBuf)` if configuration file exists in current directory, `None` otherwise.
    fn project_local_path() -> Option<PathBuf> {
        let current_dir = std::env::current_dir().ok()?;
        Self::find_config_in(&current_dir)
    }

    /// Resolve the local data configuration path.
    ///
    /// Platform-specific implementations using `directories` crate:
    /// - **Linux**: `~/.cache/APP_NAME/` (or XDG_CACHE_HOME/APP_NAME)
    /// - **macOS**: `~/Library/Caches/ORG_NAME.APP_NAME/`
    /// - **Windows**: `%LOCALAPPDATA%/ORG_NAME/APP_NAME/`
    ///
    /// Use for machine-local data that is not synced across machines.
    ///
    /// # Requires Feature Flag
    ///
    /// This method requires the `multi-scope` feature (which enables the `directories` crate).
    ///
    /// # Returns
    ///
    /// `Some(PathBuf)` if configuration file exists in local data path, `None` otherwise.
    #[cfg(feature = "multi-scope")]
    fn local_data_path() -> Option<PathBuf> {
        use directories::BaseDirs;

        let dirs = BaseDirs::new()?;
        let data_dir = dirs.data_local_dir();
        let app_dir = data_dir.join(Self::APP_NAME);
        Self::find_config_in(&app_dir)
    }

    #[cfg(not(feature = "multi-scope"))]
    fn local_data_path() -> Option<PathBuf> {
        None
    }

    /// Resolve the persistent data configuration path.
    ///
    /// Platform-specific implementations using `directories` crate:
    /// - **Linux**: `~/.local/share/APP_NAME/` (or XDG_DATA_HOME/APP_NAME)
    /// - **macOS**: `~/Library/Application Support/ORG_NAME.APP_NAME/`
    /// - **Windows**: `%APPDATA%/ORG_NAME/APP_NAME/`
    ///
    /// Use for persistent data that can be synced across machines.
    ///
    /// # Requires Feature Flag
    ///
    /// This method requires the `multi-scope` feature (which enables the `directories` crate).
    ///
    /// # Returns
    ///
    /// `Some(PathBuf)` if configuration file exists in persistent data path, `None` otherwise.
    #[cfg(feature = "multi-scope")]
    fn persistent_data_path() -> Option<PathBuf> {
        use directories::BaseDirs;

        let dirs = BaseDirs::new()?;
        let data_dir = dirs.data_dir();
        let app_dir = data_dir.join(Self::APP_NAME);
        Self::find_config_in(&app_dir)
    }

    #[cfg(not(feature = "multi-scope"))]
    fn persistent_data_path() -> Option<PathBuf> {
        None
    }

    /// Search for a configuration file with multiple format extensions.
    ///
    /// This method must be implemented to provide the file discovery logic.
    /// Most implementations should delegate to `crate::scope::find_config_in()`.
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory to search for configuration files
    ///
    /// # Returns
    ///
    /// The first matching configuration file found, or `None` if no file matches.
    fn find_config_in(dir: &std::path::Path) -> Option<PathBuf>;

    /// Get the list of scopes to load in precedence order.
    ///
    /// Default order: Preferences → UserGlobal → ProjectLocal → LocalData → PersistentData
    /// (Runtime is handled separately via environment variables).
    /// Override to customize scope loading order.
    fn default_scopes() -> Vec<crate::ConfigScope> {
        use crate::ConfigScope;

        vec![
            ConfigScope::Preferences,
            ConfigScope::UserGlobal,
            ConfigScope::ProjectLocal,
            ConfigScope::LocalData,
            ConfigScope::PersistentData,
        ]
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

    fn implicit_search_paths(&self) -> Vec<PathBuf> {
        Vec::default()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir; // Import tempdir for use in tests

    struct MockOptions {
        config: Option<PathBuf>,
        secrets: Option<PathBuf>,
        env_override: Option<Environment>,
    }

    impl LoadingOptions for MockOptions {
        type Error = SettingsError;

        fn config_path(&self) -> Option<PathBuf> {
            self.config.clone()
        }

        fn secrets_path(&self) -> Option<PathBuf> {
            self.secrets.clone()
        }

        fn implicit_search_paths(&self) -> Vec<PathBuf> {
            vec![PathBuf::from("/etc/app"), PathBuf::from("./config")]
        }

        fn environment_override(&self) -> Option<Environment> {
            self.env_override.clone()
        }
    }

    impl MultiScopeConfig for MockOptions {
        const APP_NAME: &'static str = "test-app";
        const ORG_NAME: &'static str = "test-org";

        fn find_config_in(dir: &Path) -> Option<PathBuf> {
            let path = dir.join("settings.toml");
            if path.exists() {
                Some(path)
            } else {
                None
            }
        }
    }

    #[test]
    fn test_loading_options_default_impl() {
        let opts = MockOptions {
            config: Some(PathBuf::from("config.toml")),
            secrets: Some(PathBuf::from("secrets.toml")),
            env_override: Some(Environment::from("development")),
        };

        assert_eq!(opts.config_path(), Some(PathBuf::from("config.toml")));
        assert_eq!(opts.secrets_path(), Some(PathBuf::from("secrets.toml")));
        assert_eq!(opts.implicit_search_paths().len(), 2);
        assert_eq!(opts.environment_override(), Some(Environment::from("development")));
        assert_eq!(MockOptions::env_prefix(), "APP");
        assert_eq!(MockOptions::env_separator(), "__");
    }

    #[test]
    fn test_multi_scope_config_default_scopes() {
        let scopes = MockOptions::default_scopes();
        assert_eq!(scopes.len(), 5);
        assert!(scopes.contains(&crate::ConfigScope::Preferences));
        assert!(scopes.contains(&crate::ConfigScope::ProjectLocal));
    }

    #[test]
    fn test_no_options_impl() {
        let opts: NoOptions = ();
        assert_eq!(opts.config_path(), None);
        assert_eq!(opts.secrets_path(), None);
        assert!(opts.implicit_search_paths().is_empty());
    }

    #[test]
    fn test_multi_scope_resolve_path_runtime() {
        assert_eq!(MockOptions::resolve_path(crate::ConfigScope::Runtime), None);
    }

    #[test]
    #[cfg(not(feature = "multi-scope"))]
    fn test_multi_scope_resolve_path_preferences_no_feature() {
        assert_eq!(MockOptions::resolve_path(crate::ConfigScope::Preferences), None);
    }

    #[test]
    #[cfg(not(feature = "multi-scope"))]
    fn test_multi_scope_resolve_path_user_global_no_feature() {
        assert_eq!(MockOptions::resolve_path(crate::ConfigScope::UserGlobal), None);
    }

    #[test]
    #[cfg(not(feature = "multi-scope"))]
    fn test_multi_scope_resolve_path_local_data_no_feature() {
        assert_eq!(MockOptions::resolve_path(crate::ConfigScope::LocalData), None);
    }

    #[test]
    #[cfg(not(feature = "multi-scope"))]
    fn test_multi_scope_resolve_path_persistent_data_no_feature() {
        assert_eq!(MockOptions::resolve_path(crate::ConfigScope::PersistentData), None);
    }

    #[test]
    fn test_multi_scope_resolve_path_project_local() {
        use std::fs;
        use tempfile::tempdir;

        // Create a temporary directory and a settings.toml file in it
        let dir = tempdir().unwrap();
        let current_dir_backup = std::env::current_dir().unwrap();

        let file_path_in_temp = dir.path().join("settings.toml");
        fs::write(&file_path_in_temp, "key = \"value\"").unwrap();

        std::env::set_current_dir(dir.path()).unwrap();

        // Use a consistent canonicalized path for the assertion
        let expected_canonical_path = file_path_in_temp.canonicalize().unwrap();

        let resolved_path = MockOptions::resolve_path(crate::ConfigScope::ProjectLocal);
        assert!(resolved_path.is_some(), "Resolved path should not be None");
        assert_eq!(resolved_path.unwrap().canonicalize().unwrap(), expected_canonical_path);

        std::env::set_current_dir(current_dir_backup).unwrap(); // Restore original current directory
    }

    #[test]
    fn test_multi_scope_resolve_path_project_local_no_file() {
        let dir = tempdir().unwrap();
        let current_dir_backup = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();

        let resolved_path = MockOptions::resolve_path(crate::ConfigScope::ProjectLocal);
        assert_eq!(resolved_path, None);

        std::env::set_current_dir(current_dir_backup).unwrap();
    }

    #[test]
    fn test_environment_with_override() {
        let opts = MockOptions {
            config: None,
            secrets: None,
            env_override: Some(Environment::from("test_override")),
        };
        assert_eq!(opts.environment(), Some(Environment::from("test_override")));
    }

    #[test]
    fn test_environment_with_env_var() {
        // Ensure APP_ENVIRONMENT is not set initially for a clean test
        std::env::remove_var("APP_ENVIRONMENT");
        std::env::set_var("APP_ENVIRONMENT", "test_env_var");

        let opts = MockOptions { config: None, secrets: None, env_override: None };
        assert_eq!(opts.environment(), Some(Environment::from("test_env_var")));
        std::env::remove_var("APP_ENVIRONMENT"); // Clean up
    }

    #[test]
    fn test_environment_no_override_no_env_var() {
        std::env::remove_var("APP_ENVIRONMENT"); // Ensure it's not set

        let opts = MockOptions { config: None, secrets: None, env_override: None };
        // It should warn and return None. We can't easily capture logs, so just check None.
        assert_eq!(opts.environment(), None);
    }

    #[test]
    fn test_environment_env_var_with_custom_name() {
        std::env::remove_var("CUSTOM_APP_ENV"); // Clean up any previous run
        std::env::set_var("CUSTOM_APP_ENV", "custom_env");

        struct CustomEnvOptions;
        impl LoadingOptions for CustomEnvOptions {
            type Error = SettingsError;
            fn config_path(&self) -> Option<PathBuf> {
                None
            }
            fn secrets_path(&self) -> Option<PathBuf> {
                None
            }
            fn implicit_search_paths(&self) -> Vec<PathBuf> {
                Vec::new()
            }
            fn env_app_environment() -> &'static str {
                "CUSTOM_APP_ENV"
            }
        }

        let opts = CustomEnvOptions;
        assert_eq!(opts.environment(), Some(Environment::from("custom_env")));
        std::env::remove_var("CUSTOM_APP_ENV"); // Clean up
    }
}
