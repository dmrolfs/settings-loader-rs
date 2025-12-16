# Implementation Plan - Task sl-h8h - Phase 1

**Epic**: sl-h8h - Explicit Configuration Layering API  
**Phase**: 1 (Explicit Layering)  
**Status**: Ready for Implementation  
**Total Subtasks**: 8 (strict dependency chain)  
**Total Tests**: 25 pre-written (tests/layer_builder_tests.rs)  
**Target**: v0.16.0

---

## Overview

Phase 1 implements the Explicit Configuration Layering API - allows applications to compose configuration sources with clear, explicit precedence. Later layers override earlier layers.

Follows **RED → GREEN → REFACTOR** TDD cycle:
- **RED** (sl-bru): Create types, tests fail with stub implementations
- **GREEN** (sl-bv4 through sl-5h9): Implement features incrementally
- **INTEGRATION** (sl-apw, sl-mvm): Connect to existing SettingsLoader
- **CLEANUP** (sl-uda): Documentation and exports
- **VALIDATION** (sl-6pk): All tests pass, quality gates verified

---

## What Gets Built

**New Type**: `ConfigLayer` enum (5 variants)
- `Path(PathBuf)` - Explicit file path
- `EnvVar(String)` - Config path from environment variable
- `EnvSearch { env: Environment, dirs: Vec<PathBuf> }` - Environment-directed search
- `Secrets(PathBuf)` - Secrets file
- `EnvVars { prefix: String, separator: String }` - System environment variables

**New Type**: `LayerBuilder` struct
- Fluent API for composing layers
- `new()`, `with_path()`, `with_env_var()`, `with_secrets()`, `with_env_vars()`
- Query methods: `layers()`, `layer_count()`, `is_empty()`, `has_layers()`
- Test helpers: `has_path_layer()`, `has_env_var_layer()`, etc.
- Core: `build()` → `ConfigBuilder<DefaultState>`

**Enhanced Trait**: `LoadingOptions`
- New optional method: `build_layers(builder: LayerBuilder) -> LayerBuilder`
- Default impl: returns builder unchanged (backward compatible)

**Modified Trait**: `SettingsLoader`
- `load()` checks for explicit layers first
- Falls back to implicit layering if no layers defined
- 100% backward compatible

---

## 8 Subtasks with Dependencies

### PHASE1.1: Core Types [sl-bru] (RED)

**Files**: Create `src/layer.rs`

```rust
#[derive(Debug, Clone)]
pub enum ConfigLayer {
    Path(PathBuf),
    EnvVar(String),
    EnvSearch { env: Environment, dirs: Vec<PathBuf> },
    Secrets(PathBuf),
    EnvVars { prefix: String, separator: String },
}

pub struct LayerBuilder {
    layers: Vec<ConfigLayer>,
}
```

**Tests**: 1-6 compile, fail (RED phase)

**Acceptance**:
- [ ] `src/layer.rs` created with ConfigLayer enum (5 variants)
- [ ] LayerBuilder struct with Vec<ConfigLayer> storage
- [ ] Debug, Clone traits derived
- [ ] Tests 1-6 compile and fail

**Blocking**: Blocks sl-bv4

---

### PHASE1.2: Builder Methods [sl-bv4] (GREEN)

**File**: `src/layer.rs`

Implement fluent builder interface:
```rust
impl LayerBuilder {
    pub fn new() -> Self { ... }
    pub fn with_path(mut self, path: impl AsRef<Path>) -> Self { ... }
    pub fn with_env_var(mut self, var_name: &str) -> Self { ... }
    pub fn with_secrets(mut self, path: impl AsRef<Path>) -> Self { ... }
    pub fn with_env_vars(mut self, prefix: &str, separator: &str) -> Self { ... }
    
    pub fn layers(&self) -> &[ConfigLayer] { ... }
    pub fn layer_count(&self) -> usize { ... }
    pub fn is_empty(&self) -> bool { ... }
    
    // Test helpers
    pub fn has_path_layer(&self) -> bool { ... }
    pub fn has_env_var_layer(&self, name: &str) -> bool { ... }
    pub fn has_secrets_layer(&self) -> bool { ... }
    pub fn has_env_vars_layer(&self, prefix: &str, sep: &str) -> bool { ... }
}
```

