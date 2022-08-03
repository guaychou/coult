use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VaultSchemaV1<T> {
    pub request_id: String,
    pub lease_id: String,
    pub renewable: bool,
    pub lease_duration: u32,
    pub data: T,
    pub wrap_info: Option<String>,
    pub warnings: Option<String>,
    pub auth: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VaultSchemaV2<T> {
    pub request_id: String,
    pub lease_id: String,
    pub renewable: bool,
    pub lease_duration: u32,
    pub data: Data<T>,
    pub wrap_info: Option<String>,
    pub warnings: Option<String>,
    pub auth: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Data<T> {
    pub data: T,
}
