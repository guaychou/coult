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
//!   tracing_subscriber::fmt::init();
//!   let vault = Vault::new().build().await.unwrap();
//!   let data = vault.get_secret_v2::<Secret>().await.unwrap(); // for v1, get_secret_v2
//!   println!("{:?}", data)
//!}
//! ```
//!

/// Client instance to get secret from Hashicorp Vault
pub mod client;
/// # HTTP response that maybe happen in Vault
/// - 200    Active Node
/// - 404    Invalid Path
/// - 429    Standby Node
/// - 472    Active DR Secondary Node
/// - 473    Standby Performance Node
/// - 501    Uninitialized
/// - 503    Sealed
/// - Filtered only exclude 200
pub mod error;
/// Schema is response struct from hashicorp vault when we hit /v1/secret/path
pub mod schema;
pub use client::Vault;
