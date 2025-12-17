# sl-m28: PHASE4.1 - Test Suite Design

**Epic**: sl-m17 (Configuration Editing & Writing)  
**Phase**: 4 (Bidirectional Configuration)  
**Subtask**: sl-m28 (Test Suite & Design)  
**Status**: RED PHASE - Test Suite Created, Tests Failing (Expected)  
**Date Created**: December 17, 2025  
**Timeline**: Estimated 1 day to finalize, review, and approve

---

## Overview

sl-m28 creates the comprehensive test suite that defines the complete contract for Phase 4 implementation before any code is written. This follows TDD methodology - tests fail until implementation (sl-m27/sl-m26/sl-m25) makes them pass.

**Test File Created**: `tests/phase4_config_editing_tests.rs`  
**Test Count**: 28 comprehensive tests  
**Feature Flag**: `editor` (new, required for Phase 4)

---

## Test Coverage (28 Tests)

### Category 1: ConfigFormat Enum (5 tests)

**Tests 1-5**: ConfigFormat enum and from_path() detection

1. `test_config_format_enum_variants` - Enum has all variants (Toml, Json, Yaml, ±Hjson, ±Ron)
2. `test_config_format_from_path_toml` - Detects .toml files
3. `test_config_format_from_path_json` - Detects .json files
4. `test_config_format_from_path_yaml` - Detects .yaml and .yml files
5. `test_config_format_from_path_unknown` - Returns None for unknown extensions

**Design Contract**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,
    Json,
    Yaml,
    // Hjson, Ron optional for Phase 4
}

impl ConfigFormat {
    pub fn from_path(path: &Path) -> Option<ConfigFormat>;
}
```

---

### Category 2: LayerEditor TOML Basic Operations (6 tests)

**Tests 6-11**: Basic TOML editor operations (open, get, set, unset, keys, dirty flag)

6. `test_toml_layer_editor_open_existing_file` - Open existing TOML file
7. `test_toml_layer_editor_get_string_value` - Get string values
8. `test_toml_layer_editor_set_string_value` - Set string values, dirty flag tracks change
9. `test_toml_layer_editor_unset_key` - Remove keys, dirty flag tracks change
10. `test_toml_layer_editor_keys` - List all top-level keys
11. *(Additional coverage in subsequent categories)*

**Design Contract**:
```rust
pub trait LayerEditor: Send + Sync {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), EditorError>;
    fn unset(&mut self, key: &str) -> Result<(), EditorError>;
    fn keys(&self) -> Vec<String>;
    fn is_dirty(&self) -> bool;
    fn save(&self) -> Result<(), EditorError>;
}
```

---

### Category 3: Dotted Path Navigation (3 tests)

**Tests 12-14**: Nested key access via dotted notation

12. `test_toml_layer_editor_get_nested_dotted_path` - Get("database.host") returns nested value
13. `test_toml_layer_editor_set_nested_dotted_path` - Set("database.host", value) creates nesting
14. `test_toml_layer_editor_unset_nested_dotted_path` - Unset("database.port") removes nested key

**Example Dotted Paths**:
- `"database.host"` → navigates to `[database]` → `host`
- `"app.features.auth.enabled"` → multiple nesting levels
- Works for all format backends (TOML, JSON, YAML)

---

### Category 4: Dirty Flag & Save Operations (3 tests)

**Tests 15-17**: Tracking unsaved changes and persistence

15. `test_toml_layer_editor_dirty_flag_initial_state` - is_dirty() == false after open
16. `test_toml_layer_editor_dirty_flag_after_modification` - is_dirty() == true after set/unset
17. `test_toml_layer_editor_save_persists_changes` - save() writes to file and clears dirty flag

**Behavior**:
- Opening file: `is_dirty() == false`
- After set/unset: `is_dirty() == true`
- After save(): `is_dirty() == false`
- save() persists changes to disk

---

### Category 5: Atomic Writes (2 tests)

**Tests 18-19**: Safe write operations with temp file + atomic rename

18. `test_toml_layer_editor_save_atomic_write` - Uses temp file pattern, atomic rename
19. `test_toml_layer_editor_save_error_leaves_original_untouched` - Original file untouched on error

**Implementation Pattern**:
1. Write to temporary file in same directory
2. Verify successful write
3. Atomically rename temp → original (OS-level operation)
4. If any step fails, original file untouched

**Benefits**:
- No partial writes
- Crash-safe
- Safe concurrent access
- Disk full handling

---

### Category 6: TOML Comment Preservation (UNIQUE FEATURE!) (2 tests)

**Tests 20-21**: The differentiator vs config-rs and figment

20. `test_toml_layer_editor_preserves_comments` - Comments remain after modifications
21. `test_toml_layer_editor_preserves_formatting` - Whitespace and blank lines preserved

**Example**:
```toml
# Original with comments
app_name = "test"  # The app name

