use std::{
    collections::HashMap,
    io::{Error, Read, Write},
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
};

use crate::{http_request::HttpRequest, http_response::HttpResponse};

pub struct EventLoop {
    epoll_fd: RawFd,
    connections: HashMap<RawFd, TcpStream>,
    pub route_map: HashMap<String, String>,
}

impl EventLoop {
    // Create a new server
    pub fn new() -> std::io::Result<Self> {
        let epoll_fd = unsafe { libc::epoll_create1(0) };
        if epoll_fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self {
            epoll_fd,
            connections: HashMap::new(),
            route_map: HashMap::new(),
        })
    }

    // Add a new listener (port) to handle by the server
    pub fn add_listener(&mut self, listener: &TcpListener) -> std::io::Result<()> {
        let mut event = libc::epoll_event {
            events: (libc::EPOLLIN | libc::EPOLLET) as u32,
            u64: listener.as_raw_fd() as u64,
        };

        let res = unsafe {
            libc::epoll_ctl(
                self.epoll_fd,
                libc::EPOLL_CTL_ADD,
                listener.as_raw_fd(),
                &mut event,
            )
        };

        if res < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    // Run the server
    pub fn run(&mut self, listeners: Vec<TcpListener>) -> std::io::Result<()> {
        let mut events = vec![libc::epoll_event { events: 0, u64: 0 }; 1024];

        loop {
            let num_events = unsafe {
                libc::epoll_wait(self.epoll_fd, events.as_mut_ptr(), events.len() as i32, -1)
            };

            if num_events < 0 {
                return Err(Error::last_os_error());
            }

            for i in 0..num_events as usize {
                let event = &events[i];
                for listener in listeners.iter() {
                    if event.u64 == listener.as_raw_fd() as u64 {
                        match listener.accept() {
                            Ok((stream, addr)) => {
                                println!("New request from: {:?}", addr);
                                self.add_stream(stream)?;
                            }
                            Err(e) => eprintln!("Error: {:?}", e),
                        }
                    } else {
                        let stream_fd = event.u64 as RawFd;
                        let maybe_stream = self
                            .connections
                            .get_mut(&stream_fd)
                            .map(|stream| stream.try_clone());
                        if let Some(Ok(mut stream)) = maybe_stream {
                            // Traiter la connexion
                            if let Err(e) = self.handle_connection(&mut stream) {
                                eprintln!("Error handling connection: {:?}", e);
                                self.connections.remove(&stream_fd);
                            }
                        }
                    }
                }
            }
        }
    }

    // Handle a new connection to the server
    fn add_stream(&mut self, stream: TcpStream) -> std::io::Result<()> {
        let fd = stream.as_raw_fd();
        let mut event = libc::epoll_event {
            events: (libc::EPOLLIN | libc::EPOLLET) as u32,
            u64: fd as u64,
        };

        let res = unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event) };
        if res < 0 {
            return Err(std::io::Error::last_os_error());
        }

        self.connections.insert(fd, stream);
        Ok(())
    }

    // Handle the connection
    fn handle_connection(&mut self, stream: &mut TcpStream) -> std::io::Result<()> {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            // The request is done and well handled
            Ok(0) => {
                //println!("Connection closed");
                self.connections.remove(&stream.as_raw_fd());
                Ok(())
            }
            Ok(n) => {
                // Get a new request
                let request_raw = String::from_utf8_lossy(&buffer[..n]);
                if let Some(request) = HttpRequest::from_raw(&request_raw) {
                    println!(
                        "--------------- New request ---------------\n{:?}\n",
                        request
                    );

                    let response = match self.route_map.get(&request.path) {
                        Some(message) => HttpResponse::ok(message),
                        None => HttpResponse::not_found(),
                    };

                    stream.write_all(response.to_string().as_bytes())?;
                } else {
                    eprintln!("Failed to parse request");
                    stream.write_all(HttpResponse::bad_request().to_string().as_bytes())?;
                }

                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}
