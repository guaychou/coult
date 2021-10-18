/// Config struct to match the address of vault
pub struct Config {
    pub address: &'static str,
    pub port: u16,
    pub config_path: &'static str,
    pub token: &'static str,
}

impl Config {
    pub fn new(
        address: &'static str,
        port: u16,
        config_path: &'static str,
        token: &'static str,
    ) -> Self {
        Self {
            address,
            port,
            config_path,
            token,
        }
    }
}
