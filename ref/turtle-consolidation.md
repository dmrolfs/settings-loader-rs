# Consolidation from spark-turtle → settings-loader

This document details specific components from `spark-turtle` that should be consolidated back into `settings-loader-rs` as first-class features.

---

## Overview

During the development of `spark-turtle`'s TUI settings editor, several capabilities were needed that `settings-loader-rs` doesn't provide. Rather than maintaining these in application code, they represent general-purpose functionality that belongs in the configuration library.

## Components to Consolidate

## Important: Wrapping, Not Replacing

The consolidation strategy wraps the config crate rather than replacing it. This preserves:
- Serde deserialization capability
- Multi-source composition
- Format support (YAML, JSON, TOML, etc.)

Turtle's implementations (ConfigEditor, SettingsRegistry, etc.) become first-class 
features in settings-loader, all layered on top of config crate's proven merging logic.

### 1. ConfigEditor (Multi-Format Editing)

**Source**: `spark-turtle/crates/turtle-core/src/config/editor.rs`

**Integration Point**: Becomes `LayerEditor` in settings-loader

Instead of a standalone tool, becomes part of the layering system:
```rust
pub struct LayerEditor {
    scope: ConfigScope,      // Know which scope we're editing
    path: PathBuf,
    backend: EditorBackend,  // toml_edit, json, yaml
}
```

**Benefits**:
- Works with multi-scope system
- Integrates with source provenance (knows source origin)
- Standardized across projects using settings-loader
- Comment preservation built-in for TOML

**Turtle API Before**:
```rust
let editor = ConfigEditor::load("turtle.toml")?;
editor.set_string("llm.ollama.model", "codellama")?;
editor.save()?;
```

**Turtle API After** (using settings-loader):
```rust
let mut editor = LayerEditor::for_scope(ConfigScope::ProjectLocal)?;
editor.set("llm.ollama.model", "codellama")?;
editor.save()?;
```

**APIs to Extract**:
- `load(path)` - Auto-detect format, load for editing
- `save()` - Write back with format preservation
- `get_string/bool/int(key)` - Typed getters with dot-notation keys
- `set_string/bool/int(key, value)` - Typed setters

**Benefits to settings-loader**:
- Completes the bidirectional story (currently read-only)
- TOML comment preservation is a unique selling point
- Format detection already handles extension-based dispatch

**Integration Point**:
```rust
// Proposed new API on SettingsLoader
pub trait SettingsEditor: SettingsLoader {
    fn editor_for_scope(scope: ConfigScope) -> Result<Self::Editor>;
}
```

---

### 2. SettingsMetadata Registry

**Source**: `spark-turtle/crates/turtle-core/src/config/metadata.rs`

**Integration Point**: Becomes `ConfigSchema` trait in settings-loader (Phase 5)

```rust
pub trait SettingsIntrospection {
    fn schema(&self) -> ConfigSchema;
}

pub struct ConfigSchema {
    pub name: String,
    pub settings: Vec<SettingMetadata>,
    pub groups: Vec<SettingGroup>,
}

pub struct SettingMetadata {
    pub key: String,
    pub label: String,
    pub description: String,
    pub setting_type: SettingType,
    pub default: Option<serde_json::Value>,
    pub constraints: Vec<Constraint>,
    pub visibility: Visibility,
}
```

**Benefits**:
- Rich type information (Integer, Float, Enum, etc.)
- Validation constraints
- Visibility levels (public, secret, advanced, hidden)
- Group organization
- Optional proc-macro for derivation

**Turtle API Before**:
```rust
for meta in SettingsRegistry::all() {
    println!("{}: {}", meta.key, meta.description);
}
```

**Turtle API After** (using settings-loader):
```rust
for meta in TurtleConfig::schema().settings {
    println!("{}: {} (type: {:?})", 
        meta.key, meta.description, meta.setting_type);
}
```

**Key Features**:
- Static registry of all known settings
- Conditional visibility (`is_relevant` callbacks)
- Runtime introspection for TUI/CLI

**Extension for settings-loader**:
```rust
// Add type information and validation
pub struct SettingMetadata {
    pub key: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub setting_type: SettingType,
    pub default_value: Option<&'static str>,
    pub is_required: bool,
    pub is_secret: bool,
    pub group: &'static str,
    pub is_relevant: fn(&dyn std::any::Any) -> bool,
}

pub enum SettingType {
    String,
    Integer { min: Option<i64>, max: Option<i64> },
    Float { min: Option<f64>, max: Option<f64> },
    Boolean,
    Duration,
    Path,
    Url,
    Enum { variants: &'static [&'static str] },
}
```

**Proc Macro Consideration**:
```rust
// Could generate metadata from struct + attributes
#[derive(Settings, Serialize, Deserialize)]
pub struct LlmConfig {
    #[setting(
        key = "provider",
        description = "LLM provider to use",
        variants = ["ollama", "anthropic", "openai"]
    )]
    pub provider: LlmProvider,
    
    #[setting(nested)]
    pub ollama: OllamaConfig,
}
```

---

### 3. Multi-Scope Path Resolution

