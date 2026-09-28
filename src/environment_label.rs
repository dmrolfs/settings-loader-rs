//! `EnvironmentLabel`: a normalized, open deployment-environment label with a
//! locality predicate.
//!
//! # Why this exists alongside `Environment`
//!
//! [`crate::environment::Environment`] is this crate's config-layer-selection key:
//! `SettingsLoader::load_implicit` uses it to pick which `./config/<environment>.*`
//! files to merge, and it is deliberately silent about anything beyond that -- any
//! string is a valid layer name.
//!
//! `EnvironmentLabel` answers a different, equally common question: "is this
//! deployment the one class of environment in which permissive, non-secret
//! defaults (a deterministic dev key, verbose logging, relaxed validation, ...) are
//! acceptable, or is it -- whatever it's actually called -- one of the others?"
//! That is fundamentally a **binary** predicate (`is_local`), not a small fixed
//! enumeration: real deployments commonly run more than two named environments
//! (`staging-us`, `staging-eu`, one per isolated tenant, ...), and a type that
//! forces every one of those into a closed `{Local, Production}`-shaped enum
//! destroys the operator's actual environment identity at the exact point it is
//! resolved -- useful for nothing else downstream (per-tenant metrics labeling,
//! alert routing, staged rollouts) ever recovers it.
//!
//! `EnvironmentLabel` therefore stays open (any declared string is a valid label,
//! preserved verbatim after normalization) and offers the one binary decision most
//! callers actually need as a method, [`EnvironmentLabel::is_local`], rather than as
//! a variant. This mirrors the case-insensitive whole-token comparison convention
//! .NET's `IHostEnvironment` uses (`IsDevelopment()`/`IsProduction()` compare with
//! `OrdinalIgnoreCase`, never a closed enum of every environment name).
//!
//! # Deliberately no I/O
//!
//! This type does not read `APP_ENVIRONMENT` (or any other variable) itself.
//! Reading the raw value is each consumer's own responsibility, through whatever
//! seam that consumer already uses for testable environment-variable access (this
//! crate's own [`crate::loading_options::LoadingOptions::environment`] reads the
//! process environment directly; a consumer that needs unit-testable resolution
//! without touching real process-global state should read the raw string through
//! its own injectable seam and construct an `EnvironmentLabel` from the result).
//! Baking a specific I/O strategy into this type would force every consumer onto
//! that strategy.
//!
//! # Normalization, not validation
//!
//! Trimmed and lowercased, and nothing more -- deliberately **not** a word-boundary
//! case conversion (no PascalCase/camelCase splitting into `kebab-case`, the defect
//! `Environment`'s own normalization had before it was fixed). A label is a single
//! atomic token a human types (`production`, `tenant-a-prod`), not a compound Rust
//! identifier whose word boundaries could be inferred from capitalization.

use std::convert::Infallible;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The canonical spelling [`EnvironmentLabel::is_local`] matches, case-insensitively.
const LOCAL: &str = "local";

/// A normalized, open deployment-environment label.
///
/// "Open" means any declared value is valid -- there is no rejection path and no
/// `FromStr::Err` case for an "unrecognized" value, because there is no closed set
/// to fail to match against. See the module doc for why this is deliberate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EnvironmentLabel(String);

impl EnvironmentLabel {
    /// True if this label is the canonical "local" spelling (case-insensitive;
    /// already normalized at construction, so this is a plain string comparison).
    ///
    /// The only class in which permissive, non-secret defaults are appropriate.
    /// Every other label -- however it is spelled or named -- is not local, by
    /// design: an unrecognized or misspelled label must never be silently treated
    /// as the permissive case.
    pub fn is_local(&self) -> bool {
        self.0 == LOCAL
    }

    /// The inverse of [`Self::is_local`], named for the call site that actually
    /// gates behavior: "does this deployment require production-grade secret
    /// handling, validation strictness, etc.?" Reads better than `!is_local()` at
    /// a gate.
    pub fn requires_production_strictness(&self) -> bool {
        !self.is_local()
    }
}

impl fmt::Display for EnvironmentLabel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for EnvironmentLabel {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<EnvironmentLabel> for String {
    fn from(label: EnvironmentLabel) -> Self {
        label.0
    }
}

impl From<&str> for EnvironmentLabel {
    fn from(rep: &str) -> Self {
        Self(rep.trim().to_ascii_lowercase())
    }
}

impl From<String> for EnvironmentLabel {
    fn from(rep: String) -> Self {
        rep.as_str().into()
    }
}

impl FromStr for EnvironmentLabel {
    /// Parsing an `EnvironmentLabel` cannot fail -- there is no closed set to
    /// reject against. `Infallible`, not [`crate::SettingsError`], states that
    /// precisely: a caller can `.unwrap()` a `.parse()` result without it ever
    /// being a false promise.
    type Err = Infallible;

    fn from_str(rep: &str) -> Result<Self, Self::Err> {
        Ok(rep.into())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn is_local_matches_any_casing() {
        for input in ["local", "LOCAL", "Local", "lOcAl", "  local  "] {
            assert!(EnvironmentLabel::from(input).is_local(), "{input:?} should be local");
        }
    }

    #[test]
    fn any_other_label_requires_production_strictness() {
        for input in [
            "production",
            "PRODUCTION",
            "staging",
            "staging-us",
            "staging-eu",
            "tenant-a-prod",
            "tenant-b-prod",
            "qa",
            "",
        ] {
            let label = EnvironmentLabel::from(input);
            assert!(!label.is_local(), "{input:?} should not be local");
            assert!(label.requires_production_strictness());
        }
    }

    #[test]
    fn preserves_the_declared_identity_verbatim_after_normalizing_case() {
        // The whole point: a named environment/tenant label is not collapsed into
        // a generic "production" bucket -- its identity survives normalization.
        assert_eq!(EnvironmentLabel::from("Tenant-A-Prod").to_string(), "tenant-a-prod");
        assert_eq!(EnvironmentLabel::from("Staging-EU").to_string(), "staging-eu");
    }

    #[test]
    fn display_and_as_ref_agree() {
        let label = EnvironmentLabel::from("Production");
        assert_eq!(label.to_string(), label.as_ref());
    }

    #[test]
    fn from_string_and_from_str_agree_with_parse() {
        let a = EnvironmentLabel::from("Production");
        let b: EnvironmentLabel = "Production".to_string().into();
        let c: EnvironmentLabel = "Production".parse().unwrap();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }
}