**Tests**: Tests 1-9 pass (GREEN)

**Acceptance**:
- [ ] All builder methods implemented
- [ ] All query/helper methods work
- [ ] Tests 1-9 passing
- [ ] Fluent API chaining works

**Blocked by**: sl-bru  
**Blocks**: sl-nz1

---

### PHASE1.3: Build Implementation [sl-nz1] (GREEN)

**File**: `src/layer.rs`

Implement `LayerBuilder::build()` - the core:

```rust
pub fn build(self) -> Result<ConfigBuilder<DefaultState>, SettingsError> {
    let mut config = Config::builder();
    
    for layer in self.layers {
        config = match layer {
            ConfigLayer::Path(path) => {
                let abs_path = path.absolutize()?;
                config.add_source(
                    ConfigFile::from(abs_path.as_ref()).required(true)
                )
            }
            ConfigLayer::EnvVar(var_name) => {
                match std::env::var(&var_name) {
                    Ok(env_path) => {
                        let abs_path = Path::new(&env_path).absolutize()?;
                        config.add_source(
                            ConfigFile::from(abs_path.as_ref()).required(true)
                        )
                    }
                    Err(std::env::VarError::NotPresent) => config,  // Skip gracefully
                    Err(e) => return Err(e.into()),
                }
            }
            ConfigLayer::Secrets(path) => {
                let abs_path = path.absolutize()?;
                config.add_source(
                    ConfigFile::from(abs_path.as_ref()).required(true)
                )
            }
            ConfigLayer::EnvVars { prefix, separator } => {
                config.add_source(
                    config::Environment::default()
                        .prefix(&prefix)
                        .separator(&separator)
                        .try_parsing(true)
                )
            }
            ConfigLayer::EnvSearch { .. } => config,  // Defer to Phase 3
        };
    }
    
    Ok(config)
}
```

**Key Behavior**:
- Layer precedence via config crate's deep merge (later `.add_source()` wins)
- Path files MUST exist → error
- Secrets files MUST exist → error
- EnvVar missing → skip gracefully (NO error)
- Use `path_absolutize` for consistent path handling

**Tests**: Tests 10-22 pass (GREEN) - precedence, formats, errors, env vars

**Acceptance**:
- [ ] ConfigLayer::Path loads files with absolutize
- [ ] ConfigLayer::EnvVar resolves with graceful skip
- [ ] ConfigLayer::Secrets requires file to exist
- [ ] ConfigLayer::EnvVars integrates with config crate Environment
- [ ] Layer precedence correct (later overrides earlier)
- [ ] Error handling for missing Path/Secrets files
- [ ] Tests 10-22 passing
- [ ] All file formats work (YAML, JSON, TOML auto-detected)

**Blocked by**: sl-bv4  
**Blocks**: sl-5h9

---

### PHASE1.4: File Format Detection [sl-5h9] (GREEN)

**File**: `src/layer.rs`

No custom code needed. Leverage config crate's built-in format detection:
- `.yaml`, `.yml` → YAML
- `.json` → JSON
- `.toml` → TOML
- `.hjson` → HJSON
- `.ron` → RON

ConfigFile already detects from extension automatically.

**Tests**: Tests 12-15 pass (formats already working from Phase 1.3)

**Acceptance**:
- [ ] YAML support verified
- [ ] JSON support verified
- [ ] TOML support verified
- [ ] Format detection automatic
- [ ] Tests 12-15 passing

**Blocked by**: sl-nz1  
**Blocks**: sl-apw

---

### PHASE1.5: SettingsLoader Integration [sl-apw] (GREEN)

**File**: `src/settings_loader.rs`

Modify `SettingsLoader::load()` to support explicit layering with fallback:

