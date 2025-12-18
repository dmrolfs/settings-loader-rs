# Phase 5: Settings Metadata & Introspection - Architecture

**Epic**: sl-wnc  
**Phase**: 5 of 7  
**Status**: Design Phase  
**Created**: December 18, 2025

---

## Executive Summary

Phase 5 adds runtime introspection capabilities to settings-loader-rs, enabling applications to query available settings, their types, constraints, and default values. This is essential for building TUI/CLI configuration editors, validation systems, and auto-generated documentation.

**Key Innovation**: Metadata system that works both at compile-time (via traits) and runtime (via optional proc-macro), supporting both manual and automatic metadata generation.

---

## Objectives

### Primary Goals

1. **Runtime Introspection**: Query available settings and their metadata at runtime
2. **Type Safety**: Rich type system with validation constraints
3. **TUI/CLI Support**: Enable automatic UI generation from metadata
4. **Validation Framework**: Constraint checking with meaningful error messages
5. **Security**: Visibility control to hide secrets from UI display
6. **Flexibility**: Support both manual and automatic metadata generation

### Non-Goals

1. **Schema Migration**: Not handling versioning or migrations (deferred to Phase 7)
2. **Dynamic Configuration**: Settings structure remains static (defined at compile time)
3. **Custom Validators**: Complex custom validation logic (beyond basic constraints)

---

## Architecture Overview

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                     SettingsIntrospection                   │
│                         (Trait)                              │
└──────────────────────┬──────────────────────────────────────┘
                       │
         ┌─────────────┴─────────────┐
         │                           │
         ▼                           ▼
┌────────────────────┐      ┌────────────────────┐
│  ConfigSchema      │      │ SettingMetadata    │
│  (Full schema)     │      │ (Single setting)   │
└────────────────────┘      └────────────────────┘
         │                           │
         │                           ▼
         │                  ┌────────────────────┐
         │                  │   SettingType      │
         │                  │   (Type enum)      │
         │                  └────────────────────┘
         │                           │
         │                           ▼
         │                  ┌────────────────────┐
         │                  │   Constraint       │
         │                  │   (Validation)     │
         │                  └────────────────────┘
         │                           │
         │                           ▼
         │                  ┌────────────────────┐
         │                  │   Visibility       │
         │                  │   (UI control)     │
         │                  └────────────────────┘
         │
         ▼
┌────────────────────────────────────────────────────────────┐
│           Optional: settings-loader-derive                  │
│           (Proc-macro for auto-generation)                  │
└────────────────────────────────────────────────────────────┘
```

---

## Type Definitions

### 1. SettingMetadata

Core structure describing a single setting:

```rust
#[derive(Debug, Clone)]
pub struct SettingMetadata {
    /// Setting key (dot-separated path)
    pub key: String,
    
    /// Human-readable label for UI
    pub label: String,
    
    /// Description/documentation
    pub description: String,
    
    /// Type information with constraints
    pub setting_type: SettingType,
    
    /// Default value (JSON-serialized for flexibility)
    pub default: Option<serde_json::Value>,
    
    /// Validation constraints
    pub constraints: Vec<Constraint>,
    
    /// UI visibility control
    pub visibility: Visibility,
    
    /// Group/category for organization
    pub group: Option<String>,
}
```

### 2. SettingType

Rich type system with built-in validation:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SettingType {
    /// String value with optional pattern
    String {
        pattern: Option<String>,  // Regex pattern
        min_length: Option<usize>,
        max_length: Option<usize>,
    },
    
    /// Integer value with range
    Integer {
        min: Option<i64>,
        max: Option<i64>,
    },
    
    /// Floating point value with range
    Float {
        min: Option<f64>,
        max: Option<f64>,
    },
    
    /// Boolean flag
    Boolean,
    
    /// Duration (seconds, milliseconds, etc.)
    Duration {
        min: Option<std::time::Duration>,
        max: Option<std::time::Duration>,
    },
    
    /// Filesystem path
    Path {
        must_exist: bool,
        is_directory: bool,
    },
    
    /// URL with scheme validation
    Url {
        schemes: Vec<String>,  // ["http", "https"]
    },
    
    /// Enum with fixed variants
    Enum {
        variants: Vec<String>,
    },
    
    /// Secret value (masked in UI)
    Secret,
    
    /// Array of values
    Array {
        element_type: Box<SettingType>,
        min_items: Option<usize>,
        max_items: Option<usize>,
    },
    
    /// Nested object
    Object {
        fields: Vec<SettingMetadata>,
    },
}
```

