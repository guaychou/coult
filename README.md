# Coult

Rust vault secret retriever

Example

```rust
use coult::{Config, Vault};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct Secret {
    password: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::new(
    "127.0.0.1",                      # Vault Host
     8200,                            # Port
    "config/anjim",                   # Secret Path
    "vault-plaintext-root-tokenzqwe", # Vault Token
    );
    let vault = Vault::new(config).await.unwrap();
    let data = vault.get_secret::<Secret>().await.unwrap();
    println!("{:?}", data)
}

```

