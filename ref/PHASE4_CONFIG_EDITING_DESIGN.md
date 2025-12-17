# Phase 4: Configuration Editing & Writing - Design Specification

**Epic**: sl-m17  
**Phase**: 4  
**Status**: Design Ready - Awaiting Subtask Breakdown  
**Date Created**: December 17, 2025  
**Target**: v0.22.0 (all phases combined)

---

## Overview

Phase 4 transforms settings-loader from read-only to bidirectional, enabling applications to modify and save configuration programmatically while preserving format semantics and comments (TOML).

**Key Deliverable**: LayerEditor trait with format-specific backends and TOML comment preservation.

**Unique Feature**: TOML comment preservation via `toml_edit` crate - differentiator vs config-rs and figment.

---

## Problem Statement

### Current Limitations (Phase 3 End State)
- ✅ Can read configuration from multiple scopes
- ✅ Supports all formats (YAML, JSON, TOML, HJSON, RON)
- ❌ Cannot write configuration
- ❌ Cannot modify values programmatically
- ❌ Cannot preserve TOML comments
- ❌ Cannot integrate with TUI/CLI configuration editors

### Real-World Need: Turtle TUI
Spark Turtle needs configuration editor for TUI:
1. Display current settings
2. Allow user to edit values
3. Save changes atomically
4. Preserve comments and formatting
5. Support multiple configuration scopes

---

## High-Level Design

### New Traits

#### 1. LayerEditor Trait

```rust
/// Edit configuration layer with type safety and format awareness
pub trait LayerEditor: Send + Sync {
    /// Get a setting value by key (dotted path)
    /// Example: "database.host"
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;

    /// Set a setting value by key
    /// Returns error if format-specific validation fails
    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), EditorError>;

    /// Remove a setting key
    /// Returns error if key doesn't exist
    fn unset(&mut self, key: &str) -> Result<(), EditorError>;

    /// Get all available keys in this layer
    fn keys(&self) -> Vec<String>;

    /// Check if layer has unsaved changes
    fn is_dirty(&self) -> bool;

    /// Save changes back to file
    /// Atomic: writes to temp file, then renames (all-or-nothing)
    fn save(&self) -> Result<(), EditorError>;
}
```

#### 2. SettingsEditor Trait

```rust
/// Create and manage layer editors with format auto-detection
pub trait SettingsEditor {
    type Editor: LayerEditor;

    /// Open layer editor for given path
    /// Auto-detects format from file extension
    fn open(path: &Path) -> Result<Self::Editor, EditorError>;

    /// Create new configuration layer
    /// Requires explicit format specification
    fn create(path: &Path, format: ConfigFormat) -> Result<Self::Editor, EditorError>;
}
```

#### 3. ConfigFormat Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    Toml,   // Via toml_edit (preserves comments)
    Json,   // Via serde_json
    Yaml,   // Via serde_yaml
    Hjson,  // Via deser_hjson
    Ron,    // Via ron
}

impl ConfigFormat {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Option<ConfigFormat>;
}
```

### Format-Specific Backends

#### TOML Backend (toml_edit)
```rust
struct TomlLayerEditor {
    path: PathBuf,
    document: toml_edit::Document,  // Preserves comments
    dirty: bool,
}

impl LayerEditor for TomlLayerEditor {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        // Navigate dotted path: "a.b.c" → document["a"]["b"]["c"]
        // Deserialize to T
    }

    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), EditorError> {
        // Serialize T to TOML value
        // Navigate dotted path, set value
        // Mark dirty
    }

    fn save(&self) -> Result<(), EditorError> {
        // Write to temp file
        // Rename temp → original (atomic)
        // Mark not dirty
    }
}
```

**Key Advantage**: `toml_edit::Document` preserves:
- Comments
- Whitespace
- Formatting
- Key ordering

#### JSON Backend (serde_json)
```rust
struct JsonLayerEditor {
    path: PathBuf,
    document: serde_json::Value,
    dirty: bool,
}