### 3. Constraint

Validation constraints:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Must match regex pattern
    Pattern(String),
    
    /// Numeric range (min, max)
    Range { min: f64, max: f64 },
    
    /// String length range
    Length { min: usize, max: usize },
    
    /// Required (cannot be None/null)
    Required,
    
    /// One of a set of values
    OneOf(Vec<String>),
    
    /// Custom validator (name only, actual validation in application)
    Custom(String),
}
```

### 4. Visibility

UI visibility control:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    /// Always visible
    Public,
    
    /// Hidden from UI (but accessible programmatically)
    Hidden,
    
    /// Secret value (show masked in UI)
    Secret,
    
    /// Advanced setting (show in "Advanced" section)
    Advanced,
}
```

### 5. ConfigSchema

Full schema representation:

```rust
#[derive(Debug, Clone)]
pub struct ConfigSchema {
    /// Application name
    pub name: String,
    
    /// Schema version
    pub version: String,
    
    /// All setting metadata
    pub settings: Vec<SettingMetadata>,
    
    /// Optional: Nested groups
    pub groups: Vec<SettingGroup>,
}

#[derive(Debug, Clone)]
pub struct SettingGroup {
    pub name: String,
    pub label: String,
    pub description: String,
    pub settings: Vec<String>,  // Setting keys
}
```

---

## Trait Design

### SettingsIntrospection Trait

Core trait for metadata access:

```rust
/// Runtime introspection of settings metadata
pub trait SettingsIntrospection: SettingsLoader {
    /// Get complete schema
    fn schema() -> ConfigSchema;
    
    /// Get all setting metadata
    fn metadata() -> &'static [SettingMetadata];
    
    /// Get metadata for specific setting
    fn metadata_for(key: &str) -> Option<&'static SettingMetadata> {
        Self::metadata().iter().find(|m| m.key == key)
    }
    
    /// Validate a value against metadata
    fn validate_value(key: &str, value: &serde_json::Value) -> Result<(), ValidationError> {
        // Default implementation using metadata
        // Applications can override for custom validation
    }
    
    /// Get all setting keys
    fn keys() -> Vec<&'static str> {
        Self::metadata().iter().map(|m| m.key.as_str()).collect()
    }
    
    /// Get settings by group
    fn settings_in_group(group: &str) -> Vec<&'static SettingMetadata> {
        Self::metadata().iter().filter(|m| {
            m.group.as_deref() == Some(group)
        }).collect()
    }
}
```

---

## Implementation Approaches

### Approach 1: Manual Metadata (No Dependencies)

For applications that want explicit control:

```rust
impl SettingsIntrospection for MySettings {
    fn schema() -> ConfigSchema {
        ConfigSchema {
            name: "my-app".to_string(),
            version: "1.0.0".to_string(),
            settings: vec![
                SettingMetadata {
                    key: "api_url".to_string(),
                    label: "API URL".to_string(),
                    description: "API endpoint URL".to_string(),
                    setting_type: SettingType::Url {
                        schemes: vec!["http".into(), "https".into()],
                    },
                    default: Some(json!("http://localhost:8080")),
                    constraints: vec![Constraint::Required],
                    visibility: Visibility::Public,
                    group: Some("api".to_string()),
                },
                // ... more settings
            ],
            groups: vec![],
        }
    }
    
    fn metadata() -> &'static [SettingMetadata] {
        // Lazily initialized static
        &METADATA
    }
}
```

### Approach 2: Proc-Macro (Optional)

For automatic generation from struct definition:

