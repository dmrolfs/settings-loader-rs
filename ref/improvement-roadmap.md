# settings-loader-rs Improvement Roadmap

This document outlines strategic improvements for `settings-loader-rs` based on real-world usage in applications like `spark-turtle` and analysis of current capabilities vs. modern configuration library expectations.

## Executive Summary

The current `settings-loader-rs` excels at **loading** configuration from multiple sources but lacks:
1. **Source provenance tracking** - Know which layer provided each value
2. **Configuration editing/writing** - Edit individual layers, preserve comments
3. **Explicit layering** - Organize sources into named layers
4. **Multi-scope management** - User-global vs. project-local distinction
5. **Type-aware metadata** - For validation and UI rendering

These gaps forced `spark-turtle` to implement parallel systems (`ConfigEditor`, `SettingsRegistry`) that could be consolidated into the loader.

---

## Current Architecture Analysis

### Strengths
- Clean trait-based design (`SettingsLoader`, `LoadingOptions`)
- Multi-format support via `config` crate (JSON, TOML, YAML, HJSON, RON)
- Hierarchical merging with correct precedence
- Environment-specific configuration overlays
- Secrets file separation
- Type-safe deserialization

### Gaps Identified

| Gap | Impact | Workaround in turtle |
|-----|--------|---------------------|
| No write capability | Cannot save settings from TUI | Built `ConfigEditor` |
| No introspection | Cannot enumerate available settings | Built `SettingsRegistry` |
| Single config scope | No user-global vs project-local | Manual path management |
| Non-customizable env var format | Forces `APP__` prefix | Hardcoded `TURTLE__` |
| No comment preservation | Editing loses TOML comments | Custom `toml_edit` integration |
| No validation metadata | No default values or descriptions exposed | Hardcoded in registry |

---

## Proposed Improvements

## Foundation: Why Wrap Config Crate?

Settings-loader proposes wrapping the config crate rather than replacing it because:

1. **Irreplaceable serde integration**
   - config crate implements Deserializer trait
   - Enables direct deserialization: `config.try_deserialize::<T>()?`
   - This capability is worth preserving

2. **Proven merging logic**
   - config crate's source composition and precedence rules are battle-tested
   - Merging complexity shouldn't be re-implemented

3. **Multi-format support**
   - JSON, YAML, TOML, HJSON, RON via plugins
   - No need to re-implement format support

4. **Non-invasive provenance**
   - Provenance tracking happens in parallel
   - Doesn't modify config's merge algorithm
   - Completely backward compatible

**Result**: Add new layers on top rather than replacing bottom layer.

## Phase 0: Source Provenance (NEW)

**NEW PHASE - Insert as foundational work**

Enable tracking of which source provided each configuration value.

```rust
pub struct SourceMetadata {
    pub id: String,            // "defaults", "file:config.yml", "env:APP_"
    pub source_type: SourceType,
    pub path: Option<PathBuf>,
    pub scope: Option<ConfigScope>,
}

pub struct SourceMap {
    entries: HashMap<String, (SourceMetadata, Value)>,
}

pub fn load_with_provenance<T: DeserializeOwned>(
    sources: Vec<ConfigSource>,
) -> Result<(T, SourceMap)>;
```

**Why this phase is foundational**:
- Enables layer-scoped editing (Phase 1)
- Prerequisites multi-scope (Phase 3)
- Enables source visualization (UX feature)

**Benefits**:
- Users can see where each setting came from
- Applications can track configuration origins
- Prerequisite for multi-scope path resolution
- Enables intelligent editing at layer level

### Phase 1: Explicit Layering (formerly "Configuration Writing")

Provide explicit control over configuration source composition.

**Before** (implicit, hardcoded):
```rust
// Old code - layering is implicit, ordering is hidden
let config = load_from_default_location()?;
```

**After** (explicit, controlled):
```rust
let mut builder = LayerBuilder::new();
builder
    .with_defaults(metadata_defaults)
    .with_path("/etc/app/settings.toml")
    .with_env_vars("APP_", "__")
    .with_path("./settings.local.toml");
    
let (settings, sources) = builder.build::<AppSettings>()?;

// Now you know exactly what merged into what, in what order
```

**Uses Foundation (Phase 0)**:
- Each layer wrapped with SourceMetadata
- SourceMap tracks which layer provided each value
- Explicit over implicit - programmer controls precedence

