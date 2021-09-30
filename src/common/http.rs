use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpServerSettings {
    pub host: String,
    #[serde_as(as = "DisplayFromStr")]
    pub port: u16,
}
