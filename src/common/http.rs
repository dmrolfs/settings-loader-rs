use std::net::{AddrParseError, IpAddr};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpServerSettings {
    pub host: String,
    #[serde_as(as = "DisplayFromStr")]
    pub port: u16,
}

impl HttpServerSettings {
    pub fn ip_address(&self) -> Result<IpAddr, AddrParseError> {
        IpAddr::from_str(self.host)
    }
}
