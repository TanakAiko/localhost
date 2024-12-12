use std::net::TcpListener;

use event_loop::EventLoop;
pub mod event_loop;
pub mod http_request;
pub mod http_response;

fn main() -> std::io::Result<()> {
    let mut listener_list = Vec::new();

    // Initiate listeners & push them to the listener_list
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    listener.set_nonblocking(true)?;
    println!("\nServer launch at : http://127.0.0.1:8080");
    listener_list.push(&listener);

    let listener1 = TcpListener::bind("127.0.0.1:8082")?;
    listener1.set_nonblocking(true)?;
    println!("Server launch at : http://127.0.0.1:8082\n");
    listener_list.push(&listener1);

    // Create the event_loop for the server
    let mut event_loop = EventLoop::new()?;

    // Add listeners to the server
    for listen in listener_list.iter() {
        event_loop.add_listener(listen)?;
    }

    // Run the server
    event_loop.run(listener_list)
}
