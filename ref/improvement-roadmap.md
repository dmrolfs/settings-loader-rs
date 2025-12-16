# settings-loader-rs Improvement Roadmap

This document outlines strategic improvements for `settings-loader-rs` based on real-world usage in applications like `spark-turtle` and analysis of current capabilities vs. modern configuration library expectations.

## Executive Summary

The current `settings-loader-rs` excels at **loading** configuration from multiple sources but lacks:
1. **Configuration editing/writing** capabilities
2. **Settings introspection** for UI integration
3. **Multi-scope management** (user-global vs. project-local)
4. **Type-aware metadata** for validation and UI rendering

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

### Phase 1: Configuration Writing (Core)

Add bidirectional configuration support - read AND write.

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

### Phase 2: Settings Metadata Registry

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

### Phase 3: Multi-Scope Configuration

First-class support for user-global and project-local configurations.

```rust
pub trait MultiScopeSettings: SettingsLoader {
    fn user_global_path() -> Option<PathBuf>;
    fn project_local_path() -> Option<PathBuf>;
    
    fn load_effective(options: &Self::Options) -> Result<Self>;
    fn load_scope(scope: ConfigScope) -> Result<PartialConfig>;
    
    fn setting_source(key: &str) -> SettingSource;
}

pub enum SettingSource {
    Default,
    UserGlobal,
    ProjectLocal,
    Environment,
    CliOverride,
}
```

**Benefits:**
- Users can set personal defaults globally
- Projects can override for team consistency
- UI can show "source" of each setting
- Clear precedence visualization

### Phase 4: Customizable Environment Variable Format

Allow applications to customize env var naming conventions.

```rust
pub trait LoadingOptions: Sized {
    // Existing...
    
    /// Customize env var prefix (default: "APP")
    fn env_prefix() -> &'static str { "APP" }
    
    /// Customize separator (default: "__")
    fn env_separator() -> &'static str { "__" }
    
    /// Custom key transformation
    fn env_key_transform(key: &str) -> String {
        format!("{}_{}", Self::env_prefix(), key.to_uppercase().replace('.', Self::env_separator()))
    }
}
```

**Example:**
- `spark-turtle` uses `TURTLE__LLM__OLLAMA__BASE_URL`
- Default would be `APP__LLM__OLLAMA__BASE_URL`

### Phase 5: Enhanced Error Types

Improve error messages for configuration issues.

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
| `ConfigEditor` | Phase 1: Configuration Writing |
| `SettingsRegistry` | Phase 2: Settings Metadata |
| User/Project path logic | Phase 3: Multi-Scope |
| `TURTLE__` env vars | Phase 4: Custom Env Format |

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