impl LayerEditor for JsonLayerEditor {
    // Similar pattern but JSON doesn't preserve comments
    // Comments automatically stripped on read/write
}
```

#### YAML Backend (serde_yaml)
```rust
struct YamlLayerEditor {
    path: PathBuf,
    document: serde_yaml::Value,
    dirty: bool,
}

impl LayerEditor for YamlLayerEditor {
    // Similar pattern
    // Comments may not be preserved (YAML limitation)
}
```

### Dotted Path Navigation

All editors support dotted key notation:
```rust
editor.get::<String>("database.host")?;
editor.set("database.port", 5432)?;
editor.unset("database.password")?;
```

Navigates nested structures automatically.

### Atomic Writes

All backends implement atomic writes:
1. Write to temporary file in same directory
2. Verify successful write
3. Rename temp → original (OS atomic operation)
4. If any step fails, original file untouched

Benefits:
- No partial writes
- Safe concurrent access
- Crash-safe

### Error Handling

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

## Implementation Plan (4 Subtasks)

### PHASE4.1: Test Suite & Design (sl-m28)
**Purpose**: Establish test contract before implementation

**Tests to Create**:
1. Basic layer editor operations (get, set, unset)
2. Dotted path navigation
3. Dirty flag tracking
4. Save operations
5. Format detection
6. Atomic write verification
7. Error handling
8. TOML comment preservation
9. JSON roundtrip
10. YAML roundtrip
11. Real-world Turtle scenario

**Expected**: 25+ tests guiding implementation

**Deliverable**: `tests/phase4_config_editing_tests.rs`

---

### PHASE4.2: Core Traits & TOML Backend (sl-m27)
**Purpose**: Implement LayerEditor trait and TOML editor with comment preservation

**What to Build**:
1. EditorError enum
2. ConfigFormat enum with from_path()
3. LayerEditor trait definition
4. SettingsEditor trait definition
5. TomlLayerEditor struct
6. Dotted path navigation for TOML
7. Comment preservation in toml_edit Document
8. Atomic write implementation

**Deliverables**:
- `src/editor.rs` (new module)
- `src/editor/toml.rs` (TOML backend)
- Tests: PHASE4.1 tests 1-3, 8

**Dependencies**:
- `toml_edit` crate (v0.22+)
- `thiserror` for error enum

---

### PHASE4.3: JSON & YAML Backends (sl-m26)
**Purpose**: Implement format-agnostic editors for JSON and YAML

**What to Build**:
1. JsonLayerEditor struct
2. YamlLayerEditor struct
3. Dotted path navigation for JSON/YAML
4. Format-specific error handling
5. Atomic write for each format
6. Format detection from file extension

**Deliverables**:
- `src/editor/json.rs` (JSON backend)
- `src/editor/yaml.rs` (YAML backend)
- Tests: PHASE4.1 tests 4-7, 9-10

**Dependencies**:
- Uses existing serde_json, serde_yaml

---

### PHASE4.4: Integration & Real-World Scenarios (sl-m25)
**Purpose**: Integrate editors with SettingsLoader and validate Turtle use case

**What to Build**:
1. SettingsEditor trait implementations
2. Format auto-detection system
3. Integration with existing LoadingOptions
4. Example: Turtle configuration editor
5. Integration tests with all formats
6. Performance validation

**Deliverables**:
- SettingsEditor trait implementations
- Format detection system
- Turtle usage example
- Integration tests
- Performance benchmarks

**Tests**: PHASE4.1 tests 11-25 (real-world scenarios)

---

## Design Decisions

### 1. Separate Trait from SettingsLoader

**Decision**: LayerEditor as new trait, not part of SettingsLoader

**Rationale**:
- SettingsLoader is read-only, focuses on composition
- LayerEditor is write-focused, stateful
- Separate concerns
- Apps can use SettingsLoader without importing editor
- Backward compatible (SettingsLoader unchanged)

### 2. toml_edit for TOML Comment Preservation

**Decision**: Use `toml_edit` crate instead of `toml`

**Rationale**:
- `toml` crate parses to serde Value, loses comments
- `toml_edit` preserves document structure and comments
- Differentiator vs config-rs and figment
- Required for Turtle use case (preserve user edits)

**Trade-off**:
- toml_edit API slightly different from serde
- Dotted path navigation must handle both approaches
- Worth it for comment preservation

### 3. Dotted Path Navigation

**Decision**: Support "a.b.c" notation for nested keys

**Rationale**:
- Natural for flat key notation (environment variables, CLI)
- Reduces boilerplate for common case
- Handles arbitrary nesting depth
- Consistent with config crate's key system

**Implementation**:
- Split on ".", navigate recursively
- Create intermediate objects if missing (for set)
- Error if path partially invalid (for get)

### 4. Atomic Writes

**Decision**: Temp file + rename for all formats

**Rationale**:
- Prevents partial writes
- OS-level atomicity (rename syscall)
- Safe for concurrent readers
- Crash-safe (process dies during write → original untouched)
- Standard pattern (used by git, databases, etc.)

### 5. Format Auto-Detection by Extension

**Decision**: Auto-detect from file extension, allow override

**Rationale**:
- Most common case: extension matches format
- ConfigFormat::from_path() for auto-detection
- SettingsEditor::create() requires explicit format
- Clear intent (don't guess on write)

---

## Feature Flags

### Cargo.toml

```toml
[features]
# Existing
multi-scope = ["directories"]

