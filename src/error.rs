use thiserror::Error;

/// Error variants related to configuration.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SettingsError {
    /// Error working with environment variable
    #[error("{0}")]
    Environment(#[from] std::env::VarError),

    /// Error in configuration common.
    #[error(transparent)]
    Configuration(#[from] config::ConfigError),

    #[error("failed to load option overrides into settings: {0}")]
    CliOptionError(#[from] anyhow::Error),

    #[error("{0}")]
    IOError(#[from] std::io::Error),

    /// Error in bootstrapping execution from configuration.
    #[error("error during system bootstrap: {message}: {setting}")]
    Bootstrap { message: String, setting: String },
}
