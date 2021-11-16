use std::convert::TryFrom;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::SettingsError;

#[derive(Debug, Copy, Clone, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Local,
    Production,
}

static ENVIRONMENTS: [Environment; 2] = [Environment::Local, Environment::Production];

impl Environment {
    pub fn all() -> &'static [Environment] {
        &ENVIRONMENTS
    }
}

impl AsRef<str> for Environment {
    fn as_ref(&self) -> &str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = SettingsError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(SettingsError::Bootstrap {
                message: format!("{} environment unrecognized", other),
                setting: "environment identification".to_string(),
            }),
        }
    }
}
