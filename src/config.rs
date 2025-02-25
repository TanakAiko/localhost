use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::TcpListener;
use std::process::Command;
use std::{
    fs, io,
    os::fd::{AsRawFd, RawFd},
};

// use crate::cgi::{list_directory,handle_route};

use crate::event_loop::EventLoop;

use libc::{fcntl, F_GETFL, F_SETFL, O_NONBLOCK};

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerConfig {
    pub name: String,
    pub addr: String,
    pub ports: Vec<String>,
    pub routes: HashMap<String, RouteConfig>,
    pub error_pages: Option<HashMap<u16, String>>, // Ex: 404 -> "/path/to/404.html"
    pub client_body_size_limit: Option<usize>,     // Ex: Limite d'upload en octets
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RouteConfig {
    pub accepted_methods: Option<Vec<String>>, // Ex: ["GET", "POST"]
    pub redirection: Option<String>,           // Ex: "/old" -> "/new"
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
        let mut server_addresses = Vec::new();
        // let mut addesses = Vec::new();

        for server in &self.servers {

            // Check if there's two server with the same name
            if !server_names.insert(&server.name) {
                eprintln!("IGNORE: Duplicate server name '{}'", server.name);
                continue;
            }

            // Créer une copie mutable des routes du serveur
            let mut server_routes = server.routes.clone();

            // Ajouter les routes de session
            server_routes.insert("/session".to_string(), RouteConfig {
                accepted_methods: Some(vec!["GET".to_string()]),
                default_file: Some("session.html".to_string()),
                redirection: None,
                cgi: None,
                directory_listing: None,
            });

            server_routes.insert("/create-session".to_string(), RouteConfig {
                accepted_methods: Some(vec!["POST".to_string()]),
                default_file: Some("session.html".to_string()),
                redirection: None,
                cgi: None,
                directory_listing: None,
            });

            for port in &server.ports {
                let address = format!("{}:{}", server.addr, port);

                // Check if there's two listener with the same addresses within the same server
                if server_addresses.contains(&(server.name.clone(), address.clone())) {
                    eprintln!(
                        "IGNORE: Address '{}' for server '{}' already exists.",
                        address, server.name
                    );
                    continue;
                }

                server_addresses.push((server.name.clone(), address.clone()));


                let listener = match TcpListener::bind(&address) {
                    Ok(listener) => listener,
                    Err(err) => {
                        match err.kind() {
                            std::io::ErrorKind::AddrInUse => {
                                Self::add_to_hosts(&server.name, &server.addr)?;
                                println!(
                                    "Server '{}' launched at: http://{}",
                                    server.name, address
                                );
                                event_loop.add_server(server.name.clone(), server.routes.clone(), server.error_pages.clone(), server.client_body_size_limit);
                            }
                            std::io::ErrorKind::AddrNotAvailable => {
                                eprintln!(
                                    "IGNORE: Address '{}' for server '{}' is not valid or not available.",
                                    address, server.name
                                );
                            }
                            _ => {
                                eprintln!(
                                    "IGNORE: Failed to bind to address '{}' for server '{}' due to: {:?}",
                                    address, server.name, err
                                );
                            }
                        }
                        continue;
                    }
                };

                set_non_blocking(listener.as_raw_fd())?;

                Self::add_to_hosts(&server.name, &server.addr)?;
                println!("Server '{}' launched at: http://{}", server.name, address);
                // let routes = server.routes.clone();
                event_loop.add_listener(&listener, server.name.clone(), server_routes.clone(), server.error_pages.clone(), server.client_body_size_limit)?;
                listener_list.push(listener);
            }
        }

        if let Err(e) = event_loop.run(listener_list) {
            eprintln!("ERROR: running server: {:?}", e);
        };
        Ok(())
    }

    fn add_to_hosts(name: &str, ip: &str) -> io::Result<()> {
        let hosts_path = "/etc/hosts";
        let entry = format!("{} {}", ip, name);

        // Read the content of the hosts file
        let content = fs::read_to_string(hosts_path)?;

        // Check if the mapping already exists
        if content.lines().any(|line| line.trim() == entry) {
            //println!("Mapping {} to {} already exists in hosts file.", name, ip);
            return Ok(());
        }

        // Add the mapping using sudo
        let output = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(format!("echo '{}' >> {}", entry, hosts_path))
            .output()?;

        if !output.status.success() {
            eprintln!(
                "Failed to add to hosts file: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to execute sudo command",
            ));
        }

        //println!("Mapping {} to {} added to hosts file.", name, ip);
        Ok(())
    }
}
