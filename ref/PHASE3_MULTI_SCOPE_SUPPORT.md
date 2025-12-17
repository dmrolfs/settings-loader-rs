# Phase 3: Multi-Scope Configuration Support (Revised)

**Epic**: sl-ozp  
**Phase**: 3 (Multi-Scope Configuration Support)  
**Status**: TDD RED - Tests Updated, Ready for Implementation  
**Total Subtasks**: 4 (strict dependency chain)  
**Total Tests**: 20 pre-written (tests/phase3_multi_scope_tests.rs) - UPDATED for 6 scopes  
**Target**: Merge to feat/comprehensive-config-management-v1

---

## Overview

Phase 3 adds first-class support for multi-scope configuration using the `directories` crate for platform-standard paths.

**6 Configuration Scopes** (mapped to `directories` APIs):
1. **Preferences** - User application preferences (via `BaseDirs::preference_dir()`)
2. **UserGlobal** - User configuration that applies everywhere (via `ProjectDirs::config_dir()`)
3. **ProjectLocal** - Project-specific overrides (current directory)
4. **LocalData** - Machine-local runtime data (via `BaseDirs::data_local_dir()`)
5. **PersistentData** - Cross-machine persistent app state (via `BaseDirs::data_dir()`)
6. **Runtime** - Dynamic configuration (env vars + CLI, not file-based)

Applications can specify which scopes to load and in what order, with automatic path resolution using platform conventions.

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
    Preferences,      // User preferences (~/.config, ~/Library/Preferences, %APPDATA%)
    UserGlobal,       // User config (ProjectDirs::config_dir())
    ProjectLocal,     // ./settings.{ext} in current directory
    LocalData,        // Machine-local data (BaseDirs::data_local_dir())
    PersistentData,   // Cross-machine persistent data (BaseDirs::data_dir())
    Runtime,          // Environment variables + CLI
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
        vec![
            ConfigScope::Preferences,
            ConfigScope::UserGlobal,
            ConfigScope::ProjectLocal,
            ConfigScope::LocalData,
            ConfigScope::PersistentData,
        ]
    }
}
```

**Example Usage (Turtle)**:
```rust
impl MultiScopeConfig for TurtleConfig {
    const APP_NAME: &'static str = "spark-turtle";
    const ORG_NAME: &'static str = "spark-turtle";
    const CONFIG_BASENAME: &'static str = "settings";
    
    fn find_config_in(dir: &Path) -> Option<PathBuf> {
        crate::scope::find_config_in(dir)
    }
}

// Platform-appropriate paths are automatically resolved:
// macOS: 
//   - Preferences: ~/Library/Preferences/spark-turtle/settings.toml
//   - UserGlobal: ~/Library/Application Support/com.spark-turtle.spark-turtle/settings.toml
//   - ProjectLocal: ./settings.toml
//   - LocalData: ~/Library/Caches/com.spark-turtle.spark-turtle/
//   - PersistentData: ~/Library/Application Support/com.spark-turtle.spark-turtle/
// Linux (XDG):
//   - Preferences: ~/.config/spark-turtle/settings.toml (or XDG_CONFIG_HOME)
//   - UserGlobal: ~/.config/spark-turtle/settings.toml
//   - ProjectLocal: ./settings.toml
//   - LocalData: ~/.cache/spark-turtle/ (or XDG_CACHE_HOME)
//   - PersistentData: ~/.local/share/spark-turtle/ (or XDG_DATA_HOME)
// Windows:
//   - Preferences: %APPDATA%/spark-turtle/settings.toml
//   - UserGlobal: %APPDATA%/spark-turtle/settings.toml
//   - ProjectLocal: ./settings.toml
//   - LocalData: %LOCALAPPDATA%/spark-turtle/
//   - PersistentData: %APPDATA%/spark-turtle/

