use elasticsearch::{
    auth::Credentials as ESCredentials,
    cert::CertificateValidation,
    http::{
        transport::{SingleNodeConnectionPool, TransportBuilder},
        Url,
    },
    Elasticsearch,
};

use serde_derive::Deserialize;

use crate::framework::{BootstrapResult, SinkConfig, StageReceiver};

use super::run::writer_loop;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum CredentialsConfig {
    Basic { username: String, password: String },
}

impl From<&CredentialsConfig> for ESCredentials {
    fn from(other: &CredentialsConfig) -> Self {
        match other {
            CredentialsConfig::Basic { username, password } => {
                ESCredentials::Basic(username.clone(), password.clone())
            }
        }
    }
}

#[derive(Default, Debug, Deserialize)]
pub struct Config {
    url: String,
    index: String,
    credentials: Option<CredentialsConfig>,

    #[serde(default)]
    idempotency: bool,
}

impl SinkConfig for Config {
    fn bootstrap(&self, input: StageReceiver) -> BootstrapResult {
        let pool = SingleNodeConnectionPool::new(Url::parse(&self.url)?);
        let mut transport =
            TransportBuilder::new(pool).cert_validation(CertificateValidation::None);

        if let Some(creds) = &self.credentials {
            transport = transport.auth(creds.into());
        };

        let client = Elasticsearch::new(transport.build()?);

        let index = self.index.clone();
        let idempotency = self.idempotency;
        let handle = std::thread::spawn(move || {
            writer_loop(input, client, index, idempotency).expect("writer loop failed")
        });

        Ok(handle)
    }
}