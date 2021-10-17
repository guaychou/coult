use crate::config::Config;
use crate::error::VaultError;
use crate::schema::VaultSchemaV1;
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::{Body, Client, Request, Uri};
use serde::de::DeserializeOwned;
use tracing::{error, info};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type HealthResult = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub struct Vault {
    pub http_client: Client<HttpConnector>,
    pub config: Config,
}

impl Vault {
    pub async fn new(config: Config) -> Result<Vault> {
        let client = Client::new();
        let vault = Self {
            http_client: client,
            config: config,
        };
        vault.health_check().await?;
        Ok(vault)
    }

    pub async fn health_check(&self) -> HealthResult {
        let vault_health_check = format!(
            "http://{}:{}/v1/sys/health",
            self.config.address, self.config.port
        )
        .parse::<Uri>()
        .unwrap();
        let health_req = Request::builder()
            .method("GET")
            .uri(vault_health_check)
            .header("content-type", "application/json")
            .header("X-Vault-Token", self.config.token)
            .body(Body::empty())?;
        let res = self.http_client.request(health_req).await?;
        match res.status().as_u16() {
            200 => {
                info!("Vault Connection Success");
                Ok(())
            }
            503 => {
                let err = Box::new(VaultError::VaultSealed(res.status()));
                error!("{}", err.to_string());
                Err(err)
            }
            429 => {
                let err = Box::new(VaultError::VaultSealed(res.status()));
                error!("{}", err.to_string());
                Err(err)
            }
            472 => {
                let err = Box::new(VaultError::VaultActiveDRsecondaryNode(res.status()));
                error!("{}", err.to_string());
                Err(err)
            }
            473 => {
                let err = Box::new(VaultError::VaultStandbyPerformanceNode(res.status()));
                error!("{}", err.to_string());
                Err(err)
            }
            501 => {
                let err = Box::new(VaultError::VaultNotInitialized(res.status()));
                error!("{}", err.to_string());
                Err(err)
            }
            _ => {
                let err = Box::new(VaultError::VaultNotInitialized(res.status()));
                error!("{}", err.to_string());
                Err(err)
            }
        }
    }

    pub async fn get_secret<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let address = format!(
            "http://{}:{}/v1/{}",
            self.config.address, self.config.port, self.config.config_path
        )
        .parse::<Uri>()
        .unwrap();
        let req = Request::builder()
            .method("GET")
            .uri(address)
            .header("content-type", "application/json")
            .header("X-Vault-Token", self.config.token)
            .body(Body::empty())?;
        let res = self.http_client.request(req).await?;
        let body = hyper::body::aggregate(res).await?;
        let secret: VaultSchemaV1<T> = serde_json::from_reader(body.reader())?;
        Ok(secret.data)
    }
}
