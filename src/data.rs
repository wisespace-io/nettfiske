use serde::{Deserialize};

#[derive(Deserialize, Debug)]
pub struct CertString {
    pub message_type: String,
    pub data: Data,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub leaf_cert: LeafCert,
}

#[derive(Deserialize, Debug)]
pub struct LeafCert {
    pub subject: Subject,
    pub all_domains: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Subject {
    aggregated: String,
    #[serde(rename = "C")]
    c: Option<String>,
    #[serde(rename = "ST")]
    st: Option<String>,
    #[serde(rename = "L")]
    l: Option<String>,
    #[serde(rename = "O")]
    pub organization: Option<String>,
    #[serde(rename = "OU")]
    pub organizational_unit: Option<String>,
    #[serde(rename = "CN")]
    pub common_name: Option<String>,
}

// Config
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub identities: Vec<WebsiteIdentity>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WebsiteIdentity {
    pub common_name: String,
}

