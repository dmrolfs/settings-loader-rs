# settings-loader-rs

**A comprehensive Rust configuration management library that unifies multiple sources into type-safe, validated settings.**

`settings-loader` wraps and extends [`config-rs`](https://github.com/mehcode/config-rs) with powerful features for modern Rust applications: bidirectional editing with comment preservation, metadata-driven introspection, multi-scope path resolution, and source provenance tracking.

> **Status**: Production-ready at v1.0.0 with comprehensive test coverage (88%+ mutation score).

## Why settings-loader?

**Multi-Source Composition**: Seamlessly merge configuration from files, environment variables, secrets, and CLI arguments with customizable precedence rules.

**Type Safety**: Leverage Rust's type system and serde for compile-time guarantees—no runtime type errors.

**Metadata & Introspection**: Generate UIs, validate configs, and produce documentation directly from your settings structs.

**Bidirectional Editing**: Not just read—write back to config files while preserving comments and formatting.

**Multi-Scope Support**: Handle user-global, project-local, and system configurations with platform-appropriate paths.

**Provenance Tracking**: Debug configuration issues by knowing exactly where each value came from.

---

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
settings-loader = "1.0"
```

Define your settings and load them:

```rust
use serde::Deserialize;
use settings_loader::{SettingsLoader, NoOptions};

#[derive(Debug, Deserialize)]
struct AppSettings {
    host: String,
    port: u16,
    debug: bool,
}

impl SettingsLoader for AppSettings {
    type Options = NoOptions;
    
    fn app_config_basename() -> &'static str {
        "myapp"  // Looks for myapp.yaml, myapp.toml, etc.
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = AppSettings::load(&NoOptions)?;
    println!("Server: {}:{}", settings.host, settings.port);
    Ok(())
}
```

Create `myapp.yaml` in your project root:

```yaml
host: "127.0.0.1"
port: 8080
debug: true
```

That's it! You're loading configuration in under 5 minutes.

---

## Core Features

### Multi-Format Support

Load configuration from any of these formats (listed in precedence order):
1. **YAML** (`.yaml`, `.yml`) - Highest precedence
2. **TOML** (`.toml`)
3. **JSON** (`.json`)
4. **JSON5** (`.json5`)
5. **HJSON** (`.hjson`)
6. **RON** (`.ron`) - Lowest precedence

The format is automatically detected by file extension. **Extension precedence applies independently for each configuration layer**. For example, if a directory contains both `settings.yaml` and `settings.json`, the YAML file will be loaded. This precedence order is defined by the underlying `config-rs` library.

### Hierarchical Merging with Customizable Precedence

Configuration sources are merged with well-defined precedence. The **default precedence** (highest to lowest):

1. **Command-line arguments** (explicit overrides)
2. **Environment variables** (runtime configuration)
3. **Secrets files** (sensitive credentials)
4. **Environment-specific files** (`production.yaml`, `local.yaml`)
5. **Base configuration** (`application.yaml`)
6. **Defaults** (defined in code)

This default enables the [12-factor app](https://12factor.net/config) pattern: store config in the environment, separate secrets, and maintain environment-specific overrides.

**Customizable Precedence**: You can establish any precedence order using `LayerBuilder` to define explicit configuration layers. See [Configuration Source Patterns](#configuration-source-patterns) for examples including desktop/CLI application patterns (system→user→project→runtime) and containerized application patterns.

### Type-Safe Access

Settings are deserialized into strongly-typed Rust structs using serde. This means:
- Compile-time type checking
- No runtime type coercion errors
- IDE autocomplete and refactoring support
- Clear documentation of your configuration schema

---

## Installation & Feature Gates

`settings-loader` uses Cargo features to keep dependencies minimal. Enable only what you need:

```toml
[dependencies]
settings-loader = { version = "1.0", features = ["database", "http", "multi-scope", "editor"] }
```

### Available Features

| Feature | Description | Dependencies |
|---------|-------------|--------------|
| `metadata` (default) | Metadata, introspection, validation, and schema generation | `serde_json`, `regex`, `zeroize` |
| `database` | PostgreSQL connection settings with `secrecy` integration | `sqlx`, `secrecy`, `zeroize` |
| `http` | HTTP server configuration with URL validation | `url` |
| `multi-scope` | User-global vs project-local path resolution | `directories` |
| `editor` | Bidirectional editing with comment preservation | `toml_edit`, `parking_lot`, `serde_json`, `serde_yaml` |

**Default features**: `metadata` is enabled by default, providing introspection and validation capabilities.

**Minimal installation**: Use `default-features = false` to disable all optional features:

```toml
[dependencies]
settings-loader = { version = "1.0", default-features = false }
```

---

## Getting Started Guide

### Step 1: Define Your Settings Struct

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MySettings {
    pub server: ServerSettings,
    pub database: DatabaseSettings,
}

#[derive(Debug, Deserialize)]
struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub max_connections: u32,
}
```

### Step 2: Implement LoadingOptions

For simple cases, use `NoOptions`. For custom loading behavior, implement `LoadingOptions`:

```rust
use std::path::PathBuf;
use settings_loader::{LoadingOptions, SettingsError};
use clap::Parser;

#[derive(Parser)]
struct CliOptions {
    /// Path to configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Path to secrets file
    #[arg(long)]
    secrets: Option<PathBuf>,
    
    /// Environment (local, production, etc.)
    #[arg(short, long)]
    env: Option<String>,
}

impl LoadingOptions for CliOptions {
    type Error = SettingsError;
    
    fn config_path(&self) -> Option<PathBuf> {
        self.config.clone()
    }
    
    fn secrets_path(&self) -> Option<PathBuf> {
        self.secrets.clone()
    }
    
    fn implicit_search_paths(&self) -> Vec<PathBuf> {
        vec![PathBuf::from("./config"), PathBuf::from("./")]
    }
}
```

### Step 3: Implement SettingsLoader

```rust
use settings_loader::SettingsLoader;

impl SettingsLoader for MySettings {
    type Options = CliOptions;
    
    fn app_config_basename() -> &'static str {
        "application"  // Looks for application.yaml, application.toml, etc.
    }
}
```

### Step 4: Load and Use Settings

```rust
use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = CliOptions::parse();
    let settings = MySettings::load(&options)?;
    
    println!("Connecting to database at {}:{}", 
        settings.database.host, 
        settings.database.port
    );
    
    // Use settings in your application...
    Ok(())
}
```

---

## Configuration Sources & Loading

This section describes how configuration sources are discovered, composed, and loaded. Understanding these mechanisms helps you structure your application's configuration effectively.

### Explicit Configuration Paths

The `LoadingOptions::config_path()` method provides an explicit override for the primary configuration file. When set, it bypasses implicit search and loads the specified file directly. This is useful for:
- CLI flags like `--config /path/to/config.yaml`
- Testing with specific configuration files
- Deployment scripts that know exact paths

**Relationship with Multi-Scope**: `config_path()` takes precedence over multi-scope path resolution. If you provide an explicit path, multi-scope discovery is skipped for the base configuration (though secrets and environment-specific files may still use multi-scope paths).

### Multi-Scope Configuration (`multi-scope` feature)

Applications often need configuration at different scopes: system-wide defaults, user preferences, and project-specific settings. The `multi-scope` feature provides automatic, platform-appropriate path resolution using the [`directories`](https://crates.io/crates/directories) crate.

**Platform-Specific Paths**:

| Scope | Linux | macOS | Windows |
|-------|-------|-------|---------|
| **System** | `/etc/<app>/` | `/Library/Application Support/<app>/` | `C:\ProgramData\<org>\<app>\` |
| **UserGlobal** | `~/.config/<app>/` | `~/Library/Application Support/<app>/` | `C:\Users\<user>\AppData\Roaming\<org>\<app>\` |
| **ProjectLocal** | `./<app>.{yaml,toml,json}` | `./<app>.{yaml,toml,json}` | `.\<app>.{yaml,toml,json}` |

Implement `MultiScopeConfig` to enable automatic path resolution:

```rust
use settings_loader::{MultiScopeConfig, ConfigScope};

impl MultiScopeConfig for MySettings {
    const APP_NAME: &'static str = "myapp";
    const ORG_NAME: &'static str = "myorg";
}

// Automatically resolves to platform-appropriate paths
let user_config = MySettings::resolve_path(ConfigScope::UserGlobal);
let project_config = MySettings::resolve_path(ConfigScope::ProjectLocal);
```

**Common Use Case**: CLI tools that respect user preferences while allowing project-specific overrides. For example, a code formatter might have global style preferences in `~/.config/formatter/formatter.toml` but allow per-project overrides in `./formatter.toml`.

**File Discovery**: When using `MultiScopeConfig`, the library searches for files named `<basename>.{yaml,yml,toml,json,ron,hjson,json5}` in each scope's directory, where `<basename>` is determined by your `app_config_basename()` implementation.

### Composing Configuration Layers

The `LayerBuilder` API provides explicit control over configuration source composition and precedence. This is more powerful than relying on defaults, allowing you to define exactly which sources to load and in what order.

**Available Layer Types**:
- `with_path(path)` - Load from explicit file path
- `with_path_in_dir(dir, basename)` - Discover file by basename in directory (searches for `basename.{yaml,yml,toml,json,ron,hjson,json5}`)
- `with_env_var(var_name)` - Load from path specified in environment variable
- `with_env_search(env, dirs)` - Search directories for environment-specific files (e.g., `production.yaml`)
- `with_secrets(path)` - Load secrets from file
- `with_env_vars(prefix, separator)` - Load from system environment variables
- `with_scopes<T>(scopes)` - Load from multiple configuration scopes (requires `MultiScopeConfig`)

```rust
use settings_loader::{LayerBuilder, LoadingOptions};

impl LoadingOptions for MyOptions {
    fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
        builder
            .with_path_in_dir("config", "base")      // Discovers config/base.{yaml,toml,json,...}
            .with_path("config/production.yaml")     // Environment override
            .with_secrets("secrets/db.yaml")         // Secrets (not in git)
            .with_env_vars("APP", "__")              // ENV var overrides
            // Highest precedence wins
    }
}
```

**Key Insight**: Layers are applied in order, with later layers overriding earlier ones. This gives you complete control over precedence.

---

## Configuration Source Patterns

Real-world applications have diverse configuration needs. Here are proven patterns for common scenarios.

### Pattern 1: Default (12-Factor App)

The simplest pattern for cloud-native applications following [12-factor principles](https://12factor.net/config):

```rust
impl LoadingOptions for MyOptions {
    fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
        builder
            .with_path_in_dir("config", "application")  // Base config (in git)
            .with_env_vars("APP", "__")                  // Runtime overrides
    }
}
```

**Precedence**: Environment variables > Base configuration

**Use Case**: Containerized applications where configuration is primarily environment-driven.

### Pattern 2: Desktop/CLI Application (System → User → Project → Runtime)

A comprehensive pattern for desktop and CLI applications that respect multiple configuration scopes:

```rust
impl MultiScopeConfig for AppSettings {
    const APP_NAME: &'static str = "myapp";
    const ORG_NAME: &'static str = "myorg";
}