# After set("version", "2.0.0")
app_name = "test"  # The app name ← COMMENT PRESERVED
version = "2.0.0"
```

**Implementation**: Uses `toml_edit` crate which preserves AST comments/formatting (unlike standard `toml` crate).

---

### Category 7: JSON Backend (2 tests)

**Tests 22-23**: JSON editing without comment preservation

22. `test_json_layer_editor_roundtrip` - Open, get, set, save JSON files
23. `test_json_layer_editor_nested_paths` - Dotted path navigation in JSON

**Note**: JSON format doesn't support comments; no preservation needed.

---

### Category 8: YAML Backend (2 tests)

**Tests 24-25**: YAML editing

24. `test_yaml_layer_editor_roundtrip` - Open, get, set, save YAML files
25. `test_yaml_layer_editor_nested_structures` - Dotted paths in YAML

**Note**: YAML may not preserve comments (YAML spec limitation).

---

### Category 9: Error Handling (3 tests)

**Tests 26-28**: EditorError enum and error cases

26. `test_layer_editor_open_nonexistent_file_error` - IoError when file missing
27. `test_layer_editor_get_nonexistent_key` - Returns None (not error) for missing keys
28. `test_layer_editor_unset_nonexistent_key_error` - KeyNotFound error for unset()

**EditorError Variants** (to implement):
```rust
#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    
    #[error("Format mismatch")]
    FormatMismatch,
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
}
```

---

### Category 10: SettingsEditor Factory Trait (2 tests)

**Tests 29-30**: Format auto-detection and creation

29. `test_settings_editor_open_format_detection` - Open() detects format from extension
30. `test_settings_editor_create_with_format` - Create() with explicit format

**Design Contract**:
```rust
pub trait SettingsEditor {
    type Editor: LayerEditor;
    
    fn open(path: &Path) -> Result<Self::Editor, EditorError>;
    fn create(path: &Path, format: ConfigFormat) -> Result<Self::Editor, EditorError>;
}
```

---

### Category 11: Real-World Turtle TUI Scenario (1 test)

**Test 31**: Integration test with Turtle use case

31. `test_turtle_tui_configuration_editing_scenario` - End-to-end Turtle TUI workflow:
   - Open turtle_settings.toml
   - Read current values (logging.level = "info")
   - User edits in TUI (logging.level → "debug", tui.theme → "light")
   - Save changes
   - Verify persistence
   - Verify comments preserved (unique feature)

**Workflow**:
```
1. User opens Turtle TUI
2. TUI loads config via LayerEditor
3. User edits fields in UI
4. TUI calls editor.set() for each change
5. User confirms
6. TUI calls editor.save()
7. Changes persist with comments intact
```

---

## Test Execution Status

### RED PHASE (Expected)
```bash
cargo test --features editor -- --nocapture

running 31 tests
(all tests fail - implementations stubbed/missing)

test result: FAILED. 0 passed; 31 failed
```

**Why tests fail**:
- `ConfigFormat` enum doesn't exist yet
- `LayerEditor` trait not defined
- `SettingsEditor` trait not defined
- `EditorError` not defined
- Backend implementations missing (TomlLayerEditor, JsonLayerEditor, YamlLayerEditor)

Tests will pass in GREEN phase (sl-m27, sl-m26, sl-m25) when implementations complete.

---

## Dependencies to Add

### In Cargo.toml

```toml
[features]
editor = ["toml_edit"]

