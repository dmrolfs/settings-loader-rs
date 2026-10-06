//! Regression test for the password-leak bug fixed in 1.2.1.
//!
//! `SettingsLoader::load_implicit` used to log the fully-merged, pre-deserialization
//! `config::Config` at `info` level (`tracing::info!(?config, "configuration loaded")`).
//! That value holds every configuration source -- including a secrets file -- as plain,
//! untyped strings, before `try_deserialize` ever converts it into the caller's own typed
//! `Self`. A real database password, loaded through `secrets_path()`, printed in full to this
//! crate's own log output regardless of how carefully the caller's own settings struct typed
//! its secret fields.
//!
//! This test proves the fix holds: a secret value loaded through `secrets_path()` and
//! deserialized by `load_implicit` never appears, in any form, in the `tracing` output this
//! crate's own loading path produces.

use serde::{Deserialize, Serialize};
use settings_loader::{LoadingOptions, SettingsLoader};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::MakeWriter;

/// A password-shaped field, deliberately a bare `String` -- the leak this test guards against
/// lived entirely in this crate's own pre-deserialization logging, independent of how the
/// caller's own struct types its fields. A caller should still prefer `secrecy::SecretString`
/// in real code (see `SettingsLoader`'s own "Security" doc section); this test is not that
/// caller-side guarantee, it is this crate's own guarantee that it never prints the raw merged
/// config regardless.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
struct SecretBearingConfig {
    #[serde(default)]
    service_name: String,
    #[serde(default)]
    database_url: String,
}

impl SettingsLoader for SecretBearingConfig {
    type Options = SecretBearingOptions;
}

#[derive(Debug, Clone)]
struct SecretBearingOptions {
    secrets_path: PathBuf,
}

impl LoadingOptions for SecretBearingOptions {
    type Error = settings_loader::SettingsError;

    fn config_path(&self) -> Option<PathBuf> {
        None
    }

    fn secrets_path(&self) -> Option<PathBuf> {
        Some(self.secrets_path.clone())
    }

    fn implicit_search_paths(&self) -> Vec<PathBuf> {
        Vec::new()
    }
}

/// A `MakeWriter` that appends every write to a shared, in-memory buffer, so the test can
/// inspect exactly what this crate's own `tracing` calls produced -- not whether an assertion
/// about a specific log line's absence happens to pass, but whether the raw secret substring
/// appears anywhere in the captured output at all.
#[derive(Clone)]
struct CapturingWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl std::io::Write for CapturingWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for CapturingWriter {
    type Writer = CapturingWriter;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[test]
fn load_implicit_never_logs_the_raw_secret_value() {
    let temp_dir = tempfile::tempdir().unwrap();
    let secrets_path = temp_dir.path().join("secrets.yaml");

    // A value shaped like a real credential, distinctive enough that an accidental substring
    // match elsewhere in the log output (crate names, field names, etc.) cannot produce a
    // false pass.
    const RAW_SECRET: &str = "postgres://lgw:ZqX9-REGRESSION-CANARY-7f3a@db.example.invalid:5432/lgw_ledger";

    fs::write(
        &secrets_path,
        format!("service_name: \"canary-service\"\ndatabase_url: \"{RAW_SECRET}\"\n"),
    )
    .unwrap();

    let options = SecretBearingOptions { secrets_path };

    let buffer = Arc::new(Mutex::new(Vec::new()));
    let writer = CapturingWriter { buffer: buffer.clone() };
    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_max_level(tracing::Level::TRACE)
        .finish();

    let loaded = tracing::subscriber::with_default(subscriber, || {
        SecretBearingConfig::load_implicit(&options).expect("load_implicit should succeed")
    });

    // The value loaded correctly (proves the secrets file was actually read and merged, not
    // skipped) --
    assert_eq!(loaded.database_url, RAW_SECRET);
    assert_eq!(loaded.service_name, "canary-service");

    // -- but never appeared in this crate's own log output, at any level this test enabled.
    let captured = String::from_utf8(buffer.lock().unwrap().clone()).expect("log output should be valid utf-8");
    assert!(
        !captured.contains(RAW_SECRET),
        "the raw secret value appeared in load_implicit's own tracing output -- the 1.2.1 fix \
         regressed. Captured output:\n{captured}"
    );
    assert!(
        !captured.contains("ZqX9-REGRESSION-CANARY-7f3a"),
        "a substring of the raw secret value appeared in load_implicit's own tracing output. \
         Captured output:\n{captured}"
    );
}