// With explicit layering:
impl LoadingOptions for TurtleConfig {
    fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
        builder
            .with_scopes::<TurtleConfig>(TurtleConfig::default_scopes())
            .with_env_vars(TurtleConfig::env_prefix(), TurtleConfig::env_separator())
    }
}
```

---

## 4 Subtasks with Dependencies

### PHASE3.1: Test Suite [TDD RED] (sl-x7d)

**File**: Update `tests/phase3_multi_scope_tests.rs`  
**Beads Issue**: sl-x7d

**Tests** (20 total - UPDATED for 6 scopes):

#### Core Enum Tests (2)
1. **test_config_scope_enum** - ConfigScope variants exist and behave correctly
2. **test_scope_equality_and_hashing** - ConfigScope can be used in collections

#### Scope Resolution Tests (6)
3. **test_resolve_path_preferences_scope** - Preferences scope uses BaseDirs::preference_dir()
4. **test_resolve_path_user_global_scope** - UserGlobal scope uses ProjectDirs::config_dir()
5. **test_resolve_path_project_local_scope** - ProjectLocal scope finds files in current dir
6. **test_resolve_path_local_data_scope** - LocalData scope uses BaseDirs::data_local_dir()
7. **test_resolve_path_persistent_data_scope** - PersistentData scope uses BaseDirs::data_dir()
8. **test_resolve_path_runtime_scope** - Runtime scope returns None (not file-based)

#### File Discovery Tests (5)
9. **test_find_config_toml_extension** - find_config_in searches for .toml files
10. **test_find_config_yaml_extension** - find_config_in searches for .yaml files
11. **test_find_config_json_extension** - find_config_in searches for .json files
12. **test_find_config_multiple_extensions** - Extension search order: toml > yaml > json > hjson > ron
13. **test_find_config_with_custom_basename** - find_config_in respects custom config basename

#### Trait Tests (4)
14. **test_multi_scope_config_trait** - MultiScopeConfig trait accessible and implementable
15. **test_default_scopes** - Default scope order: Preferences → UserGlobal → ProjectLocal → LocalData → PersistentData
16. **test_multi_scope_config_constants** - APP_NAME, ORG_NAME, CONFIG_BASENAME accessible
17. **test_multi_scope_find_config_in** - find_config_in trait method required

#### Integration Tests (3)
18. **test_turtle_scope_resolution** - Real-world: Turtle paths resolve correctly with all 6 scopes
19. **test_platform_specific_paths** - Paths use correct platform conventions (directories crate)
20. **test_multi_scope_with_layer_builder** - Multi-scope resolution works with LayerBuilder from Phase 1

**Acceptance**:
- [ ] tests/phase3_multi_scope_tests.rs updated with 20 tests
- [ ] All tests compile but fail (RED phase)
- [ ] Tests demonstrate all 6 scope scenarios
- [ ] Tests cover directories crate integration

**Blocks**: PHASE3.2 (sl-wcu)

---

### PHASE3.2: ConfigScope Enum & Find Logic [TDD GREEN] (sl-wcu)

**File**: Create/Update `src/scope.rs`  
**Beads Issue**: sl-wcu  
**Blocked by**: sl-x7d

Implement ConfigScope enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigScope {
    Preferences,      // User preferences (BaseDirs::preference_dir())
    UserGlobal,       // User config (ProjectDirs::config_dir())
    ProjectLocal,     // Current directory
    LocalData,        // Machine-local data (BaseDirs::data_local_dir())
    PersistentData,   // Cross-machine data (BaseDirs::data_dir())
    Runtime,          // Env vars + CLI (not file-based)
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

**Tests**: Tests 1-2, 9-13 pass (ConfigScope + find_config_in)

**Acceptance**:
- [ ] ConfigScope enum created with 6 variants
- [ ] ConfigScope derives Debug, Clone, Copy, PartialEq, Eq, Hash
- [ ] find_config_in() searches multiple extensions in correct order
- [ ] Tests 1-2, 9-13 passing

**Blocked by**: PHASE3.1 (sl-x7d)  
**Blocks**: PHASE3.3 (sl-4ug)

---

### PHASE3.3: MultiScopeConfig Trait [TDD GREEN] (sl-4ug)

**File**: Modify `src/loading_options.rs` to add trait  
**Beads Issue**: sl-4ug  
**Blocked by**: sl-wcu

Implement `MultiScopeConfig` trait:

```rust
pub trait MultiScopeConfig: LoadingOptions {
    const APP_NAME: &'static str;
    const ORG_NAME: &'static str = "";
    const CONFIG_BASENAME: &'static str = "settings";
    
    fn resolve_path(scope: ConfigScope) -> Option<PathBuf> {
        match scope {
            ConfigScope::Preferences => Self::preferences_path(),
            ConfigScope::UserGlobal => Self::user_global_path(),
            ConfigScope::ProjectLocal => Self::project_local_path(),
            ConfigScope::LocalData => Self::local_data_path(),
            ConfigScope::PersistentData => Self::persistent_data_path(),
            ConfigScope::Runtime => None,
        }
    }
    
    fn preferences_path() -> Option<PathBuf> {
        #[cfg(feature = "multi-scope")]
        {
            use directories::BaseDirs;
            let dirs = BaseDirs::new()?;
            let pref_dir = dirs.preference_dir();
            let app_dir = pref_dir.join(Self::APP_NAME);
            Self::find_config_in(&app_dir)
        }
        #[cfg(not(feature = "multi-scope"))]
        None
    }
    
    fn user_global_path() -> Option<PathBuf> {
        #[cfg(feature = "multi-scope")]
        {
            use directories::ProjectDirs;
            let proj = ProjectDirs::new(
                Self::ORG_NAME,
                Self::ORG_NAME,
                Self::APP_NAME,
            )?;
            let config_dir = proj.config_dir();
            Self::find_config_in(config_dir)
        }
        #[cfg(not(feature = "multi-scope"))]
        None
    }
    
