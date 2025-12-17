# settings-loader-rs

**settings-loader-rs** is a Rust library designed to unify configuration sources from multiple origins—including 
configuration files, command-line arguments, and environment variables—into a single, coherent application 
representation. The primary goal is to decouple configuration sourcing from the application code, enabling 
applications to retrieve configuration values seamlessly without concerning themselves with how or where the data 
originates.

> **Status**: Under active development toward v1.0.0. Currently at v0.15.0. Phase 1 (Explicit Configuration Layering) 
> being completed for v0.16.0. See [Roadmap](#roadmap) for full vision.

# Features
- **Unified Configuration Management** – SettingsLoader::load() consolidates multiple configuration sources into a single 
application representation.
- **Separation of Concerns** – The application code remains agnostic of configuration sources, relying only on the 
resolved settings.
- **Hierarchical Merging** – Supports layering of configurations, ensuring that CLI arguments override environment 
variables, which in turn override file-based configurations.
- **Typed Access** – Retrieve values as strongly typed Rust structures to prevent runtime errors.
- **Extensibility** – Easily extendable to support additional configuration sources if needed.
- **Multi-Format Support** – Supports various file formats for configuration, including:
  - **JSON** (`.json`)
  - **TOML** (`.toml`)
  - **YAML** (`.yaml`, `.yml`)
  - **HJSON** (`.hjson`)
  - **RON** (`.ron`)

## Supported Configuration Sources:
1. **Explicit Configuration Files**: If a specific configuration file path is provided, it is loaded directly.
2. **Implicit Configuration Files**: Searches for configuration files in predefined directories. Uses a default resource 
path if no directories are specified.
3. **Environment-Based Configuration**: Loads additional configuration sources based on the specified environment. Supports 
multiple environment-specific sources.
4. **Secrets Management**: If a secrets file path is provided, it is loaded as an additional configuration source.
5. **Environment Variables**: Loads settings from system environment variables.
6. **Command-Line Overrides**: Allows CLI options to override all other configuration sources.

# Installation
To use `settings-loader-rs` in your Rust project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
settings-loader = "0.14"
```

## Optional Cargo Features
`settings-loader-rs` provides additional feature flags for common settings structures:

- `database` – Includes predefined structures for database configuration settings.
- `http` – Provides common settings for HTTP-based applications.

To enable specific features, modify your Cargo.toml:
```toml
[dependencies]
settings-loader = { version = "0.14", features = ["database", "http"] }
```

## Minimum Rust Version
This library requires Rust 2018 edition or later. 

# Usage
`settings-loader-rs` loads configuration from multiple sources and merges them into a single application representation. 
The sources include:

- **Base configuration file** (application.yaml)
- **Environment-specific configuration files** (production.yaml, local.yaml)
- **Secrets file for sensitive credentials** (secrets.yaml)
- **Environment variables**
- **Command-line arguments**

## Example Configuration
`application.yaml` (Base Configuration)
```yaml
http_api:
  host: 0.0.0.0
  port: 8000
  timeout_secs: 120
  rate_limit:
    burst_size: 8
    per_seconds: 0.5

database:
  host: localhost
  port: 5432
  database_name: weather
  require_ssl: false
  max_connections: 10
  max_lifetime_secs: 1800
  acquire_timeout_secs: 120
  idle_timeout_secs: 300
```

`production.yaml` (Production environment overrides)
```yaml
application:
  host: 0.0.0.0
database:
  host: postgres_1632546102
```

`local.yaml` (Local environment overrides)
```yaml
http_api:
  host: 127.0.0.1
  base_url: "http://127.0.0.1"

database:
  username: postgres
  password: postgres
  require_ssl: false
```

`secrets.yaml` (Secret credentials sourced from a secure repository during deployment - or not used in favor of 
environment variables)
```yaml
database:
  username: my_user_name
  password: my_secret_password
```

## Command-Line Options
You can define a CLI options structure to support including command-line options in setting. For example, define a 
`CliOptions` struct in `settings::cli_options.rs` specifying the available CLI options using the `clap` crate. One role
for `CliOptions` is to configured how settings are loaded:

| Flag                                       | Description                                                                             |
|--------------------------------------------|-----------------------------------------------------------------------------------------|
| `-c, --config <PATH_TO_CONFIG_FILE>`       | Load an explicit configuration file, bypassing inferred configuration.                  |
| `--secrets <PATH_TO_SECRETS_FILE>`         | Specify a path to a secrets configuration file.                                         |
| `-e, --env <ENVIRONMENT>`                  | Specify the environment configuration override (`local`, `production`, etc.).           |
| `-s, --search-path <SETTINGS_SEARCH_PATH>` | Override the filesystem path used to search for configuration files (separated by `:`). |

### Example Usage:
1. Load settings from `local.yaml` as an environment override:
```sh
cargo run -- --env local
```

2. Explicitly specify a configuration file:
```shell
cargo run -- --config ./custom_config.yaml
```

3. Load a secrets file for credentials:
```shell
cargo run -- --secrets ./secrets.yaml
```

4. Override configuration search paths:
```shell
cargo run -- --search-path "./config:./resources"
```

## Example Application Code
Here’s an example of how an application can use `settings-loader-rs` to load configurations dynamically based on 
CLI options:

```rust, ignore
use anyhow::anyhow;
use clap::Parser;
use settings_loader::{LoadingOptions, SettingsLoader};
use my_app::settings::{CliOptions, Settings};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = CliOptions::parse();
    if options.secrets.is_none() {
        tracing::warn!("No secrets configuration provided. Passwords (e.g., for the database) should be confined to a secret configuration and sourced in a secure manner.");
    }
    let settings = load_settings(&options)?;

    // ...  define application that uses Settings ...
}
```

### How Configuration is Loaded
1. **Parsing CLI Options**: `CliOptions::parse()` extracts configuration options from the command line.

2. **Loading Configuration**: `load_settings(&options)` loads and merges settings from:
  - Base configuration (`application.yaml`)
  - Environment-specific configuration (`production.yaml` or `local.yaml`)
  - Secrets file (if provided)
  - Environment variables
  - Command-line arguments

3. **Application Initialization**: the settings object is passed to used by application

### Environment Variables
In addition to configuration files, environment variables can be used to override settings dynamically. The 
`SettingsLoader::load()` function integrates environment-based configuration loading.

#### Example Environment Variables
```shell
export APP_ENVIRONMENT=production
export DATABASE_USERNAME=my_user
export DATABASE_PASSWORD=my_secure_password
export HTTP_API_HOST=192.168.1.100
export HTTP_API_PORT=8080
```

#### How Environment Variables Work:
- The `APP_ENVIRONMENT` variable determines which environment-specific configuration file (`production.yaml`, 
`local.yaml`, etc.) is used.
- Individual settings (like `DATABASE_USERNAME`) override values in `application.yaml` and environment-specific files. 
- The priority order (from lowest to highest) is:
  1. `application.yaml` (base config)
  2. `local.yaml` or `production.yaml` (environment-specific overrides)
  3. Environment variables
  4. Command-line arguments

# Roadmap

## Vision: v1.0.0 Release

`settings-loader-rs` is evolving from a read-only configuration loader into a comprehensive configuration management 
system. The roadmap spans **7 phases** with incremental releases from v0.16.0 to v1.0.0.

**CRITICAL**: All 7 phases must be completed before v1.0.0 release. No partial releases.

### Phase Progress

| Phase | Feature | Version | Status | ETA |
|-------|---------|---------|--------|-----|
| 1 | Explicit Configuration Layering | v0.16.0 | ✅ Implementation Complete (Gate 2 Approved) | Complete |
| 2 | Environment Variable Customization | v0.17.0 | ⏳ Planned | Week 2 |
| 3 | Multi-Scope Paths | v0.18.0 | ⏳ Planned | Week 3 |
| 4 | Configuration Editing | v0.19.0 | ⏳ Planned | Weeks 4-5 |
| 5 | Metadata & Introspection | v0.20.0 | ⏳ Planned | Weeks 6-7 |
| 6 | Source Provenance | v0.21.0 | ⏳ Planned | Weeks 8-9 |
| 7 | Schema Export & Documentation | v1.0.0 | ⏳ Planned | Weeks 10-11 |

### Phase 1: Explicit Configuration Layering (CURRENT)

Enables explicit definition of configuration sources with clear precedence using the `LayerBuilder` API.

```rust
// Example: Define configuration layers
impl LoadingOptions for MyOptions {
    fn build_layers(&self, builder: LayerBuilder) -> LayerBuilder {
        builder
            .with_path("config.yaml")           // Base config
            .with_path("local.yaml")             // Environment override
            .with_secrets("secrets.yaml")        // Secrets layer
            .with_env_vars("APP", "__")          // Env var overrides
    }
}

let settings = MySettings::load(&options)?;
```

**Key Features**:
- `ConfigLayer` enum with 5 layer types
- `LayerBuilder` fluent API
- Backward compatible with implicit layering
- All formats supported (YAML, JSON, TOML, HJSON, RON)
- Tests: 35/35 passing

**Status**: Ready for v0.16.0 release after Gate 3 approval

### Future Phases

See [`history/CONSOLIDATED_ROADMAP.md`](history/CONSOLIDATED_ROADMAP.md) for detailed specifications of phases 2-7.

#### Example Usage with Environment Variables
Run the application with environment variables applied:
```shell
APP_ENVIRONMENT=local DATABASE_USERNAME=custom_user cargo run
```