impl LoadingOptions for AppOptions {
    fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
        let mut layers = builder;
        
        // System defaults (read-only, managed by package manager)
        if let Some(path) = Self::resolve_path(ConfigScope::System) {
            if path.exists() {
                layers = layers.with_path(path);
            }
        }
        
        // User global preferences
        if let Some(path) = Self::resolve_path(ConfigScope::UserGlobal) {
            if path.exists() {
                layers = layers.with_path(path);
            }
        }
        
        // Project-local configuration
        if let Some(path) = Self::resolve_path(ConfigScope::ProjectLocal) {
            if path.exists() {
                layers = layers.with_path(path);
            }
        }
        
        // Runtime: environment variables
        layers = layers.with_env_vars("APP", "__");
        
        // Secrets (if provided via CLI)
        if let Some(secrets) = &self.secrets {
            layers = layers.with_secrets(secrets);
        }
        
        layers
    }
}
```

**Precedence**: Secrets > Env Vars > Project > User > System

**Use Case**: CLI tools, desktop applications, development tools that need flexible configuration across different contexts.

### Pattern 3: Containerized Web Server (Axum/Docker)

Optimized for containerized deployments where configuration comes primarily from environment and mounted secrets:

```rust
impl LoadingOptions for ServerOptions {
    fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
        builder
            // Baked-in defaults (in container image)
            .with_path("/app/config/defaults.yaml")
            
