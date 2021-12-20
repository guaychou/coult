use crate::config::Config;
use crate::error::VaultError;
use crate::schema::VaultSchemaV1;
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::http::StatusCode;
use hyper::{Body, Client, Request, Uri};
use serde::de::DeserializeOwned;
use tracing::{error, info};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type VaultApiResult = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Initialize Vault Instance
pub struct Vault {
    pub http_client: Client<HttpConnector>,
    pub config: Config,
}

/// Implementing the Vault instance and do some health check
impl Vault {
    pub async fn new(config: Config) -> Result<Vault> {
        let client = Client::new();
        let vault = Self {
            http_client: client,
            config: config,
        };
        vault.health_check().await?;
        info!(
            "Health check connection to vault in {} success",
            vault.config.address
        );
        Ok(vault)
    }

    /// Health check is using hyper to get health check
    pub async fn health_check(&self) -> VaultApiResult {
        let vault_health_check = format!(
            "{}://{}:{}/v1/sys/health",
            self.config.protocol, self.config.address, self.config.port
        )
        .parse::<Uri>()
        .unwrap();
        let health_req = Request::builder()
            .method("GET")
            .uri(vault_health_check)
            .header("content-type", "application/json")
            .header("X-Vault-Token", self.config.token.as_str())
            .body(Body::empty())?;
        let res = self.http_client.request(health_req).await?;
        check_vault_error(res.status())
    }

    /// Getting secret from vault
    /// It will use generic type and auto parsing into struct
    pub async fn get_secret<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let address = format!(
            "{}://{}:{}/v1/secret/{}",
            self.config.protocol, self.config.address, self.config.port, self.config.config_path
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
        check_vault_error(res.status())?;
        info!("Retrieval secret from vault success");
        let body = hyper::body::aggregate(res).await?;
        let secret: VaultSchemaV1<T> = serde_json::from_reader(body.reader())?;
        Ok(secret.data)
    }
}

/// Error mapping for Vault API
fn check_vault_error(status_code: StatusCode) -> VaultApiResult {
    match status_code.as_u16() {
        200 => Ok(()),
        503 => {
            let err = Box::new(VaultError::VaultSealed(status_code));
            error!("{}", err.to_string());
            Err(err)
        }
        429 => {
            let err = Box::new(VaultError::VaultSealed(status_code));
            error!("{}", err.to_string());
            Err(err)
        }
        472 => {
            let err = Box::new(VaultError::VaultActiveDRsecondaryNode(status_code));
            error!("{}", err.to_string());
            Err(err)
        }
        473 => {
            let err = Box::new(VaultError::VaultStandbyPerformanceNode(status_code));
            error!("{}", err.to_string());
            Err(err)
        }
        404 => {
            let err = Box::new(VaultError::VaultInvalidPath(status_code));
            error!("{}", err.to_string());
            Err(err)
        }
        501 => {
            let err = Box::new(VaultError::VaultNotInitialized(status_code));
            error!("{}", err.to_string());
            Err(err)
        }
        _ => {
            let err = Box::new(VaultError::VaultNotInitialized(status_code));
            error!("{}", err.to_string());
            Err(err)
        }
    }
}
