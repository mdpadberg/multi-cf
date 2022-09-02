use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Debug, Deserialize, Serialize, Clone, Tabled)]
pub struct Environment {
    pub name: String,
    pub url: String,
    pub sso: bool,
    pub skip_ssl_validation: bool,
}