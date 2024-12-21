use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::TcpListener;
use std::{fs, io};

use crate::event_loop::EventLoop;

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

impl Config {
    // To run all valide config in our server
    pub fn start(&self) -> std::io::Result<()> {
        let mut event_loop = EventLoop::new()?;
        let mut listener_list = Vec::new();
        let mut server_names = HashSet::new();
        let mut addresses = HashSet::new();

        for server in &self.servers {
            // Check if there's two server with the same name
            if !server_names.insert(&server.name) {
                eprintln!("IGNORE: Duplicate server name '{}'", server.name);
                continue;
            }
            for port in &server.ports {
                let address = format!("{}:{}", server.addr, port);

                // Check if there's two listener with the same addresses
                if !addresses.insert(address.clone()) {
                    eprintln!(
                        "IGNORE: Duplicate address '{}' for server '{}'",
                        address, server.name
                    );
                    continue;
                }

                let listener = match TcpListener::bind(&address) {
                    Ok(listener) => listener,
                    Err(_) => {
                        eprintln!(
                            "IGNORE: Failed to bind to address '{}' for server '{}'",
                            address, server.name
                        );
                        continue;
                    }
                };

                listener.set_nonblocking(true)?;
                println!("Server '{}' launched at: http://{}", server.name, address);
                event_loop.add_listener(&listener, server.name.clone(), server.routes.clone())?;
                listener_list.push(listener);
            }
        }

        if let Err(e) = event_loop.run(listener_list) {
            eprintln!("ERROR: running server: {:?}", e);
        };
        Ok(())
    }
}
