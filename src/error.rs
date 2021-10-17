use hyper::StatusCode;
use thiserror::Error;

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
    #[error("Vault error unknown, connection to vault failed | status code: {0}")]
    Unknown(StatusCode),
}