            // Environment-specific config (mounted volume)
            .with_path("/config/production.yaml")
            
            // Secrets (mounted from secret manager)
            .with_secrets("/run/secrets/database")
            
            // Runtime overrides (Kubernetes env vars, etc.)
            .with_env_vars("APP", "__")
    }
}
```

**Precedence**: Env Vars > Secrets > Mounted Config > Defaults

**Use Case**: Docker/Kubernetes deployments with ConfigMaps, Secrets, and environment variables.

**Deployment Example**:
```yaml
# docker-compose.yml
services:
  api:
    image: myapp:latest
    environment:
      - APP__SERVER__PORT=8080
      - APP__DATABASE__MAX_CONNECTIONS=20
    volumes:
      - ./config/production.yaml:/config/production.yaml:ro
    secrets:
      - database
```

### Pattern 4: Development vs Production

Separate base configuration from environment-specific overrides:

```
config/
  ├── application.yaml      # Base config (version controlled)
  ├── local.yaml           # Local development overrides
  └── production.yaml      # Production overrides
secrets/
  └── database.yaml        # Secrets (NOT in version control)
```

```rust
impl LoadingOptions for MyOptions {
    fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
        let mut layers = builder.with_path("config/application.yaml");
        
        // Add environment-specific config
        if let Some(env) = &self.environment {
            layers = layers.with_path(format!("config/{}.yaml", env));
        }
        
