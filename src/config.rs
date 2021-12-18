/// Config struct to match the address of vault
pub struct Config {
    pub address: String,
    pub port: u16,
    pub config_path: String,
    pub token: String,
}

impl Config {
    pub fn new(address: String, port: u16, config_path: String, token: String) -> Self {
        Self {
            address,
            port,
            config_path,
            token,
        }
    }
}
