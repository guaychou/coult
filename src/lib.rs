//! Coult, is crate to getting from hashicorp vault
//! # Usage
//!
//! Coult use hyper client instead of reqwest for more simpler and lightweight crate, it will help you to send GET request to Vault
//! for retrieving the secret. This crate will help you to automatically parsing using serde,
//! and make sure your struct has Deserialize derive.
//!
//! ## Example
//!
//! ```
//! use coult::{Config, Vault};
//! use serde::Deserialize;
//!
//! #[derive(Debug, Deserialize)]
//! struct Secret {
//!    password: String,
//!}
//!
//! #[tokio::main]
//! async fn main() {
//! tracing_subscriber::fmt::init();
//! let config = Config::new(
//!    "127.0.0.1".to_string(),                      # Vault Host
//!     8200,                                        # Port
//!    "config/path".to_string(),                   # Secret Path
//!    "vault-plaintext-root-tokenzqwe".to_string(), # Vault Token
//!    );
//!    let vault = Vault::new(config).await.unwrap();
//!    let data = vault.get_secret::<Secret>().await.unwrap();
//!    println!("{:?}", data)
//!}
//! ```
//!

/// Client instance to get secret from Hashicorp Vault
pub mod client;
/// Config struct to match the address of vault
pub mod config;
/// # HTTP response that maybe happen in Vault
/// - 200	Active Node
/// - 429	Standby Node
/// - 472	Active DR Secondary Node
/// - 473	Standby Performance Node
/// - 501	Uninitialized
/// - 503	Sealed
/// - Filtered only exclude 200
pub mod error;
/// Schema is response struct from hashicorp vault when we hit /v1/secret/path
pub mod schema;
pub use client::Vault;
pub use config::Config;
