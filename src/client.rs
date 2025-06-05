use crate::error::VaultError;
use crate::schema::{VaultSchemaV1, VaultSchemaV2};

use bytes::{Buf, Bytes};
use http_body_util::{BodyExt, Empty};
use hyper::{Request, StatusCode, Uri};
use hyper_rustls::HttpsConnectorBuilder;
use hyper_util::client::legacy::{Client, connect::HttpConnector};
use hyper_util::rt::{TokioExecutor, TokioTimer};
use serde::de::DeserializeOwned;
use std::env;
use tokio::time::Duration;
use tracing::{error, info};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type VaultApiResult = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
type VaultHttpClient = Client<hyper_rustls::HttpsConnector<HttpConnector>, Empty<Bytes>>;

/// Represents a client to interact with HashiCorp Vault using HTTP(S).
///
/// Contains configuration and an HTTP client used for making API requests to Vault.
pub struct Vault {
    http_client: VaultHttpClient,
    secret_path: Option<String>,
    address: Option<String>,
    port: Option<u16>,
    token: Option<String>,
    protocol: Option<String>,
}

/// Builder for constructing a `Vault` client instance with custom or environment-configured options.
pub struct VaultBuilder {
    secret_path: Option<String>,
    address: Option<String>,
    port: Option<u16>,
    token: Option<String>,
    protocol: Option<String>,
}

impl VaultBuilder {
    pub fn address(&mut self, address: &str) -> &mut Self {
        self.address = Some(address.to_string());
        self
    }

    pub fn port(&mut self, port: u16) -> &mut Self {
        self.port = Some(port);
        self
    }

    pub fn token(&mut self, token: &str) -> &mut Self {
        self.token = Some(token.to_string());
        self
    }

    pub fn secret_path(&mut self, secret_path: &str) -> &mut Self {
        self.secret_path = Some(secret_path.to_string());
        self
    }

    pub fn https(&mut self) -> &mut Self {
        self.protocol = Some("https".to_string());
        self
    }

    pub fn protocol(&mut self, proto: &str) -> &mut Self {
        self.protocol = Some(proto.to_string());
        self
    }

    /// Builds the `Vault` client using provided values or environment variables.
    ///
    /// Environment fallbacks:
    /// - `VAULT_SECRET_PATH`
    /// - `VAULT_ADDRESS` (default: `"127.0.0.1"`)
    /// - `VAULT_PORT` (default: `8200`)
    /// - `VAULT_TOKEN`
    /// - `VAULT_PROTOCOL` (default: `"http"`)
    ///
    /// Returns an error if required values are missing or health check fails.
    pub async fn build(&mut self) -> Result<Vault> {
        self.secret_path.get_or_insert_with(|| {
            env::var("VAULT_SECRET_PATH").expect("Set secret_path or VAULT_SECRET_PATH")
        });

        self.address.get_or_insert_with(|| {
            env::var("VAULT_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string())
        });

        self.port.get_or_insert_with(|| {
            env::var("VAULT_PORT")
                .unwrap_or_else(|_| "8200".to_string())
                .parse::<u16>()
                .unwrap()
        });

        self.token
            .get_or_insert_with(|| env::var("VAULT_TOKEN").expect("Set token or VAULT_TOKEN env"));

        self.protocol.get_or_insert_with(|| {
            env::var("VAULT_PROTOCOL").unwrap_or_else(|_| "http".to_string())
        });

        let https_connector = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_or_http()
            .enable_http1()
            .build();

        let client = Client::builder(TokioExecutor::new())
            .pool_timer(TokioTimer::new())
            .pool_idle_timeout(Duration::from_secs(30))
            .build(https_connector);

        let vault = Vault {
            http_client: client,
            secret_path: self.secret_path.clone(),
            address: self.address.clone(),
            port: self.port,
            token: self.token.clone(),
            protocol: self.protocol.clone(),
        };

        vault.health_check().await?;

        info!(
            "Vault health check success for {}",
            vault.address.as_ref().unwrap()
        );

        Ok(vault)
    }
}

impl Vault {
    pub fn new() -> VaultBuilder {
        VaultBuilder {
            secret_path: None,
            address: None,
            port: None,
            token: None,
            protocol: None,
        }
    }

    // Performs a health check against the configured Vault server.
    ///
    /// This method sends a `GET` request to the `/v1/sys/health` endpoint of the Vault server,
    /// which provides status information about the Vault node (e.g., whether it is initialized,
    /// unsealed, and active).
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the Vault server responds with a healthy status (`200 OK`).
    /// - `Err(...)` with a `VaultError` if the response indicates an unhealthy or non-operational state,
    ///   such as being sealed, a standby node, or uninitialized.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The Vault URI is invalid.
    /// - The HTTP request fails.
    /// - The Vault server returns a non-200 status code that maps to a known or unknown `VaultError`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let vault = Vault::new()
    ///     .address("127.0.0.1".to_string())
    ///     .port(8200)
    ///     .token("my-token".to_string())
    ///     .https()
    ///     .build()
    ///     .await?;
    ///
    /// vault.health_check().await?;
    /// ```
    ///
    /// # Vault Status Codes
    ///
    /// - `200 OK`: Vault is initialized and unsealed.
    /// - `503`: Vault is sealed.
    /// - `472`: Vault is a standby performance secondary node.
    /// - `473`: Vault is a standby performance node.
    /// - `501`: Vault is not initialized.
    /// - `404`: Invalid path (possibly misconfigured secret path).
    pub async fn health_check(&self) -> VaultApiResult {
        let url = format!(
            "{}://{}:{}/v1/sys/health",
            self.protocol.as_ref().unwrap(),
            self.address.as_ref().unwrap(),
            self.port.unwrap()
        )
        .parse::<Uri>()?;

        let req = Request::builder()
            .method("GET")
            .uri(url)
            .header("content-type", "application/json")
            .header("X-Vault-Token", self.token.as_ref().unwrap())
            .body(Empty::<Bytes>::new())?;

        let res = self.http_client.request(req).await?;
        check_vault_error(res.status())
    }

    pub async fn get_secret<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let uri = format!(
            "{}://{}:{}/v1/{}",
            self.protocol.as_ref().unwrap(),
            self.address.as_ref().unwrap(),
            self.port.unwrap(),
            self.secret_path.as_ref().unwrap()
        )
        .parse::<Uri>()?;

        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("content-type", "application/json")
            .header("X-Vault-Token", self.token.as_ref().unwrap())
            .body(Empty::<Bytes>::new())?;

        let res = self.http_client.request(req).await?;
        check_vault_error(res.status())?;
        info!("Retrieved secret from vault");

        let secret: VaultSchemaV1<T> =
            serde_json::from_reader(res.collect().await?.aggregate().reader())?;
        Ok(secret.data)
    }

    pub async fn get_secret_v2<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let uri = format!(
            "{}://{}:{}/v1/{}",
            self.protocol.as_ref().unwrap(),
            self.address.as_ref().unwrap(),
            self.port.unwrap(),
            self.secret_path.as_ref().unwrap()
        )
        .parse::<Uri>()?;

        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .header("content-type", "application/json")
            .header("X-Vault-Token", self.token.as_ref().unwrap())
            .body(Empty::<Bytes>::new())?;

        let res = self.http_client.request(req).await?;
        check_vault_error(res.status())?;
        info!("Retrieved secret v2 from vault");

        let secret: VaultSchemaV2<T> =
            serde_json::from_reader(res.collect().await?.aggregate().reader())?;
        Ok(secret.data.data)
    }
}

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
