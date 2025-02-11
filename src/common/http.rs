//! Provides HTTP server configuration settings.
//!
//! This module defines the [`HttpServerSettings`] struct, which encapsulates
//! the host and port configuration for an HTTP server.
//!
//! # Features
//! - **Enabled via Cargo feature**: Available only when the `http` feature is enabled.
//! - **Flexible Address Formatting**: Provides utility methods to construct addresses and URLs.
//! - **Structured Serialization**: Supports `Serialize` and `Deserialize` for easy configuration management.
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use my_crate::http::HttpServerSettings;
//!
//! let server_settings = HttpServerSettings {
//!     host: "127.0.0.1".to_string(),
//!     port: 8080,
//! };
//!
//! assert_eq!(server_settings.address(), "127.0.0.1:8080");
//! assert_eq!(server_settings.url("http").unwrap().as_str(), "http://127.0.0.1:8080");
//! ```

use serde::{Deserialize, Serialize};
use url::{Host, Url};

/// Represents HTTP server configuration settings.
///
/// This struct holds the host and port settings for an HTTP server,
/// and provides utility methods to format and retrieve URLs.
///
/// # Example
///
/// ```rust,ignore
/// let settings = HttpServerSettings {
///     host: "localhost".to_string(),
///     port: 8080,
/// };
///
/// assert_eq!(settings.address(), "localhost:8080");
/// assert_eq!(settings.url("https").unwrap().as_str(), "https://localhost:8080");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpServerSettings {
    /// The hostname or IP address where the HTTP server will be bound.
    pub host: String,

    /// The port on which the HTTP server will listen.
    pub port: u16,
}

impl HttpServerSettings {
    /// Returns the server's address as a `host:port` string.
    ///
    /// # Example
    /// ```rust,ignore
    /// let settings = HttpServerSettings {
    ///     host: "127.0.0.1".to_string(),
    ///     port: 8000,
    /// };
    /// assert_eq!(settings.address(), "127.0.0.1:8000");
    /// ```
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Parses the `host` field into a `url::Host` type.
    ///
    /// # Errors
    /// Returns a `url::ParseError` if the host is invalid.
    ///
    /// # Example
    /// ```rust,ignore
    /// let settings = HttpServerSettings {
    ///     host: "localhost".to_string(),
    ///     port: 8080,
    /// };
    /// assert!(settings.url_host().is_ok());
    /// ```
    pub fn url_host(&self) -> Result<Host, url::ParseError> {
        Host::parse(self.host.as_str())
    }

    /// Constructs a full URL using the given scheme (e.g., `http` or `https`).
    ///
    /// # Errors
    /// Returns a `url::ParseError` if the URL cannot be constructed.
    ///
    /// # Example
    /// ```rust,ignore
    /// let settings = HttpServerSettings {
    ///     host: "example.com".to_string(),
    ///     port: 443,
    /// };
    ///
    /// let url = settings.url("https").unwrap();
    /// assert_eq!(url.as_str(), "https://example.com:443");
    /// ```
    pub fn url(&self, scheme: impl Into<String>) -> Result<Url, url::ParseError> {
        let url_rep = format!("{}://{}:{}", scheme.into(), self.host, self.port);
        Url::parse(url_rep.as_str())
    }
}

#[cfg(test)]
mod tests {
    use assert_matches2::assert_let;
    use pretty_assertions::assert_eq;
    use trim_margin::MarginTrimmable;

    use super::*;

    #[test]
    fn test_url_host() {
        let local = HttpServerSettings { host: "127.0.0.1".to_string(), port: 80 };
        assert_let!(Ok(actual) = local.url_host());
        assert_let!(Ok(expected) = Host::parse("127.0.0.1"));
        assert_eq!(actual, expected);

        let example = HttpServerSettings { host: "example.com".to_string(), port: 8080 };
        assert_let!(Ok(actual) = example.url_host());
        assert_let!(Ok(expected) = Host::parse("example.com"));
        assert_eq!(actual, expected);

        let dns = HttpServerSettings { host: "job_manager".to_string(), port: 8080 };
        assert_let!(Ok(actual) = dns.url_host());
        assert_let!(Ok(expected) = Host::parse("job_manager"));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_http_settings_ser() {
        let settings = HttpServerSettings { host: "example.com".to_string(), port: 80 };
        assert_let!(Ok(yaml) = serde_yaml::to_string(&settings));
        assert_eq!(
            yaml,
            r##"|host: example.com
                |port: 80
                |"##
            .trim_margin()
            .unwrap()
        );

        assert_let!(Ok(json) = serde_json::to_string(&settings));
        assert_eq!(json, r##"{"host":"example.com","port":80}"##.trim_margin().unwrap());

        assert_let!(Ok(from_yaml) = serde_yaml::from_str::<HttpServerSettings>(yaml.as_str()));
        assert_eq!(from_yaml, settings);

        assert_let!(Ok(from_json) = serde_json::from_str::<HttpServerSettings>(json.as_str()));
        assert_eq!(from_json, settings);
    }

    #[test]
    fn test_url() {
        let local = HttpServerSettings { host: "127.0.0.1".to_string(), port: 80 };
        assert_let!(Ok(actual) = local.url("https"));
        assert_let!(Ok(expected) = Url::parse("https://127.0.0.1:80"));
        assert_eq!(actual, expected);

        let example = HttpServerSettings { host: "example.com".to_string(), port: 8080 };
        assert_let!(Ok(actual) = example.url("http"));
        assert_let!(Ok(expected) = Url::parse("http://example.com:8080"));
        assert_eq!(actual, expected);

        let dns = HttpServerSettings { host: "job_manager".to_string(), port: 8888 };
        assert_let!(Ok(actual) = dns.url("https"));
        assert_let!(Ok(expected) = Url::parse("https://job_manager:8888"));
        assert_eq!(actual, expected);
    }
}
