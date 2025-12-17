# Phase 3: Multi-Scope Configuration Support

**Epic**: sl-xxx (to be assigned)  
**Phase**: 3 (Multi-Scope Configuration Support)  
**Status**: Planning (Design & TDD phase)  
**Total Subtasks**: 4 (strict dependency chain)  
**Total Tests**: 14 pre-written (tests/phase3_multi_scope_tests.rs)  
**Target**: Merge to feat/comprehensive-config-management-v1

---

## Overview

Phase 3 adds first-class support for multi-scope configuration: System (read-only defaults), UserGlobal (user settings), ProjectLocal (per-project settings), and Runtime (env vars + CLI).

Applications can now specify which scopes to load and in what order, with automatic path resolution using platform conventions.

Follows **TDD RED → GREEN → REFACTOR** cycle:
- **RED** (Phase 3.1): Create test file with failing tests
- **GREEN** (Phase 3.2): Implement ConfigScope enum
- **GREEN** (Phase 3.3): Implement MultiScopeConfig trait
- **INTEGRATION** (Phase 3.4): Verify Phase 1-2 integration + backward compatibility
- **VALIDATION**: All tests pass, quality gates verified

---

## What Gets Built

**New Enum**: `ConfigScope`
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigScope {
    System,       // System-wide defaults (read-only)
    UserGlobal,   // ~/.config/app-name/ (or platform equivalent)
    ProjectLocal, // ./app.toml, ./app.yaml, etc. (searchable)
    Runtime,      // Environment variables + CLI
}
```

**New Trait**: `MultiScopeConfig`
```rust
pub trait MultiScopeConfig: LoadingOptions {
    const APP_NAME: &'static str;
    const ORG_NAME: &'static str = "";
    const CONFIG_BASENAME: &'static str = "settings";
    
    // Path resolution for each scope
    fn resolve_path(scope: ConfigScope) -> Option<PathBuf>;
    
    // Search for config file with multiple extensions
    fn find_config_in(dir: &Path) -> Option<PathBuf>;
    
    // Get default scopes to load (in order)
    fn default_scopes() -> Vec<ConfigScope> {
        vec![ConfigScope::System, ConfigScope::UserGlobal, ConfigScope::ProjectLocal]
    }
}
```

**Example Usage (Turtle)**:
```rust
impl MultiScopeConfig for TurtleConfig {
    const APP_NAME: &'static str = "spark-turtle";
    const ORG_NAME: &'static str = "spark-turtle";
    const CONFIG_BASENAME: &'static str = "settings";
}

// Platform-appropriate paths are automatically resolved:
// macOS: ~/Library/Application Support/spark-turtle/settings.toml
// Linux: ~/.config/spark-turtle/settings.toml
// Windows: %APPDATA%/spark-turtle/settings.toml

// With explicit layering:
impl LoadingOptions for TurtleConfig {
    fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
        builder
            .with_path(TurtleConfig::resolve_path(ConfigScope::System).unwrap())
            .with_path(TurtleConfig::resolve_path(ConfigScope::UserGlobal).unwrap())
            .with_path(TurtleConfig::find_config_in(&PathBuf::from(".")).unwrap())
            .with_env_vars(TurtleConfig::env_prefix(), TurtleConfig::env_separator())
    }
}
```

---

## 4 Subtasks with Dependencies

### PHASE3.1: Test Suite [TDD RED]

**File**: Create `tests/phase3_multi_scope_tests.rs`

**Tests** (14 total):

1. **test_config_scope_enum** - ConfigScope variants exist and behave correctly
2. **test_resolve_path_system_scope** - System scope resolves to /etc/app or equivalent
3. **test_resolve_path_user_global_scope** - UserGlobal scope uses platform conventions
4. **test_resolve_path_project_local_scope** - ProjectLocal scope finds files in current dir
5. **test_resolve_path_runtime_scope** - Runtime scope returns None (not file-based)
6. **test_find_config_yaml_extension** - find_config_in searches for .yaml files
7. **test_find_config_toml_extension** - find_config_in searches for .toml files
8. **test_find_config_json_extension** - find_config_in searches for .json files
9. **test_find_config_multiple_extensions** - Extension search order: toml, yaml, json
10. **test_multi_scope_config_trait** - MultiScopeConfig trait accessible
11. **test_default_scopes** - Default scope order is System → UserGlobal → ProjectLocal
12. **test_turtle_scope_resolution** - Real-world: Turtle paths resolve correctly
13. **test_platform_specific_paths** - Paths use platform conventions (xdg_dirs style)
14. **test_scope_equality_and_hashing** - ConfigScope can be used in collections

**Acceptance**:
- [ ] tests/phase3_multi_scope_tests.rs created
- [ ] 14 tests compile but fail (RED phase)
- [ ] Tests demonstrate all scenarios

**Blocks**: PHASE3.2

---

### PHASE3.2: ConfigScope Enum & Find Logic [TDD GREEN]

**File**: Create `src/scope.rs`

Implement ConfigScope enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigScope {
    System,
    UserGlobal,
    ProjectLocal,
    Runtime,
}
```

