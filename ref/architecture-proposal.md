# Architectural Improvements for settings-loader-rs

This document proposes architectural changes to modernize `settings-loader-rs` based on patterns observed in `spark-turtle` and contemporary configuration library design.

---

## Current Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      SettingsLoader Trait                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ config crate │  │ LoadingOpts  │  │ Serde Deserialize    │  │
│  │ (read-only)  │  │ (paths/env)  │  │ (type conversion)    │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Application Settings Struct                   │
│                    (fully materialized, typed)                   │
└─────────────────────────────────────────────────────────────────┘
```

### Limitations

1. **One-way data flow**: Load only, no write-back
2. **Single output**: One merged struct, no per-scope access
3. **Opaque merging**: Cannot track which source provided which value
4. **No runtime introspection**: Keys/types not discoverable

---

## Proposed Architecture

### Core Design: Layered Configuration Stack

```
┌─────────────────────────────────────────────────────────────────┐
│                     ConfigurationStack                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌──────────┐  │
│  │   Default   │ │ User Global │ │ Project     │ │ Runtime  │  │
│  │   Layer     │ │ Layer       │ │ Local Layer │ │ Overrides│  │
│  │             │ │ ~/.config/  │ │ ./app.toml  │ │ Env+CLI  │  │
│  └─────────────┘ └─────────────┘ └─────────────┘ └──────────┘  │
│         │               │              │               │        │
│         └───────────────┴──────────────┴───────────────┘        │
│                            │                                     │
│                    ┌───────▼────────┐                           │
│                    │ Merge Engine   │                           │
│                    │ (with provenance)                          │
│                    └───────┬────────┘                           │
│                            │                                     │
│              ┌─────────────┴─────────────┐                      │
│              ▼                           ▼                      │
│     ┌────────────────┐          ┌────────────────┐             │
│     │ Effective      │          │ Source Map     │             │
│     │ Configuration  │          │ (key → origin) │             │
│     └────────────────┘          └────────────────┘             │
└─────────────────────────────────────────────────────────────────┘
```

### New Trait Hierarchy

```rust
// Base trait (unchanged external API)
pub trait SettingsLoader: Sized + DeserializeOwned {
    type Options: LoadingOptions;
    fn load(options: &Self::Options) -> Result<Self, Self::Options::Error>;
}

// NEW: Bidirectional editing
pub trait SettingsEditor: SettingsLoader {
    type Editor: LayerEditor;
    
    fn editor(scope: ConfigScope, options: &Self::Options) -> Result<Self::Editor>;
    fn commit(editor: Self::Editor) -> Result<()>;
}

// NEW: Introspection for UIs
pub trait SettingsIntrospection: SettingsLoader {
    fn schema() -> ConfigSchema;
    fn metadata() -> &'static [SettingMetadata];
}

// NEW: Source tracking
pub trait SettingsProvenance: SettingsLoader {
    fn load_with_provenance(options: &Self::Options) -> Result<(Self, SourceMap)>;
}
```

---

## Component Deep Dives

### 1. Layer Editor

Enables editing individual configuration layers without affecting others.

```rust
pub trait LayerEditor: Send + Sync {
    /// Get a value from this layer only
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    
    /// Set a value in this layer
    fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<()>;
    
    /// Remove a value from this layer (falls through to lower layers)
    fn unset(&mut self, key: &str) -> Result<()>;
    
    /// List keys modified in this layer
    fn keys(&self) -> Vec<String>;
    
    /// Persist changes
    fn save(&self) -> Result<()>;
    
    /// Check if layer has unsaved changes
    fn is_dirty(&self) -> bool;
}
```

#### Format-Specific Implementations

```rust
pub enum EditorBackend {
    /// Uses toml_edit for comment preservation
    Toml(TomlEditor),
    /// Standard serde_json (no comment preservation)
    Json(JsonEditor),
    /// serde_yaml (limited comment preservation)  
    Yaml(YamlEditor),
}

impl EditorBackend {
    pub fn from_path(path: &Path) -> Result<Self> {
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => Ok(Self::Toml(TomlEditor::load(path)?)),
            Some("json") => Ok(Self::Json(JsonEditor::load(path)?)),
            Some("yaml" | "yml") => Ok(Self::Yaml(YamlEditor::load(path)?)),
            _ => Err(EditorError::UnsupportedFormat),
        }
    }
}
```

### 2. Configuration Schema

Runtime-accessible schema for validation and UI generation.

```rust
#[derive(Debug, Clone)]
pub struct ConfigSchema {
    pub settings: Vec<SettingMetadata>,
    pub groups: Vec<SettingGroup>,
}

#[derive(Debug, Clone)]
pub struct SettingMetadata {
    pub key: String,
    pub label: String,
    pub description: String,
    pub setting_type: SettingType,
    pub default: Option<serde_json::Value>,
    pub constraints: Vec<Constraint>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub enum SettingType {
    String { pattern: Option<String> },
    Integer { min: Option<i64>, max: Option<i64> },
    Float { min: Option<f64>, max: Option<f64> },
    Boolean,
    Enum { variants: Vec<EnumVariant> },
    Array { item_type: Box<SettingType> },
    Duration,
    Path { must_exist: bool },
    Url { schemes: Vec<String> },
    Secret,  // Masked in UIs
}

#[derive(Debug, Clone)]
pub struct SettingGroup {
    pub name: String,
    pub description: String,
    pub settings: Vec<String>,  // Keys in this group
}
```

### 3. Source Map / Provenance

Track where each setting value originated.

```rust
#[derive(Debug, Clone)]
pub struct SourceMap {
    sources: HashMap<String, SettingSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingSource {
    Default,
    File { path: PathBuf, scope: ConfigScope },
    Environment { var_name: String },
    CliOverride,
    Computed,  // Derived from other settings
}

impl SourceMap {
    pub fn source_of(&self, key: &str) -> Option<&SettingSource>;
    pub fn all_from_scope(&self, scope: ConfigScope) -> Vec<&str>;
    pub fn overridden_keys(&self) -> Vec<(&str, &SettingSource)>;
}
```

### 4. Multi-Scope Configuration

Standard patterns for user vs. project configuration.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigScope {
    /// System-wide defaults (read-only typically)
    System,
    /// User's global preferences (~/.config/app/)
    UserGlobal,
    /// Project-specific settings (./app.toml)
    ProjectLocal,
    /// Runtime overrides (env vars, CLI)
    Runtime,
}

/// Configuration for multi-scope resolution
pub trait MultiScopeConfig: Sized {
    /// Application identifier for path resolution
    const APP_NAME: &'static str;
    
    /// Default filename (e.g., "settings.toml")
    const CONFIG_FILENAME: &'static str = "settings.toml";
    
    /// Get platform-appropriate path for scope
    fn path_for_scope(scope: ConfigScope) -> Option<PathBuf> {
        match scope {
            ConfigScope::System => dirs::config_dir()
                .map(|d| d.join(Self::APP_NAME).join(Self::CONFIG_FILENAME)),
            ConfigScope::UserGlobal => dirs::config_dir()
                .map(|d| d.join(Self::APP_NAME).join(Self::CONFIG_FILENAME)),
            ConfigScope::ProjectLocal => Some(PathBuf::from(Self::CONFIG_FILENAME)),
            ConfigScope::Runtime => None,
        }
    }
}
```

---

## Migration Strategy

### Backward Compatibility

All new traits are **additive**. Existing code continues to work:

```rust
// This still works exactly as before
let settings = MySettings::load(&options)?;
```

### Opt-in Features

```toml
[features]
default = []
editor = ["toml_edit"]
schema = []
provenance = []
multi-scope = ["directories"]
full = ["editor", "schema", "provenance", "multi-scope"]
```

### Implementation Order

1. **Add `editor` feature** with `LayerEditor` trait
2. **Add `provenance` feature** with source tracking
3. **Add `schema` feature** with metadata types
4. **Add `multi-scope` feature** with standard paths
5. **Add proc macro** for compile-time schema generation

---

## Example: Integrated Usage

```rust
use settings_loader::{
    SettingsLoader, SettingsEditor, SettingsIntrospection,
    ConfigScope, SettingSource, SourceMap
};

#[derive(Debug, Deserialize, SettingsSchema)]
#[settings(app = "my-app")]
pub struct AppSettings {
    #[setting(
        description = "API endpoint URL",
        default = "http://localhost:8080"
    )]
    pub api_url: String,
    
    #[setting(
        description = "Request timeout in seconds",
        default = 30,
        min = 1, max = 300
    )]
    pub timeout_secs: u64,
    
    #[setting(secret)]
    pub api_key: Option<String>,
}

// Loading with provenance
let (settings, sources) = AppSettings::load_with_provenance(&options)?;

// Check where api_url came from
match sources.source_of("api_url") {
    Some(SettingSource::Environment { var_name }) => 
        println!("api_url from env: {}", var_name),
    Some(SettingSource::File { path, scope }) =>
        println!("api_url from {:?}: {}", scope, path.display()),
    _ => println!("api_url from default"),
}

// Edit project-local settings
let mut editor = AppSettings::editor(ConfigScope::ProjectLocal, &options)?;
editor.set("timeout_secs", 60)?;
editor.save()?;

// Generate UI from schema
for meta in AppSettings::metadata() {
    println!("{}: {} (default: {:?})", 
        meta.key, meta.description, meta.default);
}
```

---

## Comparison with Alternatives

| Feature | settings-loader (proposed) | config-rs | figment |
|---------|---------------------------|-----------|---------|
| Multi-format | ✅ | ✅ | ✅ |
| Env overlay | ✅ | ✅ | ✅ |
| Config writing | ✅ | ❌ | ❌ |
| Comment preservation | ✅ (TOML) | ❌ | ❌ |
| Source tracking | ✅ | ❌ | ✅ |
| Schema/metadata | ✅ | ❌ | ❌ |
| Multi-scope | ✅ | ❌ | ❌ |
| TUI integration | ✅ | ❌ | ❌ |

---

## Next Steps

1. **RFC**: Share this document for feedback
2. **Prototype**: Implement Phase 1 (editor) in a branch
3. **Migrate turtle**: Use turtle as test case for API design
4. **Stabilize**: Iterate based on usage
5. **Release**: Version 1.0 with stable trait hierarchy