# New Phase 4
editor = ["toml_edit"]

# All features
full = ["multi-scope", "editor", ...]
```

**Rationale**:
- Optional dependency (toml_edit not required for read-only)
- Backward compatible (editor feature disabled by default)
- Users opt-in to writing capability

---

## Dependency Changes

### Add to [dependencies]
```toml
toml_edit = { version = "^0.22", optional = true }
thiserror = "^1.0"  # Already present
```

### Feature Gates in Code
```rust
#[cfg(feature = "editor")]
pub mod editor;

#[cfg(feature = "editor")]
pub use editor::{LayerEditor, SettingsEditor, ConfigFormat, EditorError};
```

---

## Integration with Existing Code

### No Changes to Phase 1-3

- LoadingOptions trait unchanged
- SettingsLoader trait unchanged
- ConfigScope enum unused by editors (separate concern)
- MultiScopeConfig trait unused by editors

### Complementary to Reading

Reading Phase (Phase 3):
```rust
let settings = MySettings::load(&options)?;  // Deserialize from layers
```

Writing Phase (Phase 4):
```rust
let mut editor = TomlLayerEditor::open(Path::new("settings.toml"))?;
editor.set("debug", true)?;
editor.save()?;  // Serialize back with comments preserved
```

---

## Real-World Use Case: Turtle TUI

### Current Flow (Phase 3)
```rust
1. Load settings from multiple scopes
2. Display in TUI
3. User can view only
4. No ability to save changes
```

### New Flow (Phase 4)
```rust
1. Load settings from multiple scopes (existing)
2. Display in TUI (existing)
3. User edits settings in UI
4. TUI calls editor.set("key", value)
5. User confirms
6. TUI calls editor.save()
7. Comments preserved, atomic write completes
```

### Example Code
```rust
use settings_loader::editor::{SettingsEditor, LayerEditor};

// Open config file
let mut editor = TomlLayerEditor::open(Path::new("~/.config/turtle/settings.toml"))?;

// User changed debug flag in TUI
editor.set("debug", true)?;
editor.set("log_level", "debug")?;

// Save changes
if editor.is_dirty() {
    editor.save()?;
}
```

---

## Success Criteria

### Code Quality
- ✅ All PHASE4.1 tests (25+) passing
- ✅ All PHASE4.2-4.4 tests passing (total TBD)
- ✅ All Phase 1-3 tests still passing (backward compat)
- ✅ 0 clippy warnings (Phase 4 code only)
- ✅ Code formatted

### Functionality
- ✅ LayerEditor trait works for TOML, JSON, YAML
- ✅ Dotted path navigation for all formats
- ✅ Atomic writes prevent corruption
- ✅ TOML comments preserved
- ✅ Format auto-detection working

### Integration
- ✅ Works with Phase 1-3 features
- ✅ Optional feature flag (editor = ["toml_edit"])
- ✅ Turtle TUI can use it
- ✅ No breaking changes

---

## Architecture Diagram

```
SettingsLoader (Phase 1-3: Read-Only)
    ├── LoadingOptions trait
    ├── LayerBuilder
    ├── ConfigScope enum
    └── MultiScopeConfig trait

