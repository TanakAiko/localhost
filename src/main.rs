use std::{collections::HashSet, net::TcpListener};

use config::load_config;
use event_loop::EventLoop;
pub mod config;
pub mod event_loop;
pub mod http_request;
pub mod http_response;

fn main() -> std::io::Result<()> {
    let config = load_config("config.json").expect("Failed to load configuration");
    //println!("\nconfig: {:?}\n", config);
    let mut event_loop = EventLoop::new()?;
    let mut listener_list = Vec::new();
    let mut server_names = HashSet::new();
    let mut addresses = HashSet::new();

    for server in &config.servers {
        // Check if there's two server with the same name
        if !server_names.insert(&server.name) {
            eprintln!("Ignore: Duplicate server name '{}'", server.name);
            continue;
        }

        for port in &server.ports {
            let address = format!("{}:{}", server.addr, port);

            // Check if there's two listener with the same addresses
            if !addresses.insert(address.clone()) {
                eprintln!(
                    "Ignore: Duplicate address '{}' for server '{}'",
                    address, server.name
                );
                continue;
            }

            let listener = TcpListener::bind(&address)?;
            listener.set_nonblocking(true)?;
            println!("Server '{}' launched at: http://{}", server.name, address);
            event_loop.add_listener(&listener, server.name.clone(), server.routes.clone())?;
            listener_list.push(listener);
        }
    }

    //println!("listener_list: {:?}", listener_list);

    if let Err(e) = event_loop.run(listener_list) {
        eprintln!("Error running server: {:?}", e);
    }

    Ok(())
}