```rust
use settings_loader_derive::SettingsSchema;

#[derive(Debug, Deserialize, SettingsSchema)]
#[settings(name = "my-app", version = "1.0.0")]
pub struct MySettings {
    #[setting(
        description = "API endpoint URL",
        default = "http://localhost:8080",
        url(schemes = ["http", "https"]),
        group = "api"
    )]
    pub api_url: String,
    
    #[setting(
        description = "Request timeout in seconds",
        default = 30,
        range(min = 1, max = 300),
        group = "api"
    )]
    pub timeout_secs: u64,
    
    #[setting(
        description = "API key",
        secret,
        group = "api"
    )]
    pub api_key: Option<String>,
}

// Proc-macro generates SettingsIntrospection impl automatically
```

---

## Validation Framework

### Validation Process

```rust
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub key: String,
    pub message: String,
    pub constraint: Option<Constraint>,
}

impl SettingMetadata {
    /// Validate a value against this setting's constraints
    pub fn validate(&self, value: &serde_json::Value) -> Result<(), ValidationError> {
        // Type checking
        self.setting_type.validate_type(value)?;
        
        // Constraint checking
        for constraint in &self.constraints {
            constraint.validate(&self.key, value)?;
        }
        
        Ok(())
    }
}

impl SettingType {
    fn validate_type(&self, value: &serde_json::Value) -> Result<(), ValidationError> {
        match (self, value) {
            (SettingType::String { .. }, serde_json::Value::String(_)) => Ok(()),
            (SettingType::Integer { .. }, serde_json::Value::Number(n)) if n.is_i64() => Ok(()),
            (SettingType::Boolean, serde_json::Value::Bool(_)) => Ok(()),
            _ => Err(ValidationError {
                key: "".to_string(),
                message: format!("Type mismatch: expected {:?}", self),
                constraint: None,
            }),
        }
    }
}
```

---

## Use Cases

### 1. TUI Configuration Editor

```rust
use settings_loader::SettingsIntrospection;

fn build_tui_editor<T: SettingsIntrospection>() {
    let schema = T::schema();
    
    for setting in schema.settings {
        match setting.visibility {
            Visibility::Secret => println!("[****] {}: {}", setting.label, setting.description),
            Visibility::Hidden => continue,
            _ => {
                match &setting.setting_type {
                    SettingType::Boolean => render_checkbox(&setting),
                    SettingType::Enum { variants } => render_select(&setting, variants),
                    SettingType::Integer { min, max } => render_number_input(&setting, *min, *max),
                    _ => render_text_input(&setting),
                }
            }
        }
    }
}
```

### 2. CLI Help Generation

```rust
fn generate_cli_help<T: SettingsIntrospection>() {
    let schema = T::schema();
    
    println!("Configuration Options:");
    for setting in schema.settings {
        if setting.visibility == Visibility::Hidden {
            continue;
        }
        
        println!("  --{}", setting.key);
        println!("      {}", setting.description);
        if let Some(default) = &setting.default {
            println!("      Default: {}", default);
        }
        println!();
    }
}
```

### 3. Validation Before Save

```rust
fn validate_settings<T: SettingsIntrospection>(
    values: &HashMap<String, serde_json::Value>
) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    
    for metadata in T::metadata() {
        if let Some(value) = values.get(&metadata.key) {
            if let Err(e) = metadata.validate(value) {
                errors.push(e);
            }
        } else if metadata.constraints.contains(&Constraint::Required) {
            errors.push(ValidationError {
                key: metadata.key.clone(),
                message: "Required setting is missing".to_string(),
                constraint: Some(Constraint::Required),
            });
        }
    }
    
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

---

## Feature Flags

```toml
[features]
# Core metadata types (no dependencies)
metadata = []

# Proc-macro for automatic generation
metadata-derive = ["metadata", "settings-loader-derive"]

# Full metadata support
full-metadata = ["metadata", "metadata-derive"]
```

---

## Backward Compatibility

✅ **100% Backward Compatible**

- New trait `SettingsIntrospection` is optional
- Applications without metadata continue working unchanged
- Feature flags control optional dependencies
- No changes to existing traits or types

---

## Integration with Other Phases

### Phase 4 Integration (Configuration Editing)

```rust
// Use metadata to validate before saving
let mut editor = MySettings::editor(ConfigScope::ProjectLocal, &options)?;
editor.set("timeout_secs", 60)?;

// Validate using metadata
if let Err(errors) = MySettings::validate_value("timeout_secs", &json!(60)) {
    // Handle validation errors
}