LayerEditor (Phase 4: Read-Write)
    ├── LayerEditor trait (base)
    │   ├── TomlLayerEditor (with comment preservation)
    │   ├── JsonLayerEditor
    │   └── YamlLayerEditor
    ├── SettingsEditor trait (factory)
    ├── ConfigFormat enum
    └── EditorError enum
```

---

## Phase Dependencies

- **Depends On**: Phase 1-3 complete ✅
- **Blocks**: Phase 5 (Metadata) - Can add metadata to settings module
- **Blocks**: Phase 6 (Provenance) - Provenance can track edits
- **Blocks**: Phase 7 (Schema) - Schema export needs to be readable/writable

---

## Known Limitations & Future Work

### Phase 4 Scope
- Single-file editing only (not cross-file transactions)
- No merge/conflict resolution for concurrent edits
- No undo/redo (applications can implement if needed)
- HJSON and RON backends not in Phase 4 (can add in Phase 5+)

### Future Enhancements
- **Phase 5+**: Cross-file transaction support
- **Phase 5+**: Conflict resolution
- **Phase 5+**: Undo/redo infrastructure
- **Phase 5+**: HJSON and RON editing backends

---

## Testing Strategy

### Unit Tests (PHASE4.1-4.4)
- Individual editor operations (get, set, unset, save)
- Dotted path navigation
- Format-specific behavior
- Error handling
- Atomic write verification

### Integration Tests
- Multi-format round-trips
- Format auto-detection
- Real file I/O
- Comment preservation
- Dirty flag lifecycle

### Real-World Scenarios
- Turtle configuration editing
- Concurrent read/write patterns
- Large file handling
- Permission edge cases

---

## Performance Expectations

### Memory
- Small files (<1MB): No noticeable overhead
- Large files: toml_edit memory usage proportional to file size

### Time
- Read: ~1-5ms for typical config files
- Write: ~2-10ms including atomic write
- Comment preservation: No additional cost vs non-preserving parser

### Scalability
- Single-threaded (editors are `Send + Sync` but not `Sync` for mutation)
- Good for TUI/CLI use cases
- Not suitable for highly concurrent server-side edits (use database for that)

---

## Rollout Plan

1. **PHASE4.1** (sl-m28): Test suite drives design
2. **PHASE4.2** (sl-m27): Core traits + TOML (biggest effort)
3. **PHASE4.3** (sl-m26): JSON + YAML backends (parallel possible)
4. **PHASE4.4** (sl-m25): Integration + real-world validation

**Timeline**: 2-3 weeks estimated

---

## Risk Mitigation

### Risk: TOML Comment Loss
**Mitigation**: Use toml_edit, not toml crate

### Risk: Atomic Write Failure
**Mitigation**: Test scenarios, handle edge cases (disk full, permissions)

### Risk: Dotted Path Navigation Complexity
**Mitigation**: Extensive testing of path resolution

### Risk: Feature Interaction with Phases 5-7
**Mitigation**: Design for composition, keep traits simple

---

## Related Documentation

- `history/PHASE3_COMPLETION_SUMMARY.md` - Phase 3 completion
- `history/CONSOLIDATED_ROADMAP.md` - All 7 phases overview
- `ref/PHASE3_MULTI_SCOPE_SUPPORT.md` - Phase 3 architecture
- `PHASE_TRACKING.md` - Progress tracking

---

## Conclusion

Phase 4 transforms settings-loader from read-only to fully bidirectional with format preservation, enabling real-world use cases like configuration editors and TUI-based admin tools.

**Key differentiator**: TOML comment preservation via toml_edit.

**Ready for implementation** when subtasks are broken down and tests designed.

---

**Created**: December 17, 2025  
**Status**: ✅ Design Ready  
**Next**: Subtask creation (sl-m28, sl-m27, sl-m26, sl-m25)
