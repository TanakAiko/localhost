use std::net::TcpListener;

use config::load_config;
use event_loop::EventLoop;
pub mod config;
pub mod event_loop;
pub mod http_request;
pub mod http_response;

fn main() -> std::io::Result<()> {
    let config = load_config("config.json").expect("Failed to load configuration");

    println!("\nconfig: {:?}\n", config);

    for server in &config.servers {
        // Create the event_loop for the server
        let mut event_loop = EventLoop::new()?;
        event_loop.route_map = server.routes.clone();
        let mut listener_list = Vec::new();

        for port in &server.ports {
            let address = format!("{}:{}", server.addr, port);
            let listener = TcpListener::bind(&address)?;
            listener.set_nonblocking(true)?;
            println!("Server '{}' launched at: http://{}", server.name, address);
            listener_list.push(listener);
        }

        for listen in listener_list.iter() {
            event_loop.add_listener(listen)?;
        }

        // Run the server
        if let Err(e) = event_loop.run(listener_list) {
            eprintln!("Error running server '{}': {:?}", server.name, e);
        }
    }

    Ok(())
}
