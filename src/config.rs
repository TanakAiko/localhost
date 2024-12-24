use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::TcpListener;
use std::{fs, io, os::fd::{AsRawFd, RawFd},};

use crate::event_loop::EventLoop;



use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerConfig {
    pub name: String,
    pub addr: String,
    pub ports: Vec<String>,
    pub routes: HashMap<String, RouteConfig>,
    pub error_pages: Option<HashMap<u16, String>>, // Ex: 404 -> "/path/to/404.html"
    pub client_body_size_limit: Option<usize>, // Ex: Limite d'upload en octets
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RouteConfig {
    pub accepted_methods: Option<Vec<String>>, // Ex: ["GET", "POST"]
    pub redirection: Option<String>,           // Ex: "/old" -> "/new"
    pub root: Option<String>,                  // Ex: "/test" -> "/usr/Desktop"
    pub default_file: Option<String>,          // Ex: "index.html"
    pub cgi: Option<String>,                   // Ex: Extension ".py" -> "/path/to/python"
    pub directory_listing: Option<bool>,
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


fn set_non_blocking(fd: RawFd) -> std::io::Result<()> {
    let flags = unsafe { fcntl(fd, F_GETFL) };
    if flags < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let res = unsafe { fcntl(fd, F_SETFL, flags | O_NONBLOCK) };
    if res < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
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

                set_non_blocking(listener.as_raw_fd())?;
                
                println!("Server '{}' launched at: http://{}", server.name, address);
                let routes = server.routes.clone();
                event_loop.add_listener(&listener, server.name.clone(), routes)?;
                listener_list.push(listener);
            }
        }

        if let Err(e) = event_loop.run(listener_list) {
            eprintln!("ERROR: running server: {:?}", e);
        };
        Ok(())
    }
}
