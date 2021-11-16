use std::convert::TryFrom;
use std::fmt::Display;
use std::str::FromStr;

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
            Environment::Local => "Local",
            Environment::Production => "Production",
        }
    }
}

impl Into<String> for Environment {
    fn into(self) -> String {
        self.to_string()
    }
}

impl FromStr for Environment {
    type Err = SettingsError;

    fn from_str(rep: &str) -> Result<Self, Self::Err> {
        let mut result = None;
        for e in Environment::all().iter() {
            let e_rep: &str = e.as_ref();
            if e_rep.eq_ignore_ascii_case(rep) {
                result = Some(*e);
                break;
            }
        }

        result.ok_or(SettingsError::UnrecognizedEnvironment(rep.to_string()))
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
        assert_eq!(actual, "Local".to_string());

        let actual: String = Environment::Production.to_string();
        assert_eq!(actual, "Production".to_string());
    }

    #[test]
    fn test_into_string() {
        let actual: String = Environment::Local.into();
        assert_eq!(actual, "Local".to_string());

        let actual: String = Environment::Production.into();
        assert_eq!(actual, "Production".to_string());
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