        // Add secrets if available
        if let Some(secrets) = &self.secrets {
            layers = layers.with_secrets(secrets);
        }
        
        // Environment variables override everything
        layers.with_env_vars("APP", "__")
    }
}
```

**Precedence**: Env Vars > Secrets > Environment File > Base Config

---

## Feature Capabilities

### Configuration Editing (`editor` feature)

Applications often need to persist user preferences or update configuration programmatically. Naive file writing loses comments and formatting, frustrating users who maintain carefully documented configs. The `editor` feature solves this by providing bidirectional editing with format preservation, particularly for TOML files where comments are common.

**Core Capability**: The `LayerEditor` trait allows reading and writing individual configuration layers while preserving structure and formatting. **Comment preservation is currently supported for TOML files only** (using `toml_edit`). JSON and YAML editors use standard serde serialization, which does not preserve comments.

```rust
use settings_loader::{SettingsEditor, ConfigScope};

// Edit project-local settings (TOML comments preserved!)
let mut editor = MySettings::editor(ConfigScope::ProjectLocal, &options)?;

// Get current value
let port: u16 = editor.get("server.port")?.unwrap_or(8080);

// Update value (comments preserved!)
editor.set("server.port", 9000)?;
editor.save()?;

// Later: reload and verify
let updated = MySettings::load(&options)?;
assert_eq!(updated.server.port, 9000);
```

**Use Cases**:
- TUI settings panels that let users edit configuration interactively
- Configuration wizards that update files based on user input
- Applications that save user preferences without destroying their comments
- Tools that programmatically update config files (e.g., version bumpers, migration scripts)

### Metadata & Introspection (`metadata` feature, default)

Building UIs, validating configurations, and generating documentation often requires knowing what settings exist, their types, constraints, and defaults. Hardcoding this information in multiple places (code, docs, UI) creates maintenance burden and drift. The `metadata` feature provides a single source of truth: register metadata once, use it everywhere.

**Core Capability**: Register metadata for your settings and automatically generate JSON Schema, HTML documentation, example configs, and validation rules. This metadata can also drive UI generation for TUI/CLI tools.

```rust
use settings_loader::metadata::{SettingMetadata, SettingType, Constraint, Visibility};
use settings_loader::registry;

// Initialize registry
registry::init_global_registry("My App", "1.0.0");

// Register setting metadata
registry::register_setting(SettingMetadata {
    key: "server.port".to_string(),
    label: "Server Port".to_string(),
    description: "The port the HTTP server will listen on.".to_string(),
    setting_type: SettingType::Integer { min: Some(1024), max: Some(65535) },
    default: Some(serde_json::json!(8080)),
    constraints: vec![
        Constraint::Required,
        Constraint::Range { min: 1024.0, max: 65535.0 }
    ],
    visibility: Visibility::Public,
    group: Some("Server".to_string()),
});

// Export JSON Schema
MySettings::export_json_schema("schema.json")?;

// Export HTML documentation
MySettings::export_docs("docs.html")?;

