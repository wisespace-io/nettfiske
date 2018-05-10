
#[derive(Deserialize, Debug)]
pub struct CertString {
    pub message_type: String,
    pub data: Data,
}

#[derive(Deserialize, Debug)]
pub struct Data {
    pub leaf_cert: LeafCert,
    pub chain: Vec<ChainObjects>
}

#[derive(Deserialize, Debug)]
pub struct LeafCert {
    pub subject: Subject,
    pub all_domains: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChainObjects {
    pub subject: Subject,
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
    pub common_name: String
}


// Config
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub identities : Vec<WebsiteIdentity>
}

#[derive(Deserialize, Debug, Clone)]
pub struct WebsiteIdentity {
   pub common_name: String,
   #[serde(default = "default_certificate")]
   pub certificate: Certificate
}

#[derive(Deserialize, Debug, Clone)]
pub struct Certificate {
    pub issued_to: String,
    pub issued_by: String
}

fn default_certificate() -> Certificate {
    Certificate {
        issued_to: "".to_string(),
        issued_by: "".to_string()
    }
}