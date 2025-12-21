use std::fmt;
use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_with::serde_as;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};

/// Database configuration settings and connection options.
///
/// This module provides the [`DatabaseSettings`] struct, which defines the
/// necessary parameters for configuring a PostgreSQL database connection.
/// It supports various connection parameters, including connection pooling,
/// timeouts, SSL requirements, and credential management.
///
/// # Feature Flag
///
/// This module is only available when the `database` feature is enabled.
/// To enable it, add the following to your `Cargo.toml`:
///
/// ```toml
/// [dependencies]
/// my_crate = { version = "0.1", features = ["database"] }
/// ```
///
/// # Configuration
///
/// The [`DatabaseSettings`] struct can be deserialized from configuration
/// files (e.g., YAML, JSON, TOML) and supports optional parameters for
/// connection pooling and timeouts.
///
/// Example configuration in **YAML**:
///
/// ```yaml
/// database:
///   username: postgres
///   password: my_secure_password
///   host: localhost
///   port: 5432
///   database_name: app_db
///   require_ssl: false
///   max_connections: 10
///   acquire_timeout_secs: 30
/// ```
///
/// # Usage
///
/// ```rust, ignore
/// use settings_loader::database::DatabaseSettings;
/// use secrecy::ExposeSecret;
///
/// let settings = DatabaseSettings {
///     username: "postgres".to_string(),
///     password: secrecy::SecretString::from("password"),
///     host: "localhost".to_string(),
///     port: 5432,
///     database_name: "app_db".to_string(),
///     require_ssl: false,
///     min_connections: Some(5),
///     max_connections: Some(10),
///     max_lifetime: None,
///     acquire_timeout: Some(std::time::Duration::from_secs(30)),
///     idle_timeout: None,
/// };
///
/// let conn_str = settings.connection_string();
/// println!("Database connection string: {}", conn_str.expose_secret());
/// ```
#[serde_as]
#[derive(Clone, Deserialize)]
pub struct DatabaseSettings {
    /// Username for database authentication.
    pub username: String,

    /// Password for database authentication (stored securely).
    pub password: SecretString,

    /// Hostname or IP address of the database server.
    pub host: String,

    /// Port number to connect to the database.
    pub port: u16,

    /// Name of the database to connect to.
    pub database_name: String,

    /// Whether to enforce SSL for the database connection.
    pub require_ssl: bool,

    /// Minimum number of connections to maintain in the pool.
    #[serde(default)]
    pub min_connections: Option<u32>,

    /// Maximum number of connections allowed in the pool.
    #[serde(default)]
    pub max_connections: Option<u32>,

    /// Maximum lifetime of a connection before it is recycled.
    #[serde(default, alias = "max_lifetime_secs")]
    #[serde_as(as = "Option<serde_with::DurationSeconds<u64>>")]
    pub max_lifetime: Option<Duration>,

    /// Timeout duration for acquiring a connection.
    #[serde(default, alias = "acquire_timeout_secs")]
    #[serde_as(as = "Option<serde_with::DurationSeconds<u64>>")]
    pub acquire_timeout: Option<Duration>,

    /// Maximum idle duration before a connection is closed.
    #[serde(default, alias = "idle_timeout_secs")]
    #[serde_as(as = "Option<serde_with::DurationSeconds<u64>>")]
    pub idle_timeout: Option<Duration>,
}

impl DatabaseSettings {
    /// Generates a connection string for PostgreSQL.
    ///
    /// # Example
    /// ```rust,ignore
    /// let settings = DatabaseSettings { ... };
    /// let conn_str = settings.connection_string();
    /// println!("Database URL: {:?}", conn_str);
    /// ```
    pub fn connection_string(&self) -> SecretString {
        let connection = format!(
            "postgres://{db_user}:{db_password}@{host}:{port}/{db_name}",
            db_user = self.username,
            db_password = self.password.expose_secret(),
            host = self.host,
            port = self.port,
            db_name = self.database_name,
        );

        SecretString::from(connection)
    }

    /// Creates a `PgPoolOptions` instance with configured settings.
    ///
    /// This function allows fine-tuning of connection pool behavior, including:
    /// - Minimum and maximum connections.
    /// - Connection acquisition timeouts.
    /// - Idle connection handling.
    pub fn pg_pool_options(&self) -> PgPoolOptions {
        let mut options = PgPoolOptions::new()
            .max_lifetime(self.max_lifetime)
            .idle_timeout(self.idle_timeout);

        if let Some(acquire) = self.acquire_timeout {
            options = options.acquire_timeout(acquire);
        }

        if let Some(min) = self.min_connections {
            options = options.min_connections(min);
        }

        if let Some(max) = self.max_connections {
            options = options.max_connections(max);
        }

        options
    }

    /// Returns PostgreSQL connection options **without specifying a database**.
    ///
    /// This is useful for initial connections where the database might need to be created.
    pub fn pg_connect_options_without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl { PgSslMode::Require } else { PgSslMode::Prefer };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    /// Returns PostgreSQL connection options **with the database specified**.
    pub fn pg_connect_options_with_db(&self) -> PgConnectOptions {
        self.pg_connect_options_without_db().database(&self.database_name)
    }
}

impl fmt::Debug for DatabaseSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatabaseSettings")
            .field("username", &self.username)
            .field("password", &self.password)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database_name", &self.database_name)
            .field("require_ssl", &self.require_ssl)
            .field("min_connections", &self.min_connections)
            .field("max_connections", &self.max_connections)
            .field("max_lifetime", &self.max_lifetime)
            .field("acquire_timeout", &self.acquire_timeout)
            .field("idle_timeout", &self.idle_timeout)
            .finish()
    }
}