**New Trait**:
```rust
pub struct LayerBuilder {
    layers: Vec<(LayerName, ConfigSource)>,
}

impl LayerBuilder {
    pub fn with_defaults(defaults: Map<String, Value>) -> Self;
    pub fn with_path(path: PathBuf) -> Self;
    pub fn with_env_vars(prefix: &str, separator: &str) -> Self;
    pub fn build<T: DeserializeOwned>(self) -> Result<(T, SourceMap)>;
}

// Environment variable customization (optional)
pub trait LoadingOptions {
    fn env_prefix() -> &'static str { "APP" }
    fn env_separator() -> &'static str { "__" }
}
```

```rust
// New trait for bidirectional config
pub trait SettingsEditor: SettingsLoader {
    type Editor: ConfigEditor;
    
    fn editor_for(scope: ConfigScope) -> Result<Self::Editor>;
    fn save(&self, scope: ConfigScope) -> Result<()>;
}

pub enum ConfigScope {
    UserGlobal,    // ~/.config/app/settings.toml
    ProjectLocal,  // ./app.toml
    Explicit(PathBuf),
}

pub trait ConfigEditor {
    fn load(path: impl Into<PathBuf>) -> Result<Self>;
    fn save(&self) -> Result<()>;
    fn get<T: FromStr>(&self, key: &str) -> Option<T>;
    fn set<T: ToString>(&mut self, key: &str, value: T) -> Result<()>;
}
```

**Features:**
- Format-specific backends (TOML with comment preservation, JSON, YAML)
- Automatic format detection by extension
- Create parent directories on save
- Atomic writes (write to temp, then rename)

### Phase 3: Settings Metadata Registry

Add compile-time or runtime metadata for settings introspection.

```rust
pub struct SettingMetadata {
    pub key: &'static str,           // "llm.ollama.base_url"
    pub description: &'static str,   // Human-readable
    pub default_value: &'static str, // Serialized default
    pub setting_type: SettingType,   // String, Int, Float, Bool, Enum
    pub is_secret: bool,
    pub is_relevant: fn(&Config) -> bool, // Conditional visibility
}

pub enum SettingType {
    String,
    Integer,
    Float,
    Boolean,
    Enum(Vec<&'static str>),
    Path,
    Url,
}

pub trait HasMetadata: SettingsLoader {
    fn metadata() -> &'static [SettingMetadata];
    fn available_settings(config: &Self) -> Vec<&'static SettingMetadata>;
}
```

**Use Cases:**
- TUI settings editors (show available settings with descriptions)
- CLI `--help` generation for settings
- Validation with meaningful error messages
- Schema generation for documentation

### Phase 2: Multi-Scope Configuration (formerly Phase 3)

Automatically discover and merge configuration from multiple scopes.

**Uses Foundation (Phase 0)**:
- SourceMap tracks which scope each value came from
- Path resolution per scope
- Automatic precedence: System → UserGlobal → ProjectLocal → Runtime

**New Trait**:
```rust
pub enum ConfigScope {
    System,           // /etc/app/settings.toml
    UserGlobal,       // ~/.config/app/settings.toml
    ProjectLocal,     // ./settings.toml
    Runtime,          // APP_* environment variables
}

pub trait MultiScopeLoader: DeserializeOwned {
    const APP_NAME: &'static str;
    
    fn load_multi_scope() -> Result<(Self, SourceMap)>;
}
```

**How SourceMap helps**:
```rust
let (settings, sources) = AppSettings::load_multi_scope()?;

match sources.source_of("database.host") {
    Some(meta) if meta.scope == Some(ConfigScope::UserGlobal) =>
        println!("User overrode db host"),
    Some(meta) if meta.scope == Some(ConfigScope::ProjectLocal) =>
        println!("Project specified db host"),
    _ => println!("Using system default"),
}
```

```rust
pub trait MultiScopeSettings: SettingsLoader {
    fn user_global_path() -> Option<PathBuf>;
    fn project_local_path() -> Option<PathBuf>;
    
    fn load_effective(options: &Self::Options) -> Result<Self>;
    fn load_scope(scope: ConfigScope) -> Result<PartialConfig>;
    
    fn setting_source(key: &str) -> SettingSource;
}

// Replaced by Phase 2 MultiScopeLoader and Phase 0 SourceMap
```

**Benefits:**
- Users can set personal defaults globally
- Projects can override for team consistency
- UI can show "source" of each setting
- Clear precedence visualization

### Phase 4: Customizable Environment Variable Format

