# Architecture Consensus: Multi-Scope Serde Deserialization

**Date**: 2025-12-19  
**Status**: Draft Architecture Proposal  
**Focus**: How to retain serde multi-source deserialization while adding provenance, editing, and introspection

---

## The Core Question

**How do we keep the most valuable feature of the config crate (multi-source serde deserialization) while adding:**
- Source provenance tracking (which file/env-var provided value?)
- Configuration editing with comment preservation
- Runtime introspection for UIs
- Multi-scope management (user-global vs project-local)

**Answer**: Wrap the config crate, don't replace it. Preserve its strengths, extend with layers above.

---

## Config Crate Architecture (Analyzed)

### What config Does Well ✅

**Core Strength: Source Composition Pipeline**

```
Defaults (Map<key, value>)
    ↓
Sources [File, Env, CustomSource]  ← Trait-based, extendable
    ↓
Merge Engine (precedence-aware)
    ↓
Unified Cache (Value tree)
    ↓
Serde Deserializer (Config implements Deserialize)
    ↓
Typed Settings Struct
```

**Key insight**: The config crate provides two critical services:
1. **Multi-format source collection** (files, env vars, custom)
2. **Serde integration** (Config itself is a Deserializer)

This allows:
```rust
let config = Config::builder()
    .set_default("db.host", "localhost")?
    .add_source(File::from("config.yml"))
    .add_source(Environment::with_prefix("APP"))
    .set_override("db.port", 5432)?
    .build()?;

let settings: DbSettings = config.try_deserialize()?;  // ← Serde magic
```

**The issue**: Once merged, you **lose** which source provided which value. The cache is opaque.

### What Config Doesn't Provide ❌

1. **Provenance tracking**: Which source did db.host come from?
2. **Layer preservation**: Can't query "what's in the environment layer only?"
3. **Bi-directional editing**: Can't save back to files
4. **Introspection**: No metadata about available settings
5. **Comment preservation**: Loses TOML comments during merge/deserialize

---

## Proposed Architecture: settings-loader Wraps Config

### Layer 1: Core (Still uses config crate)

```rust
// settings-loader/src/core/config.rs
pub struct ConfigSource {
    priority: u32,
    inner: Box<dyn config::Source + Send + Sync>,
    metadata: SourceMetadata,  // ← Track origin
}

pub struct SourceMetadata {
    pub id: String,            // "defaults", "file:config.yml", "env:APP_"
    pub source_type: SourceType,
    pub format: Option<FileFormat>,
    pub path: Option<PathBuf>,
}

pub enum SourceType {
    Default,
    File,
    Environment,
    Override,
}
```

**Key change**: Wrap each config::Source with metadata BEFORE adding to builder.

```rust
let builder = Config::builder()
    .set_default("db.host", "localhost")?;  // Auto-tracks as Default

let builder = builder.add_source(
    ConfigSource::from_file(
        File::from("config.yml"),
        SourceMetadata { id: "file:config.yml", ... }
    )
)?;

let config = builder.build()?;  // Returns Config (unchanged)
let settings: DbSettings = config.try_deserialize()?;  // Serde still works
```

**Benefit**: Config crate behavior unchanged. We just track which source is which.

---

### Layer 2: Provenance Tracking (New)

```rust
// settings-loader/src/provenance.rs
pub struct SourceMap {
    /// key -> (source_metadata, value_at_that_layer)
    entries: HashMap<String, (SourceMetadata, Value)>,
}

impl SourceMap {
    pub fn source_of(&self, key: &str) -> Option<&SourceMetadata>;
    
    pub fn all_from(&self, source_type: SourceType) -> Vec<(String, Value)>;
    
    pub fn precedence_chain(&self, key: &str) -> Vec<(SourceMetadata, Value)>;
}

pub fn load_with_provenance<T: DeserializeOwned>(
    sources: Vec<ConfigSource>,
    defaults: Map<String, Value>,
) -> Result<(T, SourceMap)> {
    // Build normal config
    let config = build_config(sources, defaults)?;
    
    // ALSO build provenance by querying each source individually
    let mut provenance = SourceMap::new();
    for source in &sources {
        let layer_values = source.inner.collect()?;  // Gets values from THIS source only
        for (key, value) in layer_values {
            provenance.add(key, source.metadata.clone(), value);
        }
    }
    
    // Deserialize as normal
    let settings: T = config.try_deserialize()?;
    
    Ok((settings, provenance))
}
```

