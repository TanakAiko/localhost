use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{fs, io};

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerConfig {
    pub name: String,
    pub addr: String,
    pub ports: Vec<String>,
    pub routes: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub servers: Vec<ServerConfig>,
}

pub fn load_config(file_path: &str) -> io::Result<Config> {
    let config_data = fs::read_to_string(file_path)?;
    let config: Config = serde_json::from_str(&config_data)?;
    Ok(config)
}