impl PartialEq for DatabaseSettings {
    fn eq(&self, other: &Self) -> bool {
        self.require_ssl == other.require_ssl
            && self.port == other.port
            && self.host == other.host
            && self.username == other.username
            && self.database_name == other.database_name
            && self.password.expose_secret() == other.password.expose_secret()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches2::{assert_let, assert_matches};
    use pretty_assertions::assert_eq;
    use trim_margin::MarginTrimmable;

    #[test]
    fn test_database_equality() {
        let base = DatabaseSettings {
            username: "Billy".to_string(),
            password: SecretString::from("my-secret"),
            port: 1234,
            host: "localhost".to_string(),
            database_name: "db_name".to_string(),
            require_ssl: true,
            min_connections: None,
            max_connections: None,
            max_lifetime: None,
            acquire_timeout: None,
            idle_timeout: None,
        };

        // Same values
        let same = base.clone();
        assert_eq!(base, same);

        // Different field
        let mut diff_user = base.clone();
        diff_user.username = "NotBilly".to_string();
        assert!(base != diff_user);

        let mut diff_pass = base.clone();
        diff_pass.password = SecretString::from("other-secret");
        assert!(base != diff_pass);

        let mut diff_port = base.clone();
        diff_port.port = 4321;
        assert!(base != diff_port);

        let mut diff_host = base.clone();
        diff_host.host = "otherhost".to_string();
        assert!(base != diff_host);

        let mut diff_db = base.clone();
        diff_db.database_name = "other_db".to_string();
        assert!(base != diff_db);

        let mut diff_ssl = base.clone();
        diff_ssl.require_ssl = false;
        assert!(base != diff_ssl);
    }

    #[test]
    fn test_database_connection_string() {
        let settings = DatabaseSettings {
            username: "Billy".to_string(),
            password: SecretString::from("my-secret"),
            port: 1234,
            host: "localhost".to_string(),
            database_name: "db_name".to_string(),
            require_ssl: true,
            min_connections: None,
            max_connections: None,
            max_lifetime: None,
            acquire_timeout: None,
            idle_timeout: None,
        };

        let conn_str = settings.connection_string();
        assert_eq!(
            conn_str.expose_secret(),
            "postgres://Billy:my-secret@localhost:1234/db_name"
        );
    }

    #[test]
    fn test_pg_pool_options() {
        let mut settings = DatabaseSettings {
            username: "Billy".to_string(),
            password: SecretString::from("my-secret"),
            port: 1234,
            host: "localhost".to_string(),
            database_name: "db_name".to_string(),
            require_ssl: true,
            min_connections: Some(2),
            max_connections: Some(20),
            max_lifetime: Some(Duration::from_secs(3600)),
            acquire_timeout: Some(Duration::from_secs(10)),
            idle_timeout: Some(Duration::from_secs(300)),
        };

        // All options set
        let _options = settings.pg_pool_options();
        // Since PgPoolOptions doesn't implement Debug/PartialEq in a helpful way for inspection,
        // we mainly verify it doesn't panic and exercises the code paths.

        // Options none
        settings.min_connections = None;
        settings.max_connections = None;
        settings.max_lifetime = None;
        settings.acquire_timeout = None;
        settings.idle_timeout = None;
        let _options_none = settings.pg_pool_options();
    }

    #[test]
    fn test_pg_connect_options() {
        let settings = DatabaseSettings {
            username: "Billy".to_string(),
            password: SecretString::from("my-secret"),
            port: 1234,
            host: "localhost".to_string(),
            database_name: "db_name".to_string(),
            require_ssl: true,
            min_connections: None,
            max_connections: None,
            max_lifetime: None,
            acquire_timeout: None,
            idle_timeout: None,
        };

        let _opts_no_db = settings.pg_connect_options_without_db();
        let _opts_with_db = settings.pg_connect_options_with_db();
    }

    #[test]
    fn test_database_deser() {
        let yaml = r##"
            |username: Billy
            |password: my-secret
            |port: 1234
            |host: localhost
            |database_name: db_name
            |require_ssl: true
            |max_connections: 10
            |acquire_timeout_secs: 5
            |idle_timeout_secs: 180
            |"##
        .trim_margin()
        .unwrap();

        assert_let!(Ok(from_yaml) = serde_yaml::from_str::<DatabaseSettings>(yaml.as_str()));
        assert_eq!(from_yaml.username, "Billy".to_string(),);
        assert_eq!(from_yaml.port, 1234,);
        assert_eq!(from_yaml.host, "localhost".to_string(),);
        assert_eq!(from_yaml.database_name, "db_name".to_string(),);
        assert_eq!(from_yaml.require_ssl, true,);
        assert_matches!(from_yaml.min_connections, None);
        assert_matches!(from_yaml.max_connections, Some(actual_max));
        assert_eq!(actual_max, 10);
        assert_matches!(from_yaml.max_lifetime, None);
        assert_let!(Some(actual_aquire_timeout) = from_yaml.acquire_timeout);
        assert_eq!(actual_aquire_timeout, Duration::from_secs(5));
        assert_let!(Some(actual_idle_timeout) = from_yaml.idle_timeout);
        assert_eq!(actual_idle_timeout, Duration::from_secs(180));
    }
}