**Critical insight**: We DON'T modify config's merge algorithm. We track sources **in parallel**.

---

### Layer 3: Explicit Layering (Phase 1)

Phase 1's `LayerBuilder` becomes a convenience wrapper:

```rust
// settings-loader/src/layer.rs
pub struct LayerBuilder {
    layers: Vec<(LayerName, ConfigSource)>,
}

impl LayerBuilder {
    pub fn with_defaults(defaults: Map<String, Value>) -> Self { ... }
    
    pub fn with_path(path: PathBuf) -> Self {
        self.layers.push((
            LayerName::from_path(&path),
            ConfigSource::from_file(
                File::from(path.clone()),
                SourceMetadata { id: format!("file:{}", path.display()), ... }
            )
        ));
        self
    }
    
    pub fn with_env_vars(prefix: &str, separator: &str) -> Self { ... }
    
    pub fn build<T: DeserializeOwned>(self) -> Result<(T, SourceMap)> {
        // Just calls load_with_provenance with our organized sources
        load_with_provenance(self.layers.into_sources(), defaults)
    }
}
```

**Benefit**: Layers are just named sources. Provenance automatically tracks them.

---

### Layer 4: Multi-Scope Support (Phase 3)

```rust
// settings-loader/src/multi_scope.rs
pub enum ConfigScope {
    System,
    UserGlobal,
    ProjectLocal,
    Runtime,
}

pub trait MultiScopeLoader: DeserializeOwned {
    const APP_NAME: &'static str;
    
    fn resolve_paths(scope: ConfigScope) -> Vec<PathBuf> {
        match scope {
            ConfigScope::UserGlobal => {
                // ~/.config/app_name/settings.toml
                directories::ProjectDirs::from("", "", Self::APP_NAME)
                    .map(|d| vec![d.config_dir().join("settings.toml")])
                    .unwrap_or_default()
            }
            ConfigScope::ProjectLocal => {
                vec![PathBuf::from("settings.toml")]
            }
            // ...
        }
    }
    
    fn load_multi_scope() -> Result<(Self, SourceMap)> {
        let mut layers = vec![];
        
        // Apply in order of increasing precedence
        for path in Self::resolve_paths(ConfigScope::System) {
            if path.exists() {
                layers.push(ConfigSource::from_file(
                    File::from(&path),
                    SourceMetadata { source_type: SourceType::File(ConfigScope::System), ... }
                ));
            }
        }
        
        for path in Self::resolve_paths(ConfigScope::UserGlobal) {
            if path.exists() {
                layers.push(ConfigSource::from_file(
                    File::from(&path),
                    SourceMetadata { source_type: SourceType::File(ConfigScope::UserGlobal), ... }
                ));
            }
        }
        
        // ... ProjectLocal ...
        // ... Runtime (env vars) ...
        
        load_with_provenance(layers, defaults)
    }
}
```

---

### Layer 5: Editing (Phase 4)

```rust
// settings-loader/src/editor.rs
pub struct LayerEditor {
    scope: ConfigScope,
    path: PathBuf,
    backend: EditorBackend,  // toml_edit, json, yaml
}

impl LayerEditor {
    pub fn for_scope(scope: ConfigScope) -> Result<Self> {
        let path = /* resolve path for scope */;
        let backend = EditorBackend::from_path(&path)?;
        Ok(Self { scope, path, backend })
    }
    
    pub fn get<T: FromStr>(&self, key: &str) -> Result<T> {
        self.backend.get(key)
    }
    
    pub fn set<T: ToString>(&mut self, key: &str, value: T) -> Result<()> {
        self.backend.set(key, value.to_string())
    }
    
    pub fn save(&self) -> Result<()> {
        self.backend.save(&self.path)
    }
}

pub enum EditorBackend {
    Toml(toml_edit::Document),  // Preserves comments
    Json(serde_json::Value),
    Yaml(serde_yaml::Value),
}
```

**Key property**: Editor works on **individual layers**, not merged config. Preserves layer boundaries and comments.

---

### Layer 6: Introspection (Phase 5)

```rust
// settings-loader/src/introspection.rs
pub trait SettingsIntrospection {
    fn schema(&self) -> ConfigSchema;
}

pub struct ConfigSchema {
    pub settings: Vec<SettingMetadata>,
    pub groups: Vec<SettingGroup>,
}
```

