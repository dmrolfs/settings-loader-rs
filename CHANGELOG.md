# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0] - 2026-09-28

### Added
- **`EnvironmentLabel`**: a normalized, open deployment-environment label with a locality
  predicate (`is_local()`/`requires_production_strictness()`), additive alongside `Environment`.
  `Environment` is this crate's config-layer-selection key (`SettingsLoader::load_implicit` picks
  `./config/<environment>.*` files by it) and stays exactly that. `EnvironmentLabel` answers a
  different, equally common question -- "is this deployment the one class in which permissive,
  non-secret defaults are acceptable, or is it one of the others?" -- without forcing every real
  deployment topology into a closed two-value enum. Real fleets commonly run more than two named
  environments (`staging-us`, `staging-eu`, one per isolated tenant, ...); a closed
  `{Local, Production}`-shaped type would collapse all of those into a single generic label,
  destroying the operator's actual environment identity at the exact point it's resolved. Any
  declared string is a valid label -- normalization is limited to trimming and lowercasing, never
  a word-boundary case conversion, and there is no rejection path (`FromStr::Err = Infallible`).
  Mirrors the case-insensitive whole-token comparison convention .NET's `IHostEnvironment` uses.
  Deliberately does no I/O of its own -- reading the raw environment variable stays each
  consumer's own responsibility, through whatever seam that consumer already uses for testable
  environment-variable access.

## [1.1.0] - 2026-09-28

### Fixed
- **`Environment` case normalization**: `Environment::from(&str)` no longer applies a
  word-boundary case conversion (`RenameRule::KebabCase`) to its input. That conversion is
  designed for compound Rust identifiers (`VeryTasty` -> `very_tasty`), not for a free-form,
  human-typed environment label -- applied to an all-caps value such as `"PRODUCTION"`, it
  inserted a separator before every letter, producing `"p-r-o-d-u-c-t-i-o-n"`, which then failed
  to equal `environment::PRODUCTION`. `Environment` now normalizes with a plain
  `trim().to_ascii_lowercase()`: any casing of the same word (`local`, `LOCAL`, `Local`, `lOcAl`)
  normalizes to the same canonical value, and any separator style already present in the input
  is preserved verbatim rather than re-derived from capitalization. This matches the
  case-insensitive whole-token comparison convention used by, e.g., .NET's `IHostEnvironment`.
  **Behavior change**: a compound label typed in PascalCase (e.g. `"StagingAwsUsWest2"`) is now
  lowercased verbatim (`"stagingawsuswest2"`) rather than split into hyphenated words
  (`"staging-aws-us-west2"`); write compound labels with your own separators
  (`"staging-aws-us-west2"`) if that form is wanted.
- **Clippy**: simplified a `match` over `Result` in `TomlLayerEditor` that clippy flagged as
  better expressed with the `?` operator directly.

### Removed
- **Internal**: deleted the now-unused `internals::case`/`RenameRule` module. It existed solely
  to support the old `Environment` normalization above; nothing else in the crate used it.

## [1.0.0] - 2026-01-06

### Added
- **New Feature**: `LayerBuilder::with_path_in_dir(dir, basename)` method for automatic format discovery
  - Searches directory for configuration files with supported extensions
  - Supports yaml, yml, toml, json, ron, hjson, json5 formats
  - Provides clear error messages when files not found
- **Documentation**: Comprehensive README refinement for 1.0.0 release
  - Added file extension precedence documentation (YAML > TOML > JSON > JSON5 > HJSON > RON)
  - Listed all 7 available `LayerBuilder` source types with descriptions
  - Added File Discovery section explaining basename search pattern
  - Clarified that extension precedence applies independently per layer
  - Clarified TOML-only comment preservation in editor feature
- **Documentation**: Created `ref/FUTURE_ENHANCEMENTS.md` with 10 potential enhancements
- **Tests**: Added 7 comprehensive tests for `with_path_in_dir` feature
  - Format discovery (YAML, TOML, JSON)
  - Format precedence when multiple files exist
  - Error handling when no files found
  - Relative path support
  - Layer composition

### Changed
- **Breaking**: Converted `LayerResult` from tuple to `LayerResolution` struct for better code clarity
  - Struct has named fields: `metadata`, `config_source`, `provenance_source`
  - Improves code readability and maintainability
- **Documentation**: Renamed "Spark-Turtle" pattern to generic "Desktop/CLI Application" pattern
- **Documentation**: Generalized example environment variable prefixes (TURTLE → APP)
- **Documentation**: Updated all examples to demonstrate new `with_path_in_dir` feature

### Fixed
- Improved error messages for missing configuration files

## [0.15.0] - Previous Release

### Added
- Multi-scope configuration support
- Configuration editing with format preservation
- Metadata and introspection capabilities
- Validation framework
- Provenance tracking

---

## Migration Guide (0.15.0 → 1.0.0)

### LayerResult Type Change

If you were using the internal `LayerResult` type (unlikely for most users), it has changed from a tuple to a struct:

**Before (0.15.0)**:
```rust
let (metadata, config_source, provenance_source) = resolve_layer(...)?;
```

**After (1.0.0)**:
```rust
let resolution = resolve_layer(...)?;
let metadata = resolution.metadata;
let config_source = resolution.config_source;
let provenance_source = resolution.provenance_source;
```

### New Feature: with_path_in_dir

You can now discover configuration files by directory and basename:

```rust
let builder = LayerBuilder::new()
    .with_path_in_dir("config", "application");  // Finds config/application.{yaml,toml,json,...}
```

This is more convenient than specifying the full path with extension.

---

[Unreleased]: https://github.com/dmrolfs/settings-loader-rs/compare/v1.0.0-rc.1...HEAD
[1.0.0-rc.1]: https://github.com/dmrolfs/settings-loader-rs/compare/v0.15.0...v1.0.0-rc.1
[0.15.0]: https://github.com/dmrolfs/settings-loader-rs/releases/tag/v0.15.0