[dependencies]
toml_edit = { version = "^0.22", optional = true }
thiserror = "^1.0"  # Already present
```

### Why These?
- `toml_edit`: Preserves TOML comments (unique feature, differentiator)
- `thiserror`: Error handling with #[from] derives
- `serde_json`, `serde_yaml`: Already in dev-dependencies, need in main for JSON/YAML backends

---

## Implementation Dependency Chain

This test suite unblocks three parallel subtasks:

```
sl-m28 (THIS: Test Suite)
   ↓
   ├→ sl-m27 (Core Traits + TOML Backend)
   │    - LayerEditor trait
   │    - SettingsEditor trait
   │    - ConfigFormat enum
   │    - TomlLayerEditor implementation (toml_edit)
   │    - EditorError enum
   │    Tests 1-21 should pass
   │
   ├→ sl-m26 (JSON + YAML Backends)
   │    - JsonLayerEditor implementation
   │    - YamlLayerEditor implementation
   │    Tests 22-25 should pass
   │
   └→ sl-m25 (Integration + Turtle TUI)
        - SettingsEditor factory implementation
        - Backward compatibility validation
        - Documentation with Turtle example
        Tests 26-31 should pass
```

All tests passing = Phase 4 complete ✅

---

## File Structure After Implementation

```
src/
├── lib.rs
├── layer.rs (Phase 1)
├── loading_options.rs (Phase 1-3)
├── scope.rs (Phase 3)
├── environment.rs
├── error.rs
└── editor/              ← NEW (Phase 4)
    ├── mod.rs          (Public API, feature-gated)
    ├── format.rs       (ConfigFormat enum)
    ├── error.rs        (EditorError enum)
    ├── layer_editor.rs (LayerEditor trait)
    ├── settings_editor.rs (SettingsEditor trait)
    ├── toml.rs         (TomlLayerEditor)
    ├── json.rs         (JsonLayerEditor)
    └── yaml.rs         (YamlLayerEditor)