Implement `find_config_in()` function (module-level utility):

```rust
pub fn find_config_in(dir: &Path) -> Option<PathBuf> {
    // Search in order: .toml, .yaml, .yml, .json, .hjson, .ron
    for ext in &["toml", "yaml", "yml", "json", "hjson", "ron"] {
        let path = dir.join(format!("settings.{}", ext));
        if path.exists() {
            return Some(path);
        }
    }
    None
}
```

**Tests**: Tests 1-2, 14 pass

**Acceptance**:
- [ ] ConfigScope enum created with 4 variants
- [ ] ConfigScope derives Debug, Clone, Copy, PartialEq, Eq, Hash
- [ ] find_config_in() searches multiple extensions in correct order
- [ ] Tests 1-2, 14 passing

**Blocked by**: PHASE3.1  
**Blocks**: PHASE3.3

---

### PHASE3.3: MultiScopeConfig Trait [TDD GREEN]

**File**: Modify `src/loading_options.rs` to add trait, create helper functions

Implement `MultiScopeConfig` trait:

```rust
pub trait MultiScopeConfig: LoadingOptions {
    const APP_NAME: &'static str;
    const ORG_NAME: &'static str = "";
    const CONFIG_BASENAME: &'static str = "settings";
    
    fn resolve_path(scope: ConfigScope) -> Option<PathBuf> {
        match scope {
            ConfigScope::System => Self::system_path(),
            ConfigScope::UserGlobal => Self::user_global_path(),
            ConfigScope::ProjectLocal => Self::project_local_path(),
            ConfigScope::Runtime => None,
        }
    }
    
    fn system_path() -> Option<PathBuf> {
        // /etc/app-name/settings.{ext}
        // Implementation differs by platform
        #[cfg(target_os = "linux")]
        {
            let dir = PathBuf::from("/etc").join(Self::APP_NAME);
            find_config_in(&dir)
        }
        #[cfg(not(target_os = "linux"))]
        None
    }
    
    fn user_global_path() -> Option<PathBuf> {
        // Uses directories crate for platform conventions
        use directories::ProjectDirs;
        
        let proj = ProjectDirs::new(Self::ORG_NAME, Self::ORG_NAME, Self::APP_NAME)?;
        let config_dir = proj.config_dir();
        find_config_in(config_dir)
    }
    
    fn project_local_path() -> Option<PathBuf> {
        // Search current directory for config file
        find_config_in(&std::env::current_dir().ok()?)
    }
    
    fn find_config_in(dir: &Path) -> Option<PathBuf>;
    
    fn default_scopes() -> Vec<ConfigScope> {
        vec![ConfigScope::System, ConfigScope::UserGlobal, ConfigScope::ProjectLocal]
    }
}
```

**Tests**: Tests 3-13 pass

**Acceptance**:
- [ ] MultiScopeConfig trait created with required constants
- [ ] resolve_path() implements platform-specific logic
- [ ] find_config_in() trait method required
- [ ] default_scopes() returns correct order
- [ ] Tests 3-13 passing
- [ ] Uses `directories` crate feature flag

**Blocked by**: PHASE3.2  
**Blocks**: PHASE3.4

---

### PHASE3.4: Integration with LayerBuilder [TDD GREEN]

**File**: `src/layer.rs`, `src/settings_loader.rs`

Modify `LayerBuilder::build()` or create convenience function to use MultiScopeConfig:

```rust
impl LayerBuilder {
    /// Load configuration from multiple scopes with automatic path resolution
    /// 
    /// Requires MultiScopeConfig trait implementation for path resolution
    pub fn with_scopes<T: MultiScopeConfig>(
        mut self,
        scopes: impl IntoIterator<Item = ConfigScope>
    ) -> Self {
        for scope in scopes {
            if let Some(path) = T::resolve_path(scope) {
                self = self.with_path(path);
            }
        }
        self
    }
}
```

**Example**:
```rust
let builder = LayerBuilder::new()
    .with_scopes::<TurtleConfig>(TurtleConfig::default_scopes())
    .with_env_vars(TurtleConfig::env_prefix(), TurtleConfig::env_separator());
```

**Tests**: All 14 tests passing

**Acceptance**:
- [ ] LayerBuilder can load multiple scopes
- [ ] Path resolution uses MultiScopeConfig
- [ ] All Phase 1-2 tests still passing (backward compatibility)
- [ ] All 14 Phase 3 tests passing
- [ ] No unsafe code
- [ ] Code formatted, 0 clippy warnings

**Blocked by**: PHASE3.3  
**Status**: Ready for implementation after TDD tests written

---

## Success Criteria

**Definition of Done**:
- ✅ All 14 tests in `tests/phase3_multi_scope_tests.rs` passing
- ✅ All existing tests still passing (backward compatibility: 39 tests from Phase 1-2)
- ✅ 0 code clippy warnings
- ✅ Code formatted with `cargo fmt`
- ✅ ConfigScope enum functional with all 4 variants
- ✅ MultiScopeConfig trait accessible and working
- ✅ Platform-specific path resolution correct

