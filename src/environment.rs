use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::SettingsError;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Local,
    Production,
}

static ENVIRONMENTS: [Environment; 2] = [Environment::Local, Environment::Production];

impl Environment {
    pub fn all() -> &'static [Self] {
        &ENVIRONMENTS
    }
}

const LOCAL_REP: &str = "local";
const PRODUCTION_REP: &str = "production";

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local => write!(f, "{}", LOCAL_REP),
            Self::Production => write!(f, "{}", PRODUCTION_REP),
        }
    }
}

impl AsRef<str> for Environment {
    fn as_ref(&self) -> &str {
        match self {
            Environment::Local => LOCAL_REP,
            Environment::Production => PRODUCTION_REP,
        }
    }
}

impl From<Environment> for String {
    fn from(env: Environment) -> Self {
        env.to_string()
    }
}

impl FromStr for Environment {
    type Err = SettingsError;

    fn from_str(rep: &str) -> Result<Self, Self::Err> {
        let mut result = None;
        for e in Self::all().iter() {
            let e_rep: &str = e.as_ref();
            if e_rep.eq_ignore_ascii_case(rep) {
                result = Some(*e);
                break;
            }
        }

        result.ok_or_else(|| SettingsError::UnrecognizedEnvironment(rep.to_string()))
    }
}

impl TryFrom<&str> for Environment {
    type Error = SettingsError;

    fn try_from(rep: &str) -> Result<Self, Self::Error> {
        Self::from_str(rep)
    }
}

impl TryFrom<String> for Environment {
    type Error = SettingsError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

#[cfg(test)]
mod tests {
    use claim::*;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_to_string() {
        let actual: String = Environment::Local.to_string();
        assert_eq!(actual, "local".to_string());

        let actual: String = Environment::Production.to_string();
        assert_eq!(actual, "production".to_string());
    }

    #[test]
    fn test_into_string() {
        let actual: String = Environment::Local.into();
        assert_eq!(actual, "local".to_string());

        let actual: String = Environment::Production.into();
        assert_eq!(actual, "production".to_string());
    }

    #[test]
    fn test_try_fromstr() {
        assert_eq!(Environment::Local, assert_ok!(Environment::from_str("local")));
        assert_eq!(Environment::Local, assert_ok!(Environment::from_str("LOCAL")));
        assert_eq!(Environment::Production, assert_ok!(Environment::from_str("PrOdUcTiOn")));
        assert_eq!(Environment::Local, assert_ok!(Environment::from_str("lOcAl")));
        assert_err!(Environment::from_str("foobar"));
        assert_err!(Environment::from_str("  local"));
        assert_err!(Environment::from_str("local "));
    }
}
