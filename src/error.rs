use hyper::StatusCode;
use thiserror::Error;

/// # HTTP response that maybe happen in Vault
/// - 200    Active Node
/// - 404    Invalid Path
/// - 429    Standby Node
/// - 472    Active DR Secondary Node
/// - 473    Standby Performance Node
/// - 501    Uninitialized
/// - 503    Sealed
/// - Filtered only exclude 200

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Vault is sealed, connection to vault failed | status code: {0}")]
    VaultSealed(StatusCode),
    #[error("Vault is not initialized, connection to vault failed | status code: {0}")]
    VaultNotInitialized(StatusCode),
    #[error("Vault is in standby, connection to vault failed | status code: {0}")]
    VaultStandby(StatusCode),
    #[error("Vault is in active DR secondary node, connection to vault failed | status code: {0}")]
    VaultActiveDRsecondaryNode(StatusCode),
    #[error("Vault is in active standby performance node, connection to vault failed | status code: {0}")]
    VaultStandbyPerformanceNode(StatusCode),
    #[error("Your request is going to invalid path | status code: {0}")]
    VaultInvalidPath(StatusCode),
    #[error("Vault error unknown, connection to vault failed | status code: {0}")]
    Unknown(StatusCode),
}
