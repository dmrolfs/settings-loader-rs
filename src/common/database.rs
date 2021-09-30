use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::fmt;

#[serde_as]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    #[serde_as(as = "DisplayFromStr")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl { PgSslMode::Require } else { PgSslMode::Prefer };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
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
            .field("password", &str::repeat("#", self.password.len()))
            .field("host", &self.host)
            .field("port", &self.port)
            .field("database_name", &self.database_name)
            .field("require_ssl", &self.require_ssl)
            .finish()
    }
}
