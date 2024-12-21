use std::{
    collections::HashMap,
    io::{Error, Read, Write},
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
};

use crate::{http_request::HttpRequest, http_response::HttpResponse};

#[derive(Debug)]
pub struct EventLoop {
    epoll_fd: RawFd,
    //connections: HashMap<RawFd, TcpStream>,
    servers: HashMap<String, Server>,
}

#[derive(Debug)]
pub struct Server {
    pub listeners: Vec<RawFd>,
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
            //connections: HashMap::new(),
            servers: HashMap::new(),
        })
    }

    // Add a new listener (port) to handle by the server
    pub fn add_listener(
        &mut self,
        listener: &TcpListener,
        server_name: String,
        routes: HashMap<String, String>,
    ) -> std::io::Result<()> {
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

        let server = self.servers.entry(server_name).or_insert(Server {
            listeners: Vec::new(),
            route_map: routes,
        });

        server.listeners.push(listener.as_raw_fd());

        //self.listeners.insert(listener.as_raw_fd(), routes);

        if res < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    // Run the server
    pub fn run(&mut self, listeners_list: Vec<TcpListener>) -> std::io::Result<()> {
        let mut events = vec![libc::epoll_event { events: 0, u64: 0 }; 1024];

        loop {
            //println!("\nAll Servers: \n{:?}\n", self);

            let num_events = unsafe {
                libc::epoll_wait(self.epoll_fd, events.as_mut_ptr(), events.len() as i32, -1)
            };

            if num_events < 0 {
                return Err(Error::last_os_error());
            }

            for i in 0..num_events as usize {
                let event = &events[i];
                let event_fd = event.u64 as RawFd;

                for listener in listeners_list.iter() {
                    //let listener_fd = listener.as_raw_fd();
                    //println!("listener_fd: {} - event_fd: {}", listener_fd, event_fd);
                    if listener.as_raw_fd() == event_fd {
                        match listener.accept() {
                            Ok((mut stream, _addr)) => {
                                //println!("New request from: {:?}", addr);
                                let routes = self.route_map(event_fd);
                                if let Err(e) = self.handle_connection(&mut stream, routes) {
                                    eprintln!("Error handling connection: {:?}", e);
                                    //self.connections.remove(&event_fd);
                                }
                            }
                            Err(e) => eprintln!("Error1: {:?}", e),
                        }
                        break;
                    }
                }
            }
        }
    }

    // Handle a new connection to the server
    /* fn add_stream(&mut self, stream: TcpStream) -> std::io::Result<()> {
           let fd = stream.as_raw_fd();
           let mut event = libc::epoll_event {
               events: (libc::EPOLLIN | libc::EPOLLET) as u32,
               u64: fd as u64,
           };

           let res = unsafe { libc::epoll_ctl(self.epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event) };
           if res < 0 {
               return Err(std::io::Error::last_os_error());
           }

           self.connections.insert(fd, stream.try_clone()?);
           Ok(())
       }
    */
    // Handle the connection
    fn handle_connection(
        &mut self,
        stream: &mut TcpStream,
        routes: HashMap<String, String>,
    ) -> std::io::Result<()> {
        //self.add_stream(stream)?;

        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            // The request is done and well handled
            Ok(0) => {
                //println!("Connection closed");
                //self.connections.remove(&stream.as_raw_fd());
                Ok(())
            }
            Ok(n) => {
                // Get a new request
                let request_raw = String::from_utf8_lossy(&buffer[..n]);
                if let Some(request) = HttpRequest::from_raw(&request_raw) {
                    println!(
                        "\n--------------- New request ---------------\n{:?}\n",
                        request
                    );
                    
                    let response = match routes.get(&request.path) {
                        Some(message) => HttpResponse::ok(request, message),
                        None if request.path == "/style.css" => HttpResponse::get_static(request),
                        None => HttpResponse::not_found(),
                    };

                    println!(
                        "\n--------------- Response ---------------\n{:?}\n",
                        response
                    );

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

    fn route_map(&self, fd: RawFd) -> HashMap<String, String> {
        self.servers
            .values()
            .find(|server| server.listeners.contains(&fd))
            .map(|server| server.route_map.clone())
            .unwrap_or_default()
    }
}
