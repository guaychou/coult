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
    "http".to_string(),                           # Vault Http Protocol http/https
    "127.0.0.1".to_string(),                      # Vault Host
     8200,                                        # Port
    "config/path".to_string(),                    # Secret Path
    "vault-plaintext-root-tokenzqwe".to_string(), # Vault Token
    );
    let vault = Vault::new(config).await.unwrap();
    let data = vault.get_secret::<Secret>().await.unwrap();
    println!("{:?}", data)
}

```