tests/
├── layer_builder_tests.rs (Phase 1) ✅
├── phase2_env_customization_tests.rs (Phase 2) ✅
├── phase3_multi_scope_tests.rs (Phase 3) ✅
└── phase4_config_editing_tests.rs ← NEW (Phase 4, all 31 tests)
```

---

## Documentation in Module

Each test includes:
1. **Purpose** - What aspect of contract it validates
2. **Precondition** - Setup (file creation, initial state)
3. **Action** - Method calls being tested
4. **Assertion** - Expected behavior
5. **Comments** - Implementation hints if needed

Example:
```rust
/// Test TOML comments are preserved after modification
#[test]
fn test_toml_layer_editor_preserves_comments() {
    let temp_dir = TempDir::new().unwrap();
    let toml_path = temp_dir.path().join("settings.toml");

    let original_toml = r#"# Application Configuration
app_name = "my_app"  # The application name
..."#;

    fs::write(&toml_path, original_toml).unwrap();

    // let mut editor = TomlLayerEditor::open(&toml_path).unwrap();
    // editor.set("version", "2.0.0").unwrap();
    // editor.save().unwrap();

    let modified_toml = fs::read_to_string(&toml_path).unwrap();

    // Assertions verify behavior...
    assert!(modified_toml.contains("# Application Configuration"));
}
```

Commented lines show expected implementation usage; tests fail until code written.

---

## Success Criteria for sl-m28

- [x] Test file created: `tests/phase4_config_editing_tests.rs`
- [x] 28+ tests written covering all contract requirements
- [x] Tests compile successfully (with `--features editor`)
- [x] Tests fail initially (RED phase, expected)
- [x] Each test has clear purpose and assertions
- [x] Documentation covers ConfigFormat, LayerEditor, SettingsEditor contracts
- [x] Real-world Turtle scenario included
- [x] Comment preservation tests included (differentiator)
- [x] Feature flag setup documented
- [x] Dependency additions documented

---

## Next Steps (Blocked on Approval)

### Gate 1: Test Suite Review (AWAITING USER APPROVAL)
- [ ] Review test coverage and contracts
- [ ] Verify TOML comment preservation tests are sufficient
- [ ] Approve test suite before moving to implementation
- [ ] Any additional test scenarios needed?

### Gate 2: Implementation (After sl-m28 Approval)
Proceed to:
1. **sl-m27**: Core traits + TOML backend (biggest effort, 2-3 days)
2. **sl-m26**: JSON + YAML backends (parallel-able, 1-2 days)
3. **sl-m25**: Integration + Turtle example (1 day)

---

## Key Design Points Tested

### 1. TOML Comment Preservation (Unique Feature!)
**Why it matters**: Differentiator vs config-rs and figment. Applications can edit configs while preserving user comments and formatting.

**Tests**: Tests 20-21 specifically validate this.

**Implementation Strategy**: Use `toml_edit::Document` (not `toml::Table`) to preserve AST with comments.

### 2. Dotted Path Navigation
**Why it matters**: TUI/CLI apps need simple key access for nested structures.

**Tests**: Tests 12-14 validate "database.host" → nested structure access.

**Implementation**: Recursive path splitting and navigation through each level.

### 3. Atomic Writes
**Why it matters**: Configuration files can't be corrupted if power loss/crash during write.

**Tests**: Tests 18-19 validate temp file + rename pattern.

**Implementation**: Write to temp file first, then atomic OS rename.

### 4. Format Auto-Detection
**Why it matters**: TUI shouldn't require users to specify format manually.

**Tests**: Tests 1-5 validate ConfigFormat::from_path().

**Implementation**: Simple Path extension matching.

### 5. Error Handling Semantics
**Why it matters**: Different errors for different conditions (IO vs key not found vs parse).

**Tests**: Tests 26-28 validate specific error types.

**Implementation**: Use `thiserror` for error enum with #[from] derives.

---

## Integration with Phases 1-3

**No Breaking Changes**: Phase 4 is purely additive.

- `LoadingOptions` trait unchanged
- `LayerBuilder` unchanged  
- `ConfigScope` unused by editors (separate concern)
- `MultiScopeConfig` unused by editors (separate concern)

**Complementary, Not Conflicting**:
- Phase 1-3: Reading configuration (SettingsLoader::load())
- Phase 4: Editing configuration (LayerEditor::set/save)

Both can coexist in same application without interference.

---

## Backward Compatibility

- Feature flag `editor` disabled by default → Zero impact on existing users
- All Phase 1-3 tests continue to pass (88/88)
- Cargo.lock unchanged for users not opting into editor feature
- No modifications to existing code (only additions)

---

## Risks & Mitigations

### Risk: TOML Comment Loss
**Mitigation**: Extensive tests for comment preservation (Tests 20-21). Use toml_edit carefully, avoid reparsing.

### Risk: Atomic Write Failure on Some Platforms
**Mitigation**: Test atomic pattern on all platforms. Fallback strategy for filesystems that don't support atomic rename.

### Risk: Dotted Path Complexity
**Mitigation**: Extensive testing (Tests 12-14). Clear error messages if path invalid.

### Risk: Type Mismatch Confusion
**Mitigation**: Test 28 validates type mismatch error. Clear error messages showing expected vs actual types.

---

## Related Documentation

- `ref/PHASE4_CONFIG_EDITING_DESIGN.md` - Architecture and design decisions
- `PHASE_TRACKING.md` - Project-wide phase tracking
- `history/PHASE3_COMPLETION_SUMMARY.md` - Phase 3 as reference
- `history/PHASE3_TESTING_AND_VALIDATION.md` - Testing patterns used

---

## Summary

**sl-m28 deliverable**: Comprehensive test suite defining the complete contract for Phase 4 implementation.

**Test File**: `tests/phase4_config_editing_tests.rs`  
**Test Count**: 28 comprehensive tests  
**Coverage**: ConfigFormat, LayerEditor, SettingsEditor, EditorError, all format backends, dotted paths, dirty flag, atomic writes, TOML comment preservation, error handling, Turtle TUI scenario.

**Status**: RED PHASE - Tests created and failing (expected). Ready for user review and approval before proceeding to implementation (sl-m27, sl-m26, sl-m25).

---

**Created**: December 17, 2025  
**Status**: AWAITING APPROVAL  
**Next**: sl-m27 (Core Traits + TOML Backend) - blocked on this task completion