editor.save()?;
```

### Phase 6 Integration (Source Provenance)

```rust
// Combine metadata with provenance
let (settings, sources) = MySettings::load_with_provenance(&options)?;

for metadata in MySettings::metadata() {
    if let Some(source) = sources.source_of(&metadata.key) {
        println!("{} ({}): {:?}", metadata.label, metadata.description, source);
    }
}
```

### Phase 7 Integration (Schema Export)

```rust
// Export JSON Schema using metadata
let schema = MySettings::schema();
let json_schema = schema.to_json_schema();
fs::write("schema.json", serde_json::to_string_pretty(&json_schema)?)?;
```

---

## Testing Strategy

### Unit Tests

1. **Metadata Construction**: Verify SettingMetadata struct creation
2. **Type Validation**: Test SettingType validation for each type
3. **Constraint Validation**: Test each Constraint variant
4. **Visibility Control**: Test filtering by visibility

### Integration Tests

1. **Manual Implementation**: Test manual SettingsIntrospection impl
2. **Schema Query**: Test schema() and metadata() methods
3. **Validation Errors**: Test validation error messages
4. **Group Organization**: Test settings_in_group()

### Proc-Macro Tests (if implemented)

1. **Code Generation**: Verify generated trait impl
2. **Attribute Parsing**: Test all #[setting(...)] attributes
3. **Complex Types**: Test nested objects and arrays
4. **Error Messages**: Test compilation errors for invalid attributes

---

## Performance Considerations

### Static Metadata

- Metadata stored in static `&'static [SettingMetadata]`
- No runtime allocation for metadata access
- Schema construction is lazy (on first access)

### Validation Performance

- Constraint checking is O(n) in number of constraints
- Type checking is O(1)
- No regex compilation on validation path (compile-time or lazy static)

---

## Security Considerations

### Secret Handling

1. **Visibility Control**: Secrets marked with `Visibility::Secret`
2. **UI Masking**: TUI/CLI must respect visibility flags
3. **No Default Secrets**: Secrets should never have default values in metadata
4. **Validation**: Secret constraints should not leak information in error messages

---

## Dependencies

### Core (metadata feature)

- **None**: Base metadata types have no new dependencies
- Uses existing `serde_json` for default value serialization

### Proc-Macro (metadata-derive feature)

- `syn`: Parsing Rust syntax
- `quote`: Code generation
- `proc-macro2`: Proc-macro utilities
- `darling`: Attribute parsing helper (optional, simplifies macro implementation)

---

## Documentation Requirements

### API Documentation

1. **Trait Documentation**: SettingsIntrospection with examples
2. **Type Documentation**: Each SettingType variant with examples
3. **Constraint Documentation**: Each Constraint with validation rules
4. **Proc-Macro Documentation**: All #[setting(...)] attributes

### User Guide

1. **Manual Metadata**: How to implement SettingsIntrospection manually
2. **Proc-Macro Usage**: How to use #[derive(SettingsSchema)]
3. **TUI Integration**: Example TUI editor using metadata
4. **Validation**: How to validate settings before save

---

## Success Criteria

Phase 5 is complete when:

- [ ] All core types implemented (SettingMetadata, SettingType, Constraint, Visibility, ConfigSchema)
- [ ] SettingsIntrospection trait defined with default implementations
- [ ] Manual metadata implementation working (no proc-macro)
- [ ] Validation framework functional
- [ ] All unit tests passing
- [ ] Integration tests demonstrating TUI/CLI use cases
- [ ] Optional: Proc-macro implemented and tested
- [ ] Documentation complete with examples
- [ ] 0 clippy warnings
- [ ] 100% backward compatible

---

## Next Steps

1. **Phase 5.1**: Implement core types (SettingMetadata, SettingType, etc.)
2. **Phase 5.2**: Implement SettingsIntrospection trait
3. **Phase 5.3**: Implement validation framework
4. **Phase 5.4**: Integration tests and examples
5. **Phase 5.5**: Optional proc-macro implementation
6. **Phase 5.6**: Documentation and quality review

---

**Created**: December 18, 2025  
**Author**: GitHub Copilot CLI Agent  
**Status**: Design Complete - Ready for Implementation Plan