// Export example config
MySettings::export_example_config("application.example.toml")?;
```

**Use Cases**:
- Auto-generating CLI `--help` text from metadata
- Building TUI settings editors that enumerate available options
- Generating JSON Schema for documentation and IDE integration
- Creating configuration validation with meaningful error messages
- Exporting example config files for users
- Generating API documentation for configuration endpoints

See [`examples/schema_generation.rs`](examples/schema_generation.rs) for a complete example.

### Settings Validation

Configuration errors should be caught early with clear, actionable error messages. Waiting until runtime to discover that a port number is invalid or a required field is missing wastes time and creates poor user experience. The validation system provides declarative constraints that are checked automatically during loading.

**Core Capability**: Define constraints on your settings (required, range, pattern, etc.) and get automatic validation with detailed error messages that guide users to fix issues.

```rust
use settings_loader::metadata::Constraint;

// Define constraints
let constraints = vec![
    Constraint::Required,
    Constraint::Range { min: 1024.0, max: 65535.0 },
    Constraint::Pattern { 
        pattern: r"^\d{1,5}$".to_string() 
    },
];

// Validation happens automatically when loading
let settings = MySettings::load(&options)?;
// If validation fails, you get detailed error messages:
// "Setting 'server.port' is out of range: expected 1024-65535, got 80"
```

**Use Cases**:
- Preventing invalid configurations from reaching production
- Providing clear error messages that guide users to fix issues
- Enforcing business rules (e.g., connection pool size limits)
- Validating complex constraints (e.g., URL schemes, file paths)

### Provenance Tracking

When debugging configuration issues in production, you need to know where each value came from. Was it the base config? An environment variable? A secrets file? A user override? Without provenance tracking, you're left guessing or manually checking multiple sources. The provenance system tracks the source of every configuration value.

**Core Capability**: Load settings with full provenance information, allowing you to query the source of any value for debugging, auditing, or understanding configuration precedence.

```rust
use settings_loader::SettingsLoader;

let (settings, sources) = MySettings::load_with_provenance(&options)?;

// Find out where a specific setting came from
if let Some(source) = sources.source_of("database.host") {
    match source.source_type {
        SourceType::File => println!("From file: {:?}", source.path),
        SourceType::Environment => println!("From env var: {}", source.id),
        SourceType::Default => println!("Using default value"),
        _ => {}
    }
}

// Get all settings from a specific scope
let user_settings = sources.all_from_scope(ConfigScope::UserGlobal);
```

**Use Cases**:
- Debugging configuration issues in production
- Creating audit trails for compliance
- Understanding configuration precedence in complex deployments
- Generating configuration reports showing active sources

See [`examples/provenance_audit.rs`](examples/provenance_audit.rs) for a complete example.

---

## Common Patterns

### Environment Variables with File-Based Configs

Combining environment variables with file-based configuration is a cornerstone of cloud-native applications. Environment variables provide runtime flexibility while files provide structure and documentation.

```rust
impl LoadingOptions for MyOptions {
    fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
        builder
            .with_path("config.yaml")
            .with_env_vars("APP", "__")  // APP__DATABASE__HOST overrides database.host
    }
}
```

**Environment variable naming convention**:
- Prefix: `APP` (customizable)
- Separator: `__` (double underscore)
- Example: `APP__DATABASE__HOST=localhost` sets `database.host`

**Precedence**: Environment variables override file-based configuration.

```bash
# Override database host via environment variable
export APP__DATABASE__HOST=prod.db.example.com
export APP__DATABASE__PORT=5432

# Run application (env vars override config files)
cargo run
```

### Secrets Management

Sensitive values like passwords, API keys, and certificates should never be committed to version control. The `secrecy` crate integration ensures secrets are handled safely and redacted in error messages.

```rust
use secrecy::{Secret, ExposeSecret};
use serde::Deserialize;

#[derive(Deserialize)]
struct DatabaseSettings {
    pub host: String,
    pub username: String,
    #[serde(deserialize_with = "deserialize_secret")]
    pub password: Secret<String>,
}

// Secrets are automatically redacted in error messages
impl std::fmt::Debug for DatabaseSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseSettings")
            .field("host", &self.host)
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