    fn project_local_path() -> Option<PathBuf> {
        let current_dir = std::env::current_dir().ok()?;
        Self::find_config_in(&current_dir)
    }
    
    fn local_data_path() -> Option<PathBuf> {
        #[cfg(feature = "multi-scope")]
        {
            use directories::BaseDirs;
            let dirs = BaseDirs::new()?;
            let data_dir = dirs.data_local_dir();
            let app_dir = data_dir.join(Self::APP_NAME);
            Self::find_config_in(&app_dir)
        }
        #[cfg(not(feature = "multi-scope"))]
        None
    }
    
    fn persistent_data_path() -> Option<PathBuf> {
        #[cfg(feature = "multi-scope")]
        {
            use directories::BaseDirs;
            let dirs = BaseDirs::new()?;
            let data_dir = dirs.data_dir();
            let app_dir = data_dir.join(Self::APP_NAME);
            Self::find_config_in(&app_dir)
        }
        #[cfg(not(feature = "multi-scope"))]
        None
    }
    
    fn find_config_in(dir: &Path) -> Option<PathBuf>;
    
    fn default_scopes() -> Vec<ConfigScope> {
        vec![
            ConfigScope::Preferences,
            ConfigScope::UserGlobal,
            ConfigScope::ProjectLocal,
            ConfigScope::LocalData,
            ConfigScope::PersistentData,
        ]
    }
}
```

**Tests**: Tests 3-20 pass (all scope resolution + trait functionality)

**Acceptance**:
- [ ] MultiScopeConfig trait created with required constants
- [ ] resolve_path() implements 6-scope dispatch logic
- [ ] preferences_path() uses BaseDirs::preference_dir()
- [ ] user_global_path() uses ProjectDirs::config_dir()
- [ ] project_local_path() searches current directory
- [ ] local_data_path() uses BaseDirs::data_local_dir()
- [ ] persistent_data_path() uses BaseDirs::data_dir()
- [ ] All path methods guarded by `multi-scope` feature flag
- [ ] find_config_in() trait method required
- [ ] default_scopes() returns all 6 scopes in correct order
- [ ] Tests 3-20 passing
- [ ] Uses `directories` crate feature flag

**Blocked by**: PHASE3.2 (sl-wcu)  
**Blocks**: PHASE3.4 (sl-evw)

---

### PHASE3.4: Integration with LayerBuilder [TDD GREEN] (sl-evw)

**File**: `src/layer.rs`, `src/settings_loader.rs`  
**Beads Issue**: sl-evw  
**Blocked by**: sl-4ug

Modify `LayerBuilder` to add convenience method:

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

**Tests**: All 20 tests passing (integration verified)

**Acceptance**:
- [ ] LayerBuilder can load multiple scopes
- [ ] with_scopes() method added
- [ ] Path resolution uses MultiScopeConfig
- [ ] All Phase 1-2 tests still passing (backward compatibility)
- [ ] All 20 Phase 3 tests passing
- [ ] No unsafe code
- [ ] Code formatted, 0 clippy warnings

**Blocked by**: PHASE3.3 (sl-4ug)  
**Status**: Ready for implementation after TDD tests written

---

## Success Criteria

**Definition of Done**:
- ✅ All 20 tests in `tests/phase3_multi_scope_tests.rs` passing
- ✅ All existing tests still passing (backward compatibility: 39 tests from Phase 1-2)
- ✅ 0 code clippy warnings
- ✅ Code formatted with `cargo fmt`
- ✅ ConfigScope enum functional with all 6 variants
- ✅ MultiScopeConfig trait accessible and working
- ✅ Platform-specific path resolution correct (via directories crate)
- ✅ All scopes integrated with LayerBuilder

**Code Quality Gates**:
- ✅ Zero clippy warnings
- ✅ All 59 tests passing (39 + 20 new)
- ✅ Code formatted
- ✅ No unsafe code
- ✅ Backward compatible (all existing code unchanged)

---

## Design Decisions

### Why 6 Scopes Instead of 4?

**Original 4 scopes**: System, UserGlobal, ProjectLocal, Runtime

**New 6 scopes**: Preferences, UserGlobal, ProjectLocal, LocalData, PersistentData, Runtime

**Rationale**:
- **Preferences**: User application preferences (maps to `BaseDirs::preference_dir()`)
- **UserGlobal**: User configuration across projects (maps to `ProjectDirs::config_dir()`)
- **ProjectLocal**: Project-specific overrides (current directory, always searchable)
- **LocalData**: Machine-local runtime data, not synced (maps to `BaseDirs::data_local_dir()`)
- **PersistentData**: Cross-machine persistent state (maps to `BaseDirs::data_dir()`)
- **Runtime**: Dynamic config from env vars + CLI (not file-based)

This aligns with how modern applications structure configuration and data storage.

### Why Use directories Crate for All File-Based Scopes?

**Decision**: Use `directories` crate APIs exclusively for file-based path resolution

**Rationale**:
- **Platform consistency**: Automatically handles XDG on Linux, ~/Library on macOS, %APPDATA% on Windows
- **Standard conventions**: Follows OS best practices without manual path construction
- **No hardcoded paths**: Eliminates `/etc` authz issues and makes system-level config optional
- **Cleaner API**: All paths derive from documented `directories` crate semantics
- **Feature-gated**: `multi-scope` feature controls `directories` dependency

### MultiScopeConfig as Separate Trait

**Decision**: `MultiScopeConfig: LoadingOptions` composition

**Rationale**:
- Apps can use LoadingOptions without scopes (optional)
- Scopes only loaded for apps that need them
- Doesn't break existing implementations
- Clear separation of concerns

---

## Test Strategy

### Tests 1-2: ConfigScope Enum
Verify enum structure and collection compatibility.

### Tests 3-8: Scope Resolution
Verify each scope resolves to correct path using directories crate.

### Tests 9-13: File Format Detection
Verify find_config_in searches multiple extensions in order.

### Tests 14-17: Trait Functionality
Verify MultiScopeConfig trait accessibility and method signatures.

### Tests 18-20: Real-World Integration
Demonstrate Turtle use case, platform correctness, LayerBuilder integration.

---

## Implementation Notes

1. **No breaking changes**: All new, optional traits
2. **Feature flag**: `multi-scope` feature gates `directories` dependency
3. **Extension search**: Order matters - prefer toml, then yaml, then json
4. **Platform detection**: Use `#[cfg(target_os = "...")]` for OS-specific behaviors
5. **Path search**: ProjectLocal search starts from current directory
6. **Backward compatible**: Existing code works unchanged
7. **directories crate**: All file-based scopes use `directories` APIs