**Code Quality Gates**:
- ✅ Zero clippy warnings
- ✅ All 53 tests passing (39 + 14 new)
- ✅ Code formatted
- ✅ No unsafe code
- ✅ Backward compatible (all existing code unchanged)

---

## Design Decisions

### Why 4 Scopes?

**Scopes chosen**: System, UserGlobal, ProjectLocal, Runtime

**Rationale**:
- System: Immutable defaults (like /etc/app-name)
- UserGlobal: User preferences that apply everywhere (like ~/.config/app-name)
- ProjectLocal: Project-specific overrides (like ./app.toml)
- Runtime: Dynamic configuration (env vars, CLI)

Each layer can be optional and searchable.

### Platform Conventions

**Decision**: Use `directories` crate for platform-specific paths

**Rationale**:
- macOS: ~/Library/Application Support/app-name
- Linux: ~/.config/app-name (XDG defaults)
- Windows: %APPDATA%/app-name
- No manual path construction needed

### MultiScopeConfig as Separate Trait

**Decision**: `MultiScopeConfig: LoadingOptions` composition

**Rationale**:
- Apps can use LoadingOptions without scopes (optional)
- Scopes only loaded for apps that need them
- Doesn't break existing implementations
- Clear separation of concerns

---

## Test Strategy

### Tests 1-2: ConfigScope & System Paths
Verify enum structure and system scope resolution.

### Tests 3-5: Platform Path Resolution
Verify UserGlobal and ProjectLocal use correct directories.

### Tests 6-9: File Format Detection
Verify find_config_in searches multiple extensions.

### Test 10: Trait Accessibility
Verify MultiScopeConfig trait accessible.

### Tests 11-13: Real-World Scenarios
Demonstrate Turtle use case, default scopes, path correctness.

### Test 14: Scope as Collection Keys
Verify ConfigScope works in HashMap/HashSet.

---

## Implementation Notes

1. **No breaking changes**: All new, optional traits
2. **Feature flag**: `multi-scope` feature guards `directories` dependency
3. **Extension search**: Order matters - prefer toml, then yaml, then json
4. **Platform detection**: Use `#[cfg(target_os = "...")]` for OS-specific paths
5. **Path search**: ProjectLocal search starts from current directory
6. **Backward compatible**: Existing code works unchanged

---

## Feature Flag

Add to `Cargo.toml`:

```toml
[features]
multi-scope = ["directories"]

[dev-dependencies]
# ... existing ...
```

Only enable in tests that use MultiScopeConfig:

```rust
#[cfg(feature = "multi-scope")]
#[test]
fn test_multi_scope_feature() {
    // ...
}
```

---

## File Structure After Completion

```
src/
  scope.rs                    # NEW: ConfigScope enum, find_config_in utility
  loading_options.rs          # MODIFIED: Add MultiScopeConfig trait
  layer.rs                    # MODIFIED: with_scopes() convenience method
  settings_loader.rs          # UNCHANGED

tests/
  phase3_multi_scope_tests.rs # NEW: 14 tests
  phase2_env_customization_tests.rs  # UNCHANGED: 12 tests (Phase 2)
  layer_builder_tests.rs             # UNCHANGED: 27 tests (Phase 1)

ref/
  PHASE3_MULTI_SCOPE_SUPPORT.md      # This file
```

---

## Progress Tracking

- [ ] PHASE3.1 (TDD RED) - Create test file
- [ ] PHASE3.2 (TDD GREEN) - Implement ConfigScope enum
- [ ] PHASE3.3 (TDD GREEN) - Implement MultiScopeConfig trait
- [ ] PHASE3.4 (TDD GREEN) - Verify integration
- [ ] Validation - All tests pass, quality gates verified

---

## Next Steps

1. Create tests/phase3_multi_scope_tests.rs with 14 failing tests (RED phase)
2. Review test design and accept
3. Implement ConfigScope enum in src/scope.rs (GREEN phase)
4. Implement MultiScopeConfig trait in src/loading_options.rs (GREEN phase)
5. Add with_scopes() convenience to LayerBuilder
6. Run validation: `cargo test --all && cargo clippy && cargo fmt`
7. Merge to feat/comprehensive-config-management-v1

---

## Related Documents

- `history/CONSOLIDATED_ROADMAP.md` - Phase 3 overview + Phase 4-7 vision
- `ref/PHASE2_ENV_CUSTOMIZATION.md` - Phase 2 completed work
- `ref/PHASE1_IMPLEMENTATION_PLAN.md` - Phase 1 completed work
- `tests/layer_builder_tests.rs` - Phase 1 test patterns (reference)
- `tests/phase2_env_customization_tests.rs` - Phase 2 test patterns (reference)
- `tests/phase3_multi_scope_tests.rs` - This phase's tests (TBD)