**Independence**: Introspection doesn't depend on config crate at all. Pure metadata.

---

## Complete Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                   User Application Code                         │
├─────────────────────────────────────────────────────────────────┤
│  let (settings, sources) = AppConfig::load_with_provenance()?;  │
│  println!("db_host from: {:?}", sources.source_of("db_host")); │
└────────────────────────────┬────────────────────────────────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
┌──────────────────┐ ┌──────────────┐ ┌──────────────────┐
│ LayerBuilder     │ │MultiScope    │ │ LayerEditor      │
│ (Phase 1)        │ │ Loader       │ │ (Phase 4)        │
│                  │ │ (Phase 3)    │ │                  │
│Explicit layer    │ │              │ │Bidirectional     │
│composition       │ │Auto-resolve  │ │editing with      │
│                  │ │system/user/  │ │comment preserve  │
└────────┬─────────┘ │local paths   │ └────────┬─────────┘
         │           │              │          │
         │           └──────┬───────┘          │
         │                  │                  │
         │    ┌─────────────┴──────────────────┘
         │    │
         ▼    ▼
    ┌─────────────────────────────────────────┐
    │  SourceMetadata + Provenance Tracking   │
    │  (Phase 2 - NEW)                        │
    │                                         │
    │  Wraps each config::Source with:        │
    │  - Source ID and type                   │
    │  - Parallel collection for tracking     │
    │  - Returns (T, SourceMap)               │
    └────────────────┬────────────────────────┘
                     │
                     ▼
    ┌─────────────────────────────────────────┐
    │  Config Crate (Unchanged)               │
    │                                         │
    │  ✅ Multi-format sources                │
    │  ✅ Merge with correct precedence       │
    │  ✅ Serde deserializer impl             │
    │                                         │
    │  Builder pattern:                       │
    │  - set_default()                        │
    │  - add_source() [File, Env, Custom]     │
    │  - set_override()                       │
    │  - build() -> Config                    │
    └────────────────┬────────────────────────┘
                     │
                     ▼
    ┌─────────────────────────────────────────┐
    │  Introspection / Metadata               │
    │  (Phase 5)                              │
    │                                         │
    │  ConfigSchema + SettingMetadata         │
    │  (Optional, independent)                │
    └─────────────────────────────────────────┘
```

---

## Why This Solves the Problem

### 1. **Serde Deserialization Preserved** ✅

```rust
// Old API still works perfectly
let settings: AppSettings = config.try_deserialize()?;

// New API adds provenance
let (settings, sources) = AppConfig::load_with_provenance()?;
```

The config crate's merged cache is used directly for serde. No duplication.

### 2. **Provenance Without Modifying Config** ✅

We build provenance **in parallel** by:
1. Letting config crate do its normal merge
2. ALSO querying each source individually for tracking
3. Returning both the deserialized struct AND the SourceMap

Zero changes to config crate internal logic.

### 3. **Layers Become Named Sources** ✅

Phase 1's explicit layering is just a convenience:
```rust
LayerBuilder creates ConfigSource wrapped with SourceMetadata
  → load_with_provenance uses both
  → SourceMap automatically tracks which layer each value came from
```

### 4. **Editing Works at Layer Granularity** ✅

```rust
// Edit ONLY the project-local layer
let mut editor = LayerEditor::for_scope(ConfigScope::ProjectLocal)?;
editor.set("db.port", 5432)?;
editor.save()?;  // Saves to ./settings.toml with comments preserved

// Doesn't touch user-global or system defaults
```

### 5. **Multi-Scope Paths are Just Bookkeeping** ✅

```rust
// Phase 3 MultiScopeLoader just organizes the layers:
// System defaults → User global → Project local → Runtime env
// 
// All handled by layering + provenance + source tracking
```

---

## Migration Path: Wrap, Don't Replace

### Phase 1-5 (Current)
- Keep using `config` crate for merging
- Add `SourceMetadata` wrapper (transparent to config)
- Build provenance tracking layer above it
- Existing code works unchanged

### Phase 6 (Future)
- If we want to replace config crate entirely, we can
- But we have working provenance in place first
- Can refactor bottom layers without breaking top layers

### Feature Flags

```toml
[dependencies]
config = "0.13"  # Core serde support

[features]
default = []