Moved to Phase 1 (Explicit Layering) as trait methods on `LoadingOptions`.

### Phase 4: Configuration Editing

Enable reading and writing of configuration files while preserving structure and comments.

**Uses Foundation (Phase 0)**:
- SourceMap tracks which file belongs to which scope
- Edit only the target scope, don't touch other scopes

**New Type**:
```rust
pub struct LayerEditor {
    scope: ConfigScope,
    path: PathBuf,
    backend: EditorBackend,  // toml_edit, json, yaml
}

impl LayerEditor {
    pub fn for_scope(scope: ConfigScope) -> Result<Self>;
    pub fn get<T: FromStr>(&self, key: &str) -> Result<T>;
    pub fn set<T: ToString>(&mut self, key: &str, value: T) -> Result<()>;
    pub fn save(&self) -> Result<()>;  // Preserves comments
}
```

**Example**:
```rust
// Edit only project-local settings
let mut editor = LayerEditor::for_scope(ConfigScope::ProjectLocal)?;
editor.set("database.host", "prod.db.example.com")?;
editor.save()?;  // Saves to ./settings.toml, comments preserved

// User-global and system defaults untouched
```

**Format Support**:
- **TOML**: Full comment preservation via toml_edit
- **JSON**: Standard JSON editing via serde_json
- **YAML**: Limited comment support via serde_yaml

**Auto-detection**: Format detected from file extension

```rust
#[derive(Debug, Error)]
pub enum SettingsError {
    // Existing variants...
    
    #[error("setting '{key}' not found. Available: {available:?}")]
    UnknownSetting { key: String, available: Vec<String> },
    
    #[error("invalid value for '{key}': expected {expected_type}, got '{actual}'")]
    TypeMismatch { key: String, expected_type: String, actual: String },
    
    #[error("setting '{key}' requires value when '{dependent_key}' is set")]
    DependencyMissing { key: String, dependent_key: String },
    
    #[error("failed to parse {format} config at {path}: {source}")]
    ParseError { format: String, path: PathBuf, source: Box<dyn std::error::Error> },
}
```

---

## Integration with spark-turtle

### Components to Consolidate

| turtle Component | → settings-loader Feature |
|-----------------|---------------------------|
| `ConfigEditor` | Phase 0 + Phase 4: LayerEditor (wraps config) |
| `SettingsRegistry` | Phase 5: ConfigSchema (optional) |
| User/Project path logic | Phase 2: MultiScopeLoader |
| `TURTLE__` env vars | Phase 1: LoadingOptions trait method |

### Migration Path

1. **Immediate**: Use turtle's implementation as reference
2. **Short-term**: Add Phase 1 (editing) to settings-loader
3. **Medium-term**: Add Phase 2 (metadata) with proc macro support
4. **Long-term**: Full multi-scope with source tracking

### API Changes for turtle

Before (current):
```rust
// turtle-core/src/config/editor.rs
let editor = ConfigEditor::load("turtle.toml")?;
editor.set_string("llm.ollama.model", "codellama")?;
editor.save()?;

// turtle-tui/src/settings.rs
for meta in SettingsRegistry::all() { ... }
```

After (with improved settings-loader):
```rust
// Using enhanced settings-loader
let editor = TurtleConfig::editor_for(ConfigScope::ProjectLocal)?;
editor.set("llm.ollama.model", "codellama")?;
editor.save()?;

for meta in TurtleConfig::metadata() { ... }
```

---

## Implementation Priority

1. **Phase 1: Configuration Writing** - Highest impact, enables TUI editing
2. **Phase 4: Custom Env Format** - Low effort, high usability
3. **Phase 2: Settings Metadata** - Enables advanced UIs
4. **Phase 3: Multi-Scope** - Full configuration management
5. **Phase 5: Enhanced Errors** - Quality of life

---

## Open Questions

1. **Proc Macro for Metadata?**
   - `#[setting(key = "llm.ollama.model", description = "...", default = "...")]`
   - Tradeoffs: compile-time vs runtime, dependency weight

2. **Comment Preservation Scope?**
   - Full comment preservation requires format-specific editors
   - TOML: `toml_edit`, JSON: none, YAML: limited

3. **Breaking Changes?**
   - New traits could be additive (non-breaking)
   - Enum variants are already `#[non_exhaustive]`

4. **Feature Flags?**
   - Keep editing optional (`editor` feature)?
   - Metadata could be opt-in for smaller binaries
