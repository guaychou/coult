#![allow(dead_code)]

use crate::error::VaultError;
use crate::schema::{VaultSchemaV1, VaultSchemaV2};
use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::http::StatusCode;
use hyper::{Body, Client, Request, Uri};
use serde::de::DeserializeOwned;
use std::env;
use tracing::{error, info};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type VaultApiResult = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Initialize Vault Instance
pub struct Vault {
    http_client: Client<HttpConnector>,
    secret_path: Option<String>,
    address: Option<String>,
    port: Option<u16>,
    token: Option<String>,
    protocol: Option<String>,
}

pub struct VaultBuilder {
    http_client: Client<HttpConnector>,
    secret_path: Option<String>,
    address: Option<String>,
    port: Option<u16>,
    token: Option<String>,
    protocol: Option<String>,
}

impl VaultBuilder {
    fn address(&mut self, address: String) -> &mut Self {
        self.address = Some(address);
        self
    }
    fn port(&mut self, port: u16) -> &mut Self {
        self.port = Some(port);
        self
    }
    fn token(&mut self, token: String) -> &mut Self {
        self.token = Some(token);
        self
    }
    fn https(&mut self) -> &mut Self {
        self.protocol = Some("https".to_string());
        self
    }

    async fn build(&mut self) -> Result<Vault> {
        if self.secret_path.is_none() {
            self.secret_path = Some(
                self.secret_path
                    .as_ref()
                    .expect("Set the secret path with secret_path function")
                    .to_owned(),
            )
        }
        if self.address.is_none() {
            self.address = Some(env::var("VAULT_ADDR").unwrap_or_else(|_| "127.0.0.1".to_owned()))
        }
        if self.port.is_none() {
            self.port = Some(
                env::var("VAULT_PORT")
                    .unwrap_or_else(|_| "8200".to_owned())
                    .parse::<u16>()
                    .unwrap(),
            )
        }
        if self.token.is_none() {
            self.token = Some(env::var("VAULT_TOKEN").expect(
                "Please set the vault token from token method or VAULT_TOKEN environment variable",
            ));
        }
        if self.protocol.is_none() {
            self.protocol = Some(env::var("VAULT_PROTOCOL").unwrap_or_else(|_| "http".to_owned()));
        }

        let vault = Vault {
            http_client: self.http_client.clone(),
            secret_path: self.secret_path.clone(),
            address: self.address.clone(),
            port: self.port,
            token: self.token.clone(),
            protocol: self.protocol.clone(),
        };
        vault.health_check().await?;
        info!(
            "Health check connection to vault in {} success",
            vault.address.as_ref().unwrap()
        );
        Ok(vault)
    }
}

/// Implementing the Vault instance and do some health check
impl Vault {
    fn new() -> VaultBuilder {
        VaultBuilder {
            http_client: Client::new(),
            secret_path: None,
            address: None,
            port: None,
            token: None,
            protocol: None,
        }
    }
    /// Health check is using hyper to get health check
    pub async fn health_check(&self) -> VaultApiResult {
        let vault_health_check = format!(
            "{}://{}:{}/v1/sys/health",
            self.protocol.as_ref().unwrap(),
            self.address.as_ref().unwrap(),
            self.port.unwrap()
        )
        .parse::<Uri>()
        .unwrap();
        let health_req = Request::builder()
            .method("GET")
            .uri(vault_health_check)
            .header("content-type", "application/json")
            .header("X-Vault-Token", self.token.as_ref().unwrap().as_str())
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
            "{}://{}:{}/v1/{}",
            self.protocol.as_ref().unwrap(),
            self.address.as_ref().unwrap(),
            self.port.unwrap(),
            self.secret_path.as_ref().unwrap(),
        )
        .parse::<Uri>()
        .unwrap();
        let req = Request::builder()
            .method("GET")
            .uri(address)
            .header("content-type", "application/json")
            .header("X-Vault-Token", self.token.as_ref().unwrap())
            .body(Body::empty())?;
        let res = self.http_client.request(req).await?;
        check_vault_error(res.status())?;
        info!("Retrieval secret from vault success");
        let body = hyper::body::aggregate(res).await?;
        let secret: VaultSchemaV1<T> = serde_json::from_reader(body.reader())?;
        Ok(secret.data)
    }

    pub async fn get_secret_v2<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let address = format!(
            "{}://{}:{}/v1/{}",
            self.protocol.as_ref().unwrap(),
            self.address.as_ref().unwrap(),
            self.port.unwrap(),
            self.secret_path.as_ref().unwrap(),
        )
        .parse::<Uri>()
        .unwrap();
        let req = Request::builder()
            .method("GET")
            .uri(address)
            .header("content-type", "application/json")
            .header("X-Vault-Token", self.token.as_ref().unwrap())
            .body(Body::empty())?;
        let res = self.http_client.request(req).await?;
        check_vault_error(res.status())?;
        info!("Retrieval secret from vault success");
        let body = hyper::body::aggregate(res).await?;
        let secret: VaultSchemaV2<T> = serde_json::from_reader(body.reader())?;
        Ok(secret.data.data)
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
            let err = Box::new(VaultError::Unknown(status_code));
            error!("{}", err.to_string());
            Err(err)
        }
    }
}
