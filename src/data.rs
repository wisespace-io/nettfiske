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
        issued_by: "".to_string(),
        issued_to: "".to_string()
    }
}