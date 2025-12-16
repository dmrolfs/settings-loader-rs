# Consolidation from spark-turtle → settings-loader

This document details specific components from `spark-turtle` that should be consolidated back into `settings-loader-rs` as first-class features.

---

## Overview

During the development of `spark-turtle`'s TUI settings editor, several capabilities were needed that `settings-loader-rs` doesn't provide. Rather than maintaining these in application code, they represent general-purpose functionality that belongs in the configuration library.

## Components to Consolidate

### 1. ConfigEditor (Multi-Format Editing)

**Source**: `spark-turtle/crates/turtle-core/src/config/editor.rs`

**Current Implementation**:
```rust
pub struct ConfigEditor {
    path: PathBuf,
    backend: EditorBackend,
}

enum EditorBackend {
    Toml(DocumentMut),  // toml_edit for comment preservation
    Json(serde_json::Value),
    Yaml(serde_yaml::Value),
}
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

**Current Implementation**:
```rust
pub struct SettingMetadata {
    pub key: &'static str,
    pub description: &'static str,
    pub default_value: &'static str,
    pub is_relevant: fn(&TurtleConfig) -> bool,
}

pub struct SettingsRegistry;

impl SettingsRegistry {
    pub fn all() -> &'static [SettingMetadata] { ... }
    pub fn available_for(config: &TurtleConfig) -> Vec<&'static SettingMetadata> { ... }
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

**Source**: `spark-turtle/crates/turtle-tui/src/settings.rs`

**Current Implementation**:
```rust
fn init_editors(&mut self, project_path: Option<&Path>) {
    // User Global
    if let Some(proj_dirs) = ProjectDirs::from("", "turtle", "spark-turtle") {
        let setting_path = proj_dirs.config_dir().join("settings.toml");
        self.user_editor = ConfigEditor::load(&setting_path).ok();
    }
    
    // Project Local
    let path = project_path.unwrap_or(&PathBuf::from("turtle.toml"));
    self.project_editor = ConfigEditor::load(&path).ok();
}
```

**Generalization for settings-loader**:
```rust
pub trait ScopedConfig: SettingsLoader {
    /// Application name for directory resolution
    const APP_NAME: &'static str;
    const ORG_NAME: &'static str = "";
    
    /// Config filename (supports multiple extensions)
    const CONFIG_BASENAME: &'static str = "settings";
    const SUPPORTED_EXTENSIONS: &'static [&'static str] = &["toml", "yaml", "json"];
    
    fn resolve_path(scope: ConfigScope) -> Option<PathBuf> {
        match scope {
            ConfigScope::UserGlobal => {
                directories::ProjectDirs::from("", Self::ORG_NAME, Self::APP_NAME)
                    .map(|d| Self::find_config_in(d.config_dir()))
            }
            ConfigScope::ProjectLocal => {
                Self::find_config_in(Path::new("."))
            }
            // ...
        }
    }
    
    fn find_config_in(dir: &Path) -> Option<PathBuf> {
        for ext in Self::SUPPORTED_EXTENSIONS {
            let path = dir.join(format!("{}.{}", Self::CONFIG_BASENAME, ext));
            if path.exists() {
                return Some(path);
            }
        }
        None
    }
}
```

---

### 4. Environment Variable Format Customization

**Source**: `spark-turtle` uses `TURTLE__LLM__OLLAMA__BASE_URL` format

**Current settings-loader**: Hardcoded `APP__` prefix

**Proposed Change**:
```rust
pub trait LoadingOptions: Sized {
    // ... existing methods ...
    
    /// Prefix for environment variable overrides
    /// Default: "APP"
    fn env_prefix() -> &'static str { "APP" }
    
    /// Separator between nested keys in env vars
    /// Default: "__"  
    fn env_separator() -> &'static str { "__" }
    
    /// Transform a config key to env var name
    /// Default: PREFIX + key.replace('.', separator).to_uppercase()
    fn key_to_env_var(key: &str) -> String {
        format!(
            "{}{}{}",
            Self::env_prefix(),
            Self::env_separator(),
            key.to_uppercase().replace('.', Self::env_separator())
        )
    }
}
```

**turtle usage would become**:
```rust
impl LoadingOptions for TurtleOptions {
    fn env_prefix() -> &'static str { "TURTLE" }
    // ... rest unchanged
}
```

---

### 5. Source Provenance Tracking

**Source**: `spark-turtle` needs to show "Effective" view with source indicators

**Current turtle approach**: Separate effective config view without sources

**Proposed for settings-loader**:
```rust
pub struct LoadedSettings<T> {
    pub settings: T,
    pub sources: SourceMap,
}

pub struct SourceMap {
    entries: HashMap<String, SettingSource>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingSource {
    Default,
    File { path: PathBuf, line: Option<usize> },
    Environment { var_name: String },
    CliArgument { flag: String },
    Computed,
}

impl SettingSource {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Default => "Default",
            Self::File { path, .. } => path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("File"),
            Self::Environment { .. } => "Environment",
            Self::CliArgument { .. } => "CLI",
            Self::Computed => "Computed",
        }
    }
}
```

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