**Integration Point**: Becomes `MultiScopeLoader` in settings-loader (Phase 2)

Automatically discovers and merges configuration from all scopes:

```rust
pub enum ConfigScope {
    System,           // System defaults
    UserGlobal,       // ~/.config/app/
    ProjectLocal,     // ./settings.toml
    Runtime,          // APP_* env vars
}

pub trait MultiScopeLoader: DeserializeOwned {
    const APP_NAME: &'static str;
    
    fn load_multi_scope() -> Result<(Self, SourceMap)> {
        // Automatically discovers and merges from all scopes
        // SourceMap tracks which scope each value came from
    }
}
```

**Benefits**:
- Automatic scope path resolution (uses `directories` crate)
- Single call to load all scopes
- Source tracking (know which scope each setting came from)
- Consistent across projects

**Turtle API Before**:
```rust
fn init_editors(&mut self, project_path: Option<&Path>) {
    // Manual path management, error handling for each scope
}
```

**Turtle API After** (using settings-loader):
```rust
let (config, sources) = TurtleConfig::load_multi_scope()?;

// Query where a value came from
if let Some(meta) = sources.source_of("llm.provider") {
    println!("LLM provider set in {:?}", meta.scope);
}
```

---

### 4. Environment Variable Format Customization

**Current turtle implementation**:
- Hardcoded `TURTLE__` prefix
- Hardcoded `__` separator
- No customization

**Integration Point**: Trait method in settings-loader

```rust
pub trait LoadingOptions {
    fn env_prefix() -> &'static str { "APP" }
    fn env_separator() -> &'static str { "__" }
}
```

**Turtle API After**:
```rust
impl LoadingOptions for TurtleOptions {
    fn env_prefix() -> &'static str { "TURTLE" }
    // Returns "TURTLE__LLM__OLLAMA__BASE_URL"
}
```

---

### 5. Source Provenance Tracking

**Current turtle limitation**:
- No way to tell if setting came from environment or file
- "Effective" config loses source information
- Users can't understand where values came from

**Integration Point**: `SourceMap` in settings-loader (Phase 0)

```rust
pub struct SourceMap {
    entries: HashMap<String, (SourceMetadata, Value)>,
}

pub struct SourceMetadata {
    pub source_type: SourceType,  // File, Environment, Default
    pub path: Option<PathBuf>,
    pub scope: Option<ConfigScope>,  // Which scope?
}
```

**Benefits**:
- Track which file provided each setting
- Track which environment variable
- Track which scope (UserGlobal vs ProjectLocal)
- Enable "show origin" feature in TUI

**Usage**:

---

## Implementation Effort Estimate

| Component | Complexity | Dependencies | Priority |
|-----------|------------|--------------|----------|
| ConfigEditor | Medium | `toml_edit` | High |
| Env Var Customization | Low | None | High |
| SettingsMetadata | Medium | Optional proc-macro | Medium |
| Multi-Scope Paths | Low | `directories` | Medium |
| Source Provenance | High | Refactor core loading | Low |

---

## Feature Flag Organization

```toml
[features]
default = []

# Bidirectional config editing
editor = ["toml_edit"]

# Settings introspection/schema
metadata = []

# Derive macro for metadata generation  
metadata-derive = ["metadata", "settings-loader-derive"]

# Multi-scope path resolution
multi-scope = ["directories"]

# Source tracking
provenance = []

# All features
full = ["editor", "metadata", "multi-scope", "provenance"]
```

---

## API Surface Changes

### Before (0.15.0)
```rust
// Load-only API
let settings = MySettings::load(&options)?;
```

### After (1.0.0 proposed)
```rust
// Load with provenance (opt-in via feature)
let loaded = MySettings::load_with_sources(&options)?;
println!("api_url from: {:?}", loaded.sources.get("api_url"));

// Edit and save (opt-in via feature)
let mut editor = MySettings::editor(ConfigScope::ProjectLocal)?;
editor.set("timeout", 60)?;
editor.save()?;

// Introspection (opt-in via feature)
for meta in MySettings::metadata() {
    println!("{}: {}", meta.key, meta.description);
}
```

---

## Testing Strategy

### Migrate turtle Tests
- `spark-turtle` test fixtures → settings-loader test resources
- Integration tests for edit → reload cycle
- Property tests for round-trip (load → modify → save → reload)

### New Test Cases
- Comment preservation in TOML files
- Multi-format detection edge cases
- Scope precedence verification
- Source tracking accuracy

---

## Implementation Roadmap

```
Week 1-2: ConfigEditor extraction
  - Copy turtle's editor.rs to settings-loader
  - Add feature flag
  - Write integration tests
  
Week 3: Env Var Customization
  - Add trait methods to LoadingOptions
  - Update internal env var handling
  - Backward compatible defaults

Week 4-5: Multi-Scope Paths
  - Add ConfigScope enum
  - Implement path resolution
  - Add directories dependency

Week 6-7: Metadata System
  - Define SettingMetadata types
  - Optional proc-macro for derivation
  - Schema export (JSON Schema compatible)

Week 8+: Source Provenance
  - Refactor loading pipeline
  - Track sources during merge
  - SourceMap API design
```
