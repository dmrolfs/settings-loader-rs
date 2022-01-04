use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use url::{Host, Url};

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpServerSettings {
    pub host: String,
    #[serde_as(as = "DisplayFromStr")]
    pub port: u16,
}

impl HttpServerSettings {
    pub fn url_host(&self) -> Result<Host, url::ParseError> {
        Host::parse(self.host.as_str())
    }

    pub fn url(&self, scheme: impl Into<String>) -> Result<Url, url::ParseError> {
        let url_rep = format!("{}://{}:{}", scheme.into(), self.host, self.port);
        Url::parse(url_rep.as_str())
    }
}

#[cfg(test)]
mod tests {
    use claim::*;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_url_host() {
        let local = HttpServerSettings { host: "127.0.0.1".to_string(), port: 80 };
        let actual = assert_ok!(local.url_host());
        let expected = assert_ok!(Host::parse("127.0.0.1"));
        assert_eq!(actual, expected);

        let example = HttpServerSettings { host: "example.com".to_string(), port: 8080 };
        let actual = assert_ok!(example.url_host());
        let expected = assert_ok!(Host::parse("example.com"));
        assert_eq!(actual, expected);

        let dns = HttpServerSettings { host: "job_manager".to_string(), port: 8080 };
        let actual = assert_ok!(dns.url_host());
        let expected = assert_ok!(Host::parse("job_manager"));
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_url() {
        let local = HttpServerSettings { host: "127.0.0.1".to_string(), port: 80 };
        let actual = assert_ok!(local.url("https"));
        let expected = assert_ok!(Url::parse("https://127.0.0.1:80"));
        assert_eq!(actual, expected);

        let example = HttpServerSettings { host: "example.com".to_string(), port: 8080 };
        let actual = assert_ok!(example.url("http"));
        let expected = assert_ok!(Url::parse("http://example.com:8080"));
        assert_eq!(actual, expected);

        let dns = HttpServerSettings { host: "job_manager".to_string(), port: 8888 };
        let actual = assert_ok!(dns.url("https"));
        let expected = assert_ok!(Url::parse("https://job_manager:8888"));
        assert_eq!(actual, expected);
    }
}
