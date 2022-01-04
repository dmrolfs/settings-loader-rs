use std::fmt;

use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use sqlx::postgres::{PgConnectOptions, PgSslMode};

#[serde_as]
#[derive(Clone, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    #[serde_as(as = "DisplayFromStr")]
    pub port: u16,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl { PgSslMode::Require } else { PgSslMode::Prefer };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
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
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_password_redaction() {
        let settings = DatabaseSettings {
            username: "Billy".to_string(),
            password: Secret::new("my-secret".to_string()),
            port: 1234,
            host: "localhost".to_string(),
            database_name: "db_name".to_string(),
            require_ssl: true,
        };

        let actual = format!("{:?}", settings);
        assert_eq!(
            actual,
            r##"DatabaseSettings { username: "Billy", password: Secret([REDACTED alloc::string::String]), host: "localhost", port: 1234, database_name: "db_name", require_ssl: true }"##
        )
    }
}