// Use the secret
let connection_string = format!(
    "postgres://{}:{}@{}/db",
    settings.database.username,
    settings.database.password.expose_secret(),
    settings.database.host
);
```

### CLI Integration

Command-line arguments should have the highest precedence, allowing users to override any configuration for testing or one-off operations.

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    #[arg(short, long)]
    env: Option<String>,
    
    /// Override database host
    #[arg(long)]
    db_host: Option<String>,
}

impl LoadingOptions for Cli {
    fn load_overrides(&self, config: ConfigBuilder<DefaultState>) 
        -> Result<ConfigBuilder<DefaultState>, Self::Error> 
    {
        let mut config = config;
        
        // Apply CLI overrides
        if let Some(host) = &self.db_host {
            config = config.set_override("database.host", host.clone())?;
        }
        
        Ok(config)
    }
}
```

**Precedence**: CLI arguments > environment variables > files > defaults.

---

## Examples

The [`examples/`](examples/) directory contains complete, runnable examples:

- **[`schema_generation.rs`](examples/schema_generation.rs)**: Demonstrates metadata registration and exporting JSON Schema, HTML documentation, and example TOML configs.
- **[`provenance_audit.rs`](examples/provenance_audit.rs)**: Shows source tracking and debugging configuration by identifying where each value originated.

Run examples with:

```bash
cargo run --example schema_generation --features metadata
cargo run --example provenance_audit
```

---

## Comparison with Alternatives

### Architectural Foundation

**`settings-loader` wraps and builds on [`config-rs`](https://github.com/mehcode/config-rs)**, extending it with additional capabilities rather than replacing it. This means you get all the benefits of `config-rs`'s mature multi-source merging and serde integration, plus the features below.

### What settings-loader Adds

| Feature | config-rs | figment | settings-loader |
|---------|-----------|---------|-----------------|
| Multi-source merging | ✅ | ✅ | ✅ (via config-rs) |
| Serde integration | ✅ | ✅ | ✅ (via config-rs) |
| Multiple formats | ✅ | ✅ | ✅ (via config-rs) |
| **Bidirectional editing** | ❌ | ❌ | ✅ (with comment preservation) |
| **Metadata/introspection** | ❌ | Limited | ✅ (full schema generation) |
| **Multi-scope paths** | Manual | Manual | ✅ (platform-aware via `directories`) |
| **Provenance tracking** | ❌ | ✅ | ✅ (detailed source info) |
| **Opinionated patterns** | ❌ | ❌ | ✅ (12-factor, multi-scope, etc.) |

### When to Use settings-loader

Choose `settings-loader` if you need:
- **Bidirectional editing**: Update config files programmatically while preserving comments
- **UI generation**: Build TUI/CLI settings editors from metadata
- **Multi-scope support**: Handle user-global vs project-local configs automatically
- **Provenance tracking**: Debug configuration by knowing where values came from
- **Opinionated patterns**: 12-factor app support, secrets management, validation

Choose `config-rs` directly if you only need basic multi-source loading and don't require the additional features.

Choose `figment` if you prefer its API style and need its specific features (like typed providers).

---

## Roadmap

### Current State (v1.0.0)

`settings-loader` is production-ready with:
- ✅ Multi-source composition with customizable precedence
- ✅ Multi-format support (YAML, JSON, TOML, HJSON, RON)
- ✅ Type-safe serde integration
- ✅ Multi-scope path resolution (`multi-scope` feature)
- ✅ Bidirectional editing with comment preservation (`editor` feature)
- ✅ Metadata, introspection, and validation (`metadata` feature)
- ✅ Provenance tracking
- ✅ Schema generation (JSON Schema, HTML docs, example configs)
- ✅ Comprehensive test coverage (88%+ mutation score)

### Future Enhancements (Product Roadmap)

Possible future enhancements include configuration hot reload, remote configuration sources (etcd, Consul, AWS Parameter Store), IDE integration via LSP, configuration diffing & migration, validation UI, templates & profiles, observability, encryption at rest, testing framework, and web-based editor.

See [`ref/FUTURE_ENHANCEMENTS.md`](ref/FUTURE_ENHANCEMENTS.md) for detailed descriptions and [`history/CONSOLIDATED_ROADMAP.md`](history/CONSOLIDATED_ROADMAP.md) for technical roadmap.

---

## Contributing

Contributions are welcome! Please:
1. Check existing issues or create a new one
2. Fork the repository
3. Create a feature branch
4. Add tests for new functionality
5. Ensure all tests pass: `cargo test`
6. Submit a pull request

---

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

## Acknowledgments

Built on the excellent [`config-rs`](https://github.com/mehcode/config-rs) library by [@mehcode](https://github.com/mehcode).