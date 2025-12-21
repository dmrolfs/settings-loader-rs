# Implementation Plan - sl-0qz: Test Quality Improvements

This plan outlines the targeted improvements to address coverage and mutation gaps in `settings-loader-rs`.

## Goal
Improve crate quality by identifying and filling testing gaps that offer the most return on investment, pushing line coverage towards 90%+ and reducing mutation survivors significantly.

## Proposed Changes

### [Common Library]

#### [MODIFY] [database.rs](file:///Users/rolfs/Documents/dev/dmrolfs/settings-loader-rs/src/common/database.rs)
- Add comprehensive tests for `PartialEq` implementation, ensuring all fields (including `password` comparison via `ExposeSecret`) are correctly handled.
- Add tests for `pg_pool_options` with various `Option` combinations (none, some, all).

#### [MODIFY] [http.rs](file:///Users/rolfs/Documents/dev/dmrolfs/settings-loader-rs/src/common/http.rs)
- Add tests for `url` and `url_host` error cases.

### [Configuration Editor]

#### [MODIFY] [json.rs](file:///Users/rolfs/Documents/dev/dmrolfs/settings-loader-rs/src/editor/json.rs), [toml.rs](file:///Users/rolfs/Documents/dev/dmrolfs/settings-loader-rs/src/editor/toml.rs), [yaml.rs](file:///Users/rolfs/Documents/dev/dmrolfs/settings-loader-rs/src/editor/yaml.rs)
- Add tests for `is_dirty()` tracking over multiple operations.
- Add tests for `keys()` to verify top-level key retrieval.
- Add negative tests for path resolution (e.g., trying to descend into a non-mapping value).

#### [MODIFY] [config_editor.rs](file:///Users/rolfs/Documents/dev/dmrolfs/settings-loader-rs/src/editor/config_editor.rs)
- Add integration tests for multi-layer `unset` operations.
- Verify `is_dirty` and `dirty_files` aggregation across layers.

### [Metadata & Validation]

#### [MODIFY] [metadata.rs](file:///Users/rolfs/Documents/dev/dmrolfs/settings-loader-rs/src/metadata.rs)
- Expand schema/example generation tests to cover all `SettingType` variants.

#### [MODIFY] [validation.rs](file:///Users/rolfs/Documents/dev/dmrolfs/settings-loader-rs/src/validation.rs)
- Add missing coverage for `Constraint::validate_length` (specifically Array arm).

### [Loading & Paths]

#### [MODIFY] [loading_options.rs](file:///Users/rolfs/Documents/dev/dmrolfs/settings-loader-rs/src/loading_options.rs)
- Strengthen tests for `MultiScopeConfig` directory resolution.

## Verification Plan

### Automated Tests
- Run `cargo test` to verify all tests pass.
- Run `cargo tarpaulin --out html` to verify improved coverage.
- Run `cargo mutants` to confirm that the identified mutants are now caught.
