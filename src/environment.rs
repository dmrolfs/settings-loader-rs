use crate::internals::RenameRule;
use once_cell::sync::Lazy;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::SettingsError;

pub static LOCAL: Lazy<Environment> = Lazy::new(|| Environment("local".to_string()));
pub static PRODUCTION: Lazy<Environment> = Lazy::new(|| Environment("production".to_string()));

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Environment(String);

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Environment {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<Environment> for String {
    fn from(env: Environment) -> Self {
        env.0
    }
}

impl From<String> for Environment {
    fn from(rep: String) -> Self {
        rep.as_str().into()
    }
}

impl FromStr for Environment {
    type Err = SettingsError;

    fn from_str(rep: &str) -> Result<Self, Self::Err> {
        Ok(rep.into())
    }
}

impl From<&str> for Environment {
    fn from(rep: &str) -> Self {
        Self(RenameRule::KebabCase.apply(rep.trim()))
    }
}

#[cfg(test)]
mod tests {
    use claim::*;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_to_string() {
        let actual: String = LOCAL.to_string();
        assert_eq!(actual, "local".to_string());

        let actual: String = PRODUCTION.to_string();
        assert_eq!(actual, "production".to_string());
    }

    #[test]
    fn test_into_string() {
        let actual: String = LOCAL.clone().into();
        assert_eq!(actual, "local".to_string());

        let actual: String = PRODUCTION.clone().into();
        assert_eq!(actual, "production".to_string());
    }

    #[test]
    fn test_try_fromstr() {
        assert_eq!(assert_ok!(Environment::from_str("local")), LOCAL.clone());
        assert_eq!(
            assert_ok!(Environment::from_str("LOCAL")),
            assert_ok!(Environment::from_str("l-o-c-a-l"))
        );
        assert_eq!(assert_ok!(Environment::from_str("PrOdUcTiOn")), "pr-od-uc-ti-on".into());
        assert_eq!(assert_ok!(Environment::from_str("lOcAl")), "l-oc-al".into());
        assert_ok!(Environment::from_str("foobar"));
        assert_eq!(assert_ok!(Environment::from_str("  local")), LOCAL.clone());
        assert_eq!(assert_ok!(Environment::from_str("local ")), LOCAL.clone());
        assert_eq!(
            assert_ok!(Environment::from_str("Int-1AwsEuWest-1Dev")),
            Environment("int-1-aws-eu-west-1-dev".to_string()),
        );
        let actual: Environment = "PRODUCTION".into();
        assert_eq!(actual, Environment("p-r-o-d-u-c-t-i-o-n".to_string()));
        let staging: String = "StagingAwsUsWest2".to_string();
        let actual: Environment = staging.into();
        assert_eq!(actual, Environment("staging-aws-us-west2".to_string()));
    }
}
