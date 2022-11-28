use std::fmt;
use std::time::Duration;

use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_with::serde_as;
use sqlx::pool::PoolOptions;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,

    pub password: Secret<String>,

    pub host: String,

    pub port: u16,

    pub database_name: String,

    pub require_ssl: bool,

    #[serde(default)]
    pub min_connections: Option<u32>,

    #[serde(default)]
    pub max_connections: Option<u32>,

    #[serde(default, alias = "max_lifetime_secs")]
    #[serde_as(as = "Option<serde_with::DurationSeconds<u64>>")]
    pub max_lifetime: Option<Duration>,

    #[serde(default, alias = "idle_timeout_secs")]
    #[serde_as(as = "Option<serde_with::DurationSeconds<u64>>")]
    pub idle_timeout: Option<Duration>,
}

impl DatabaseSettings {
    pub fn pg_pool_options(&self) -> PgPoolOptions {
        let mut options = PgPoolOptions::new()
            .max_lifetime(self.max_lifetime)
            .idle_timeout(self.idle_timeout);

        if let Some(min) = self.min_connections {
            options = options.min_connections(min);
        }

        if let Some(max) = self.max_connections {
            options = options.max_connections(max);
        }

        options
    }

    pub fn pg_connect_options_without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl { PgSslMode::Require } else { PgSslMode::Prefer };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

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
            && self.min_connections == other.min_connections
            && self.max_connections == other.max_connections
            && self.max_lifetime == other.max_lifetime
            && self.idle_timeout == other.idle_timeout
            && self.password.expose_secret() == other.password.expose_secret()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use claim::*;
    use pretty_assertions::assert_eq;
    use trim_margin::MarginTrimmable;

    #[test]
    fn test_password_redaction() {
        let settings = DatabaseSettings {
            username: "Billy".to_string(),
            password: Secret::new("my-secret".to_string()),
            port: 1234,
            host: "localhost".to_string(),
            database_name: "db_name".to_string(),
            require_ssl: true,
            min_connections: None,
            max_connections: None,
            max_lifetime: None,
            idle_timeout: None,
        };

        let actual = format!("{:?}", settings);
        assert_eq!(
            actual,
            r##"DatabaseSettings { username: "Billy", password: Secret([REDACTED alloc::string::String]), host: "localhost", port: 1234, database_name: "db_name", require_ssl: true, min_connections: None, max_connections: None, max_lifetime: None, idle_timeout: None }"##
        )
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
            |idle_timeout_secss: 180
            |"##
        .trim_margin()
        .unwrap();

        let from_yaml: DatabaseSettings = assert_ok!(serde_yaml::from_str(yaml.as_str()));
        assert_eq!(from_yaml.username, "Billy".to_string(),);
        assert_eq!(from_yaml.port, 1234,);
        assert_eq!(from_yaml.host, "localhost".to_string(),);
        assert_eq!(from_yaml.database_name, "db_name".to_string(),);
        assert_eq!(from_yaml.require_ssl, true,);
        assert_none!(from_yaml.min_connections);
        assert_eq!(assert_some!(from_yaml.max_connections), 10);
        assert_none!(from_yaml.max_lifetime);
        assert_eq!(assert_some!(from_yaml.idle_timeout), Duration::from_secs(180));
    }
}