---

## Feature Flag

Add to `Cargo.toml`:

```toml
[features]
multi-scope = ["directories"]

[dependencies]
directories = { version = "5.0", optional = true }
```

Only enable in tests/code that use MultiScopeConfig:

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
  loading_options.rs          # MODIFIED: Add MultiScopeConfig trait with 6 scope methods
  layer.rs                    # MODIFIED: with_scopes() convenience method
  settings_loader.rs          # UNCHANGED

tests/
  phase3_multi_scope_tests.rs # UPDATED: 20 tests (was 14, now covers 6 scopes)
  phase2_env_customization_tests.rs  # UNCHANGED: 12 tests (Phase 2)
  layer_builder_tests.rs             # UNCHANGED: 27 tests (Phase 1)

ref/
  PHASE3_MULTI_SCOPE_SUPPORT.md      # This file (REVISED for 6 scopes)
```

---

## Progress Tracking

- [ ] PHASE3.1 (sl-x7d) - TDD RED - Update test file for 6 scopes
- [ ] PHASE3.2 (sl-wcu) - TDD GREEN - Implement ConfigScope enum
- [ ] PHASE3.3 (sl-4ug) - TDD GREEN - Implement MultiScopeConfig trait
- [ ] PHASE3.4 (sl-evw) - TDD GREEN - Verify integration
- [ ] Validation - All tests pass, quality gates verified

**Beads Issue Dependencies**:
```
sl-x7d (PHASE3.1)
  ↓
sl-wcu (PHASE3.2)
  ↓
sl-4ug (PHASE3.3)
  ↓
sl-evw (PHASE3.4)
```

---

## Next Steps

1. Review and accept test updates (20 tests for 6 scopes)
2. Begin PHASE3.1 (sl-x7d): Verify test file compiles
3. Then PHASE3.2 (sl-wcu): Implement ConfigScope enum  
4. Then PHASE3.3 (sl-4ug): Implement MultiScopeConfig trait
5. Then PHASE3.4 (sl-evw): Add with_scopes() to LayerBuilder
6. Run comprehensive validation: `cargo test --all && cargo clippy && cargo fmt`
7. All tests passing → ready for merging to feat/comprehensive-config-management-v1

---

## Related Documents

- `history/CONSOLIDATED_ROADMAP.md` - Phase 3 overview + Phase 4-7 vision
- `ref/PHASE2_ENV_CUSTOMIZATION.md` - Phase 2 completed work
- `ref/PHASE1_IMPLEMENTATION_PLAN.md` - Phase 1 completed work
- `tests/layer_builder_tests.rs` - Phase 1 test patterns (reference)
- `tests/phase2_env_customization_tests.rs` - Phase 2 test patterns (reference)
- `tests/phase3_multi_scope_tests.rs` - Phase 3 tests (UPDATED)