```rust
#[tracing::instrument(level = "info")]
fn load(options: &Self::Options) -> Result<Self, SettingsError>
where
    Self: DeserializeOwned,
{
    let initial_builder = LayerBuilder::new();
    let builder = options.build_layers(initial_builder);
    
    // If explicit layers defined, use them
    if builder.has_layers() {
        let config_builder = builder.build()?;
        let config = config_builder.build()?;
        return config.try_deserialize().map_err(Into::into);
    }
    
    // Fall back to implicit layering for backward compatibility
    Self::load_implicit(options)
}

// Extract current load() logic here
fn load_implicit(options: &Self::Options) -> Result<Self, SettingsError>
where
    Self: DeserializeOwned,
{
    // Current implementation moved here
    ...
}
```

**Key Points**:
- Explicit layers take precedence
- If no explicit layers, use existing implicit layering
- 100% backward compatible

**Tests**: All existing tests pass (backward compatibility verified)

**Acceptance**:
- [ ] `load()` modified to check explicit layers
- [ ] `has_layers()` implemented on LayerBuilder
- [ ] Fall back to `load_implicit()` if no layers
- [ ] All existing tests still passing
- [ ] No changes to existing test fixtures

**Blocked by**: sl-5h9  
**Blocks**: sl-mvm

---

### PHASE1.6: LoadingOptions Enhancement [sl-mvm] (GREEN)

**File**: `src/lib.rs`

Add optional trait method with default implementation:

```rust
pub trait LoadingOptions: Sized {
    // ... existing methods ...
    
    /// Build explicit configuration layers.
    /// 
    /// Default implementation returns builder unchanged (backward compatible).
    /// Override to define explicit layer composition.
    fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
        builder
    }
}
```

**Key Points**:
- New trait method with default implementation
- Zero breaking changes - all existing implementors work unchanged
- Apps can override to define explicit layers

**Tests**: All existing tests still pass

**Acceptance**:
- [ ] LoadingOptions trait modified
- [ ] `build_layers()` method added with default impl
- [ ] All existing tests still pass
- [ ] No trait implementors require changes

**Blocked by**: sl-apw  
**Blocks**: sl-uda

---

### PHASE1.7: Documentation & Exports [sl-uda] (GREEN)

**File**: `src/lib.rs`

Export types from module:
```rust
pub mod layer;
pub use layer::{ConfigLayer, LayerBuilder};
```

Add comprehensive docs to `src/layer.rs`:
```rust
//! Explicit configuration layering API.
//!
//! Allows applications to compose configuration sources with clear, explicit 
//! precedence. Later layers override earlier layers.
//!
//! # Layer Types
//!
//! - `Path` - Load from explicit file path
//! - `EnvVar` - Load from path specified in environment variable
//! - `Secrets` - Load from secrets file
//! - `EnvVars` - Load from system environment variables with prefix
//!
//! # Example
//!
//! ```rust,ignore
//! use settings_loader::{LayerBuilder, SettingsLoader, LoadingOptions};
//!
//! impl LoadingOptions for MyOptions {
//!     fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
//!         builder
//!             .with_path("config.yaml")
//!             .with_secrets("secrets.yaml")
//!             .with_env_vars("APP", "__")
//!     }
//! }
//!
//! let settings = MySettings::load(&options)?;
//! ```
```

Document each variant of ConfigLayer with examples.

**Tests**: Doc tests passing

**Acceptance**:
- [ ] ConfigLayer exported from lib.rs
- [ ] LayerBuilder exported from lib.rs
- [ ] Module-level docs complete
- [ ] ConfigLayer enum docs
- [ ] LayerBuilder struct docs
- [ ] Working examples in docs
- [ ] Backward compatibility examples

**Blocked by**: sl-mvm  
**Blocks**: sl-6pk

---

### PHASE1.8: Validation & Quality [sl-6pk] (REFACTOR)

**Files**: All

Run comprehensive validation:

```bash
cargo fmt --all
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo check
```

Verify:
- All 25 new tests in layer_builder_tests.rs passing
- All existing tests passing (backward compatibility)
- Zero clippy warnings
- Code formatted
- Implicit layering behavior unchanged
- No modifications to existing test fixtures

**Tests**: All 25 new + all existing tests passing

**Acceptance**:
- [ ] `cargo test --all` passes
  - [ ] All 25 layer_builder_tests passing
  - [ ] All existing tests passing
- [ ] `cargo fmt --all` passes
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` (0 warnings)
- [ ] `cargo check` passes
- [ ] Implicit layering unchanged
- [ ] Ready for code review (Gate 2)

