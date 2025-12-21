# Test Quality Analysis Findings (sl-0qz)

Analysis of test coverage and mutation robustness for `settings-loader-rs` using `cargo-tarpaulin` and `cargo-mutants`.

## Summary
The baseline analysis reveals that while the core loading logic is well-tested, common utility structs, the new editor functionality, and exhaustive schema generation have significant gaps.

## Key Findings

### 1. Missing Robustness in Common Structs
- **File**: `src/common/database.rs`, `src/common/http.rs`
- **Issue**: `PartialEq` implementations have surviving mutants (e.g., `&&` replaced with `||`). This indicates that tests for equality are missing or do not check all fields.
- **Impact**: Potential for silent regressions in configuration comparison logic.

### 2. Editor State & Path Resolution
- **Files**: `src/editor/json.rs`, `src/editor/toml.rs`, `src/editor/yaml.rs`, `src/editor/config_editor.rs`
- **Issue**: `is_dirty()` tracking and `keys()` retrieval are under-tested. Many mutants related to these functions survived.
- **Issue**: Path resolution lacks negative tests (e.g., descending into a non-mapping value).

### 3. Metadata & Schema Generation
- **Files**: `src/metadata.rs`, `src/validation.rs`
- **Issue**: `SettingType` variants like `Enum`, `Duration`, and `Url` are not fully exercised in schema/example generation tests.
- **Issue**: `Constraint::validate_length` has missing coverage for `Array` values.

### 4. Path Resolution in MultiScopeConfig
- **File**: `src/loading_options.rs`
- **Issue**: survivors in directory resolution and implicit search paths.

## Metrics Baseline
- **Mutation survivors**: 139 missed mutants.
- **Coverage**: Core modules have high coverage, but utility and editor modules are lagging.
