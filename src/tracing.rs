//! Provides utilities for setting up structured tracing in tests.
//!
//! This module configures a `tracing` subscriber to capture logs in tests, allowing
//! better debugging and structured log output. It uses `tracing_bunyan_formatter`
//! to format logs as JSON and `tracing_subscriber` to handle filtering and layering.
//!
//! # Features
//! - **Conditional Logging in Tests**: Enables logging if `TEST_LOG` is set.
//! - **Structured Logging**: Uses Bunyan-style JSON formatting.
//! - **Thread-safe Global Subscriber**: Ensures tracing is initialized only once.
//!
//! # Usage
//!
//! To enable logging in your tests, set the `TEST_LOG` environment variable:
//!
//! ```sh
//! TEST_LOG=1 cargo test
//! ```
//!
//! Otherwise, logging will be disabled by default to keep test output clean.
//!
//! # Example
//!
//! ```rust, ignore
//! use tracing::info;
//! use crate::tracing::TEST_TRACING;
//!
//! fn example_test() {
//!     Lazy::force(&TEST_TRACING); // Ensure tracing is initialized
//!     info!("This will appear in logs if TEST_LOG is set");
//! }
//! ```
//!
//! # Components
//! - [`TEST_TRACING`]: Ensures tracing is initialized once per test suite.
//! - [`get_subscriber`]: Creates a new tracing subscriber with JSON formatting.
//! - [`init_subscriber`]: Sets the created subscriber as the global default.

use once_cell::sync::Lazy;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;

use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

/// Ensures tracing is initialized once for the test suite.
///
/// This static initializer sets up a global tracing subscriber only if the `TEST_LOG`
/// environment variable is set. If `TEST_LOG` is missing, logs are suppressed.
///
/// # Behavior:
/// - **Enabled (TEST_LOG is set)**: Logs are written to `stdout`.
/// - **Disabled (TEST_LOG is unset)**: Logs are suppressed using `std::io::sink()`.
#[allow(dead_code)]
pub static TEST_TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info";
    let subscriber_name = "test";
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

/// Creates a `tracing` subscriber with structured JSON logging.
///
/// This function configures a `tracing` subscriber with the following features:
/// - **Environment-based filtering**: Reads from `RUST_LOG`, defaults to provided `env_filter`.
/// - **JSON-formatted output**: Uses `tracing_bunyan_formatter` for structured logging.
/// - **Storage layer**: Adds `JsonStorageLayer` for capturing context values.
///
/// # Arguments
/// - `name`: A name for the logging layer.
/// - `env_filter`: The log level filter (e.g., `"info"`).
/// - `sink`: A writer (e.g., `std::io::stdout` or `std::io::sink()`).
///
/// # Returns
/// A configured `tracing` subscriber.
///
/// # Example
/// ```rust, ignore
/// let subscriber = subscriber("test", "debug", std::io::stdout);
/// init_subscriber(subscriber);
/// ```
#[allow(unused)]
pub fn subscriber<S0, S1, W>(name: S0, env_filter: S1, sink: W) -> impl Subscriber + Sync + Send
where
    S0: Into<String>,
    S1: AsRef<str>,
    W: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

    let formatting_layer = BunyanFormattingLayer::new(name.into(), sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Initializes the given tracing subscriber as the global default.
///
/// This function:
/// - **Redirects logs to `tracing`** using `tracing_log::LogTracer::init()`.
/// - **Sets the global subscriber** using `tracing::subscriber::set_global_default()`.
///
/// # Panics
/// Panics if the subscriber cannot be set (e.g., if tracing was already initialized).
///
/// # Example
/// ```rust, ignore
/// let subscriber = subscriber("test", "info", std::io::stdout);
/// init_subscriber(subscriber);
/// ```
pub fn init_subscriber(subscriber: impl Subscriber + Sync + Send) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
