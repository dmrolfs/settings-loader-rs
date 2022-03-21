//! substantially adapted from serde_derive::internals::case

// See https://users.rust-lang.org/t/psa-dealing-with-warning-unused-import-std-ascii-asciiext-in-today-s-nightly/13726
#[allow(deprecated, unused_imports)]
use std::ascii::AsciiExt;

use self::RenameRule::*;
use crate::SettingsError;

/// The different possible ways to change case of strings.
#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq)]
pub enum RenameRule {
    /// Don't apply a default rename rule.
    None,
    /// Rename direct children to "lowercase" style.
    LowerCase,
    /// Rename direct children to "UPPERCASE" style.
    UpperCase,
    /// Rename direct children to "PascalCase" style, as typically used for
    /// enum variants.
    PascalCase,
    /// Rename direct children to "camelCase" style.
    CamelCase,
    /// Rename direct children to "snake_case" style, as commonly used for
    /// fields.
    SnakeCase,
    /// Rename direct children to "SCREAMING_SNAKE_CASE" style, as commonly
    /// used for constants.
    ScreamingSnakeCase,
    /// Rename direct children to "kebab-case" style.
    KebabCase,
    /// Rename direct children to "SCREAMING-KEBAB-CASE" style.
    ScreamingKebabCase,
}

static RENAME_RULES: &[(&str, RenameRule)] = &[
    ("lowercase", LowerCase),
    ("UPPERCASE", UpperCase),
    ("PascalCase", PascalCase),
    ("camelCase", CamelCase),
    ("snake_case", SnakeCase),
    ("SCREAMING_SNAKE_CASE", ScreamingSnakeCase),
    ("kebab-case", KebabCase),
    ("SCREAMING-KEBAB-CASE", ScreamingKebabCase),
];

impl RenameRule {
    #[allow(dead_code)]
    pub fn from_str(rename_all_str: &str) -> Result<Self, SettingsError> {
        for (name, rule) in RENAME_RULES {
            if rename_all_str == *name {
                return Ok(*rule);
            }
        }

        Err(SettingsError::UnrecognizedEnvironment(rename_all_str.to_string()))
    }

    pub fn apply(&self, rep: &str) -> String {
        match *self {
            None => rep.to_owned(),
            PascalCase => {
                let mut pascal = String::new();
                let mut capitalize = true;
                for ch in rep.chars() {
                    if ch == '_' {
                        capitalize = true;
                    } else if capitalize {
                        pascal.push(ch.to_ascii_uppercase());
                        capitalize = false;
                    } else {
                        pascal.push(ch);
                    }
                }
                pascal
            },
            LowerCase => rep.to_ascii_lowercase(),
            UpperCase => rep.to_ascii_uppercase(),
            CamelCase => rep[..1].to_ascii_lowercase() + &rep[1..],
            SnakeCase => {
                let mut snake = String::new();
                for (i, ch) in rep.char_indices() {
                    if i > 0 && ch.is_uppercase() {
                        snake.push('_');
                    }
                    snake.push(ch.to_ascii_lowercase());
                }
                snake
            },
            ScreamingSnakeCase => SnakeCase.apply(rep).to_ascii_uppercase(),
            KebabCase => SnakeCase.apply(rep).replace('_', "-"),
            ScreamingKebabCase => ScreamingSnakeCase.apply(rep).replace('_', "-"),
        }
    }
}

#[test]
fn rename_strings() {
    for &(original, lower, upper, camel, snake, screaming, kebab, screaming_kebab) in &[
        (
            "Outcome", "outcome", "OUTCOME", "outcome", "outcome", "OUTCOME", "outcome", "OUTCOME",
        ),
        (
            "VeryTasty",
            "verytasty",
            "VERYTASTY",
            "veryTasty",
            "very_tasty",
            "VERY_TASTY",
            "very-tasty",
            "VERY-TASTY",
        ),
        ("A", "a", "A", "a", "a", "A", "a", "A"),
        ("Z42", "z42", "Z42", "z42", "z42", "Z42", "z42", "Z42"),
    ] {
        assert_eq!(None.apply(original), original);
        assert_eq!(LowerCase.apply(original), lower);
        assert_eq!(UpperCase.apply(original), upper);
        assert_eq!(PascalCase.apply(original), original);
        assert_eq!(CamelCase.apply(original), camel);
        assert_eq!(SnakeCase.apply(original), snake);
        assert_eq!(ScreamingSnakeCase.apply(original), screaming);
        assert_eq!(KebabCase.apply(original), kebab);
        assert_eq!(ScreamingKebabCase.apply(original), screaming_kebab);
    }
}