# Phase 1: Explicit layering
layering = []

# Phase 2: Source provenance (NEW - the key addition)
provenance = []

# Phase 3: Multi-scope support
multi-scope = ["directories"]

# Phase 4: Bidirectional editing
editor = ["toml_edit"]

# Phase 5: Introspection
introspection = []

full = ["layering", "provenance", "multi-scope", "editor", "introspection"]
```

---

## API Comparison: Old vs New

### Old (v0.15.0)
```rust
let options = MyOptions::default();
let settings = MySettings::load(&options)?;
println!("db_host: {}", settings.database.host);
```

### New (v1.0.0)
```rust
// Option A: Just want settings (backward compatible)
let settings = MySettings::load(&MyOptions::default())?;

// Option B: Want to know sources (new)
let (settings, sources) = MySettings::load_with_provenance(&MyOptions::default())?;
match sources.source_of("database.host") {
    Some(meta) => println!("from {:?}", meta.source_type),
    None => println!("using default"),
}

// Option C: Explicit layering
let mut builder = LayerBuilder::new();
builder
    .with_defaults(/* ... */)
    .with_path("/etc/app/settings.toml")
    .with_env_vars("APP_", "__")
    .with_path("./settings.local.toml");
let (settings, sources) = builder.build::<MySettings>()?;

// Option D: Multi-scope
let (settings, sources) = MySettings::load_multi_scope()?;

// Option E: Edit a layer
let mut editor = LayerEditor::for_scope(ConfigScope::ProjectLocal)?;
editor.set("database.host", "prod.db.example.com")?;
editor.save()?;

// Option F: Introspection
for meta in MySettings::schema().settings {
    println!("{}: {} (default: {:?})", 
        meta.key, meta.description, meta.default);
}
```

---

## Backward Compatibility Matrix

| Feature | v0.15.0 | v1.0.0 | Impact |
|---------|---------|--------|--------|
| Basic load + deserialize | ✅ | ✅ | No breaking changes |
| Serde trait impl | ✅ | ✅ | Still works |
| config crate under hood | ✅ | ✅ | Dependency preserved |
| New provenance API | ❌ | ✅ | Opt-in feature |
| New editor API | ❌ | ✅ | Opt-in feature |
| New multi-scope | ❌ | ✅ | Opt-in feature |
| New introspection | ❌ | ✅ | Opt-in feature |

---

## Implementation Order

1. **Phase 1-2**: SourceMetadata wrapper + Provenance tracking
   - Non-breaking addition to existing load flow
   - Parallel tracking doesn't affect config crate usage
   - Enables `load_with_provenance()` method

2. **Phase 3**: Multi-scope loader
   - Uses provenance infrastructure
   - Just organizes sources by scope
   - Adds `directories` dependency (optional feature)

3. **Phase 4**: Layer editor
   - Works on individual scoped files
   - Independent of config crate's merged cache
   - Adds `toml_edit` dependency (optional feature)

4. **Phase 5**: Introspection
   - Completely independent
   - Can be done anytime after design settled

---

## Addressing Your Questions

### DMR#5: "How do we retain serde support?"

**Answer**: We never lose it. Config crate still does the merge and serde deserialization. We wrap it with provenance tracking that happens in parallel. The merged config goes directly to serde. No duplication, no modification of config's core behavior.

### DMR#6: "Key agreement on multi-scoped/layered serde is paramount"

**Answer**: That's exactly what this architecture provides:
- Serde handles deserialization (config crate)
- Layering handles composition and precedence (LayerBuilder + named sources)
- Provenance handles tracking (SourceMap)
- Multi-scope handles path resolution (MultiScopeLoader)

All four work together, each doing one thing well.

---

## Key Decisions Documented

1. **Don't replace config crate** - Wrap it instead
2. **Provenance is parallel** - Not embedded in config's merge
3. **Layers are named sources** - Explicit layering uses config's source abstraction
4. **Editing is layer-scoped** - Not on merged cache
5. **Introspection is optional** - Metadata independent of config loading

---

## Next Steps

1. ✅ Review this architecture against config crate realities
2. ⬜ Update Phase 1-2 implementation plan with SourceMetadata wrapping
3. ⬜ Create `src/provenance.rs` with SourceMap design
4. ⬜ Refactor existing phase documents to reference this architecture
5. ⬜ Create migration guide showing serde support preserved