**Blocked by**: sl-uda  
**Status**: Ready for Gate 2

---

## Success Criteria

**Definition of Done**:
- ✅ All 25 tests in `tests/layer_builder_tests.rs` passing
- ✅ All existing tests still passing (backward compatibility)
- ✅ 0 clippy warnings
- ✅ Code formatted with `cargo fmt`
- ✅ Documentation complete with examples
- ✅ ConfigLayer and LayerBuilder exported from lib.rs
- ✅ LoadingOptions::build_layers() trait method added with default impl
- ✅ SettingsLoader::load() modified with explicit layer support and fallback

**Code Quality Gates**:
- [ ] Zero clippy warnings
- [ ] All tests passing
- [ ] Code formatted
- [ ] No unsafe code
- [ ] Documentation complete
- [ ] Backward compatible

---

## Implementation Notes

1. **Don't reinvent**: Config crate already handles format detection via file extension - use it
2. **Path handling**: Use existing `path_absolutize` crate for consistent absolutization
3. **Graceful degradation**: EnvVar not set = skip layer (don't error)
4. **File validation**: Path and Secrets files must exist (required=true)
5. **Layer precedence**: Config crate's `.add_source()` order determines precedence - later adds override
6. **No new dependencies**: Phase 1 uses only existing dependencies
7. **Backward compatible**: Apps not implementing `build_layers()` continue to work unchanged
8. **TDD discipline**: Implement features to make tests pass, not the other way around

---

## File Structure After Completion

```
src/
  layer.rs                # NEW: ConfigLayer, LayerBuilder
  lib.rs                  # MODIFIED: Export types, add build_layers()
  settings_loader.rs      # MODIFIED: Explicit layers + fallback
  environment.rs          # UNCHANGED
  error.rs                # UNCHANGED
  internals/              # UNCHANGED
  common/                 # UNCHANGED

tests/
  layer_builder_tests.rs  # Tests already exist (25 tests)
  
history/
  CONSOLIDATED_ROADMAP.md
  DESIGN.md
  IMPLEMENTATION_PLAN.md
  TEST_PLAN_SUMMARY.md
  MIGRATION_GUIDE.md
  sl-h8h_phase1_tdd_breakdown.md
```

---

## Progress Tracking

Track completion as each subtask finishes:

- [ ] PHASE1.1 (sl-bru) - Core Types [RED]
- [ ] PHASE1.2 (sl-bv4) - Builder Methods [GREEN]
- [ ] PHASE1.3 (sl-nz1) - Build Logic [GREEN]
- [ ] PHASE1.4 (sl-5h9) - Format Detection [GREEN]
- [ ] PHASE1.5 (sl-apw) - SettingsLoader Integration [GREEN]
- [ ] PHASE1.6 (sl-mvm) - LoadingOptions Trait [GREEN]
- [ ] PHASE1.7 (sl-uda) - Documentation [GREEN]
- [ ] PHASE1.8 (sl-6pk) - Validation [REFACTOR] → Ready for Gate 2

---

## Next Steps

1. Approve implementation plan
2. Create feature branch: `feat/phase1-explicit-layering`
3. Start PHASE1.1: Create `src/layer.rs` with ConfigLayer + LayerBuilder types
4. Run `cargo test layer_builder_tests` → Tests 1-6 fail (RED phase)
5. Continue through subtasks in dependency order
6. Run comprehensive validation in PHASE1.8
7. Request code review approval (Gate 2)

---

## Related Documents

- `history/CONSOLIDATED_ROADMAP.md` - Full 7-phase roadmap (v0.16.0 through v1.0.0)
- `history/DESIGN.md` - Phase 1 detailed design specification
- `history/sl-h8h_phase1_tdd_breakdown.md` - TDD task breakdown with test mapping
- `tests/layer_builder_tests.rs` - 25 pre-written tests (RED phase ready)
- `history/MIGRATION_GUIDE.md` - Migration guide for users of Phase 1 API
- `ref/architecture-proposal.md` - Original architecture vision
- `ref/turtle-consolidation.md` - Turtle use case analysis
