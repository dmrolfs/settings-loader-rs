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
    /// Normalizes to a case-insensitive token: trimmed, lowercased, and otherwise
    /// verbatim.
    ///
    /// Deliberately **not** a word-boundary case conversion (no PascalCase/camelCase
    /// splitting into `kebab-case`). An environment name is a single atomic label an
    /// operator types directly (`production`, `staging-us-west-2`), not a compound
    /// Rust identifier whose word boundaries this type could infer from
    /// capitalization -- there is no such information to infer, so an algorithm
    /// that tries to (as an earlier version of this type did, via
    /// `RenameRule::KebabCase`) can only guess wrong, e.g. turning an all-caps
    /// `"PRODUCTION"` into `"p-r-o-d-u-c-t-i-o-n"`, which then fails to equal
    /// [`PRODUCTION`] by value. This matches the case-insensitive-whole-token
    /// comparison convention used by, e.g., .NET's `IHostEnvironment` (`IsProduction`
    /// etc. compare with `OrdinalIgnoreCase`, never a word-split transform) rather
    /// than inventing a bespoke one. Any separator style the operator already typed
    /// (`-`, `_`) is preserved verbatim; only casing is normalized.
    fn from(rep: &str) -> Self {
        Self(rep.trim().to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::{assert_let, assert_matches};
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
        assert_let!(Ok(local_0) = Environment::from_str("local"));
        assert_eq!(local_0, LOCAL.clone());
        // Case-insensitive whole-token normalization: any casing of the same word
        // equals the canonical lowercase form -- no word-splitting is applied.
        assert_let!(Ok(local_1) = Environment::from_str("LOCAL"));
        assert_eq!(local_1, LOCAL.clone());
        assert_let!(Ok(local_2) = Environment::from_str("Local"));
        assert_eq!(local_2, LOCAL.clone());
        assert_let!(Ok(local_3) = Environment::from_str("lOcAl"));
        assert_eq!(local_3, LOCAL.clone());
        assert_let!(Ok(local_4) = Environment::from_str("  local"));
        assert_eq!(local_4, LOCAL.clone());
        assert_let!(Ok(local_5) = Environment::from_str("local "));
        assert_eq!(local_5, LOCAL.clone());

        // Any separator style the operator already typed is preserved verbatim --
        // only casing is normalized, never re-derived from capitalization.
        assert_let!(Ok(env_0) = Environment::from_str("Int-1AwsEuWest-1Dev"));
        assert_eq!(env_0, Environment("int-1awseuwest-1dev".to_string()));

        assert_matches!(Environment::from_str("foobar"), Ok(_));

        // Case-insensitive whole-token normalization, the production-side twin of
        // the LOCAL case above: any casing of "production" equals `*PRODUCTION`.
        let actual: Environment = "PRODUCTION".into();
        assert_eq!(actual, PRODUCTION.clone());
        assert_eq!(actual, Environment("production".to_string()));
        let actual: Environment = "PrOdUcTiOn".into();
        assert_eq!(actual, PRODUCTION.clone());

        // A compound label is never re-split into words -- it is lowercased
        // verbatim, preserving whatever separators (or lack thereof) the operator
        // wrote. Contrast with the pre-lowercase-only behavior, which guessed word
        // boundaries from capitalization and produced "staging-aws-us-west2".
        let staging: String = "StagingAwsUsWest2".to_string();
        let actual: Environment = staging.into();
        assert_eq!(actual, Environment("stagingawsuswest2".to_string()));
    }
}
