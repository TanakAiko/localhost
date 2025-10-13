use crate::{
    config::RouteConfig, http_request::HttpRequest, http_response::HttpResponse,
    request_queue::RequestQueue,
};
use std::{
    collections::HashMap,
    io::{Error, Read, Write},
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
};

#[derive(Debug)]
pub struct EventLoop {
    pub epoll_fd: RawFd,
    pub servers: HashMap<String, Server>,
    pub request_queues: HashMap<RawFd, RequestQueue>,
}

#[derive(Debug)]
pub struct Server {
    pub name: String,
    pub listeners: Vec<RawFd>,
    pub route_map: HashMap<String, RouteConfig>,
    pub error_pages: Option<HashMap<u16, String>>,
    pub size_limit: Option<usize>,
}

impl EventLoop {
    pub fn new() -> std::io::Result<Self> {
        let epoll_fd = unsafe { libc::epoll_create1(0) };
        if epoll_fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self {
            epoll_fd,
            servers: HashMap::new(),
            request_queues: HashMap::new(),
        })
    }

    pub fn add_server(
        &mut self,
        server_name: String,
        routes: HashMap<String, RouteConfig>,
        error_pages: Option<HashMap<u16, String>>,
        size_limit: Option<usize>,
    ) {
        if self.servers.contains_key(&server_name) {
            eprintln!(
                "Server with name '{}' already exists. Skipping addition.",
                server_name
            );
            return;
        }

        self.servers.insert(
            server_name.clone(),
            Server {
                name: server_name,
                listeners: Vec::new(),
                route_map: routes,
                error_pages,
                size_limit,
            },
        );
    }

    // Add a new listener (port) to handle by the server
    pub fn add_listener(
        &mut self,
        listener: &TcpListener,
        server_name: String,
        routes: HashMap<String, RouteConfig>,
        error_pages: Option<HashMap<u16, String>>,
        size_limit: Option<usize>,
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

        let server = self.servers.entry(server_name.clone()).or_insert(Server {
            name: server_name,
            listeners: Vec::new(),
            route_map: routes,
            error_pages,
            size_limit,
        });

        server.listeners.push(listener.as_raw_fd());

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
                    if listener.as_raw_fd() == event_fd {
                        match listener.accept() {
                            Ok((mut stream, _addr)) => {
                                if let Err(e) = self.handle_connection(&mut stream, event_fd) {
                                    eprintln!("Error handling connection: {:?}", e);

                                    // 🔸 Close the connection
                                    if let Err(shutdown_err) =
                                        stream.shutdown(std::net::Shutdown::Both)
                                    {
                                        eprintln!(
                                            "Error shutting down connection: {:?}",
                                            shutdown_err
                                        );
                                    }
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

    fn route_map(&self, fd: RawFd, hostname: String) -> HashMap<String, RouteConfig> {
        let host = hostname.split_once(":").unwrap_or(("", "")).0;

        if let Some(server) = self
            .servers
            .values()
            .find(|server| server.name.to_lowercase() == host)
        {
            return server.route_map.clone();
        }

        self.servers
            .values()
            .find(|server| server.listeners.contains(&fd))
            .map(|server| server.route_map.clone())
            .unwrap_or_default()
    }

    fn get_error_pages(&self, fd: RawFd, hostname: String) -> Option<HashMap<u16, String>> {
        let host = hostname.split_once(":").unwrap_or(("", "")).0;

        // Recherche par nom d'hôte
        if let Some(server) = self
            .servers
            .values()
            .find(|server| server.name.to_lowercase() == host)
        {
            return server.error_pages.clone();
        }

        // Recherche par descripteur de fichier (fd)
        self.servers
            .values()
            .find(|server| server.listeners.contains(&fd))
            .and_then(|server| server.error_pages.clone())
    }

    fn get_size_limit(&self, fd: RawFd, hostname: String) -> Option<usize> {
        let host = hostname.split_once(":").unwrap_or(("", "")).0;

        // Recherche par nom d'hôte
        if let Some(server) = self
            .servers
            .values()
            .find(|server| server.name.to_lowercase() == host)
        {
            return server.size_limit.clone();
        }

        // Recherche par descripteur de fichier (fd)
        self.servers
            .values()
            .find(|server| server.listeners.contains(&fd))
            .and_then(|server| server.size_limit.clone())
    }

    fn process_request(&mut self, request: HttpRequest) -> HttpResponse {
        let hostname = request
            .headers
            .get("Host")
            .map(|h| h.to_string())
            .unwrap_or_default();

        let routes = Self::route_map(&self, request.listener_fd, hostname.clone());
        let error_pages = Self::get_error_pages(&self, request.listener_fd, hostname.clone());
        let size_limit = Self::get_size_limit(&self, request.listener_fd, hostname);

        match routes.get(&request.path) {
            Some(route_config) => HttpResponse::ok(request, route_config, error_pages, size_limit),
            None => HttpResponse::get_static(request, error_pages),
        }
    }

    fn handle_connection(
        &mut self,
        stream: &mut TcpStream,
        listener_fd: RawFd,
    ) -> std::io::Result<()> {
        let stream_fd = stream.as_raw_fd();

        // Create a new tail if it does not exist
        if !self.request_queues.contains_key(&stream_fd) {
            self.request_queues
                .insert(stream_fd, RequestQueue::new(100)); // 100 est la taille max de la queue
        }

        let mut keep_alive = true;

        println!("\n*******************New Connection*******************",);

        while keep_alive {
            stream.set_read_timeout(Some(std::time::Duration::from_millis(500)))?;

            // Read the request and add it to the tail
            match read_request(stream, listener_fd) {
                Ok(request) => {
                    println!(
                        "-----------------New Request-----------------\n{:?}\n",
                        request
                    );
                    keep_alive = check_connection_headers(&request);

                    // Treat the request immediately instead of using a tail
                    let mut requests_to_process = Vec::new();

                    // Collect tail requests
                    if let Some(queue) = self.request_queues.get_mut(&stream_fd) {
                        if queue.push(request).is_ok() {
                            while let Some(req) = queue.pop() {
                                requests_to_process.push(req);
                            }
                        } else {
                            let error_response = HttpResponse::service_unavailable(None);
                            stream.write_all(&error_response.to_bytes())?;
                            keep_alive = false;
                            continue;
                        }
                    }

                    // Treat all requests collected
                    for req in requests_to_process {
                        let response = self.process_request(req);
                        let final_response = add_connection_headers(response, keep_alive);
                        stream.write_all(&final_response.to_bytes())?;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // println!("Timeout's over");
                    keep_alive = false;
                }
                Err(e) => {
                    eprintln!("Reading error of the request: {:?}", e);
                    self.request_queues.remove(&stream_fd);
                    return Err(e);
                }
            }
        }

        // Clean the tail when the connection is closed
        self.request_queues.remove(&stream_fd);
        Ok(())
    }
}

fn read_request(stream: &mut TcpStream, listener_fd: RawFd) -> std::io::Result<HttpRequest> {
    let mut buffer = Vec::new();
    let mut temp_buffer = [0; 1024];

    // Initial reading for headers
    let bytes_read = stream.read(&mut temp_buffer)?;
    if bytes_read == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Connection closed by peer",
        ));
    }
    buffer.extend_from_slice(&temp_buffer[..bytes_read]);

    // Sprinkle the headers to obtain the length of the body
    let new_buff = buffer.clone();
    let request_raw = String::from_utf8_lossy(&new_buff);
    let content_length = parse_content_length(&request_raw).unwrap_or(0);

    // Continue reading if a body is expected
    while buffer.len() < content_length + request_raw.find("\r\n\r\n").unwrap_or(0) + 4 {
        let bytes_read = stream.read(&mut temp_buffer)?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&temp_buffer[..bytes_read]);
    }

    match HttpRequest::from_raw(&buffer, listener_fd, stream.as_raw_fd()) {
        Some(request) => Ok(request),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid HTTP request",
        )),
    }
}

fn add_connection_headers(mut response: HttpResponse, keep_alive: bool) -> HttpResponse {
    let connection_value = if keep_alive { "keep-alive" } else { "close" };
    response
        .headers
        .push(("Connection".to_string(), connection_value.to_string()));

    if keep_alive {
        response
            .headers
            .push(("Keep-Alive".to_string(), "timeout=5, max=100".to_string()));
    }

    response
}

fn check_connection_headers(request: &HttpRequest) -> bool {
    let connection_header = request.headers.get("Connection").map(|h| h.to_lowercase());

    match connection_header {
        Some(header) => {
            if header == "close" {
                false
            } else if header == "keep-alive" {
                true
            } else {
                // Par défaut en HTTP/1.1, la connexion est keep-alive
                request.version == "HTTP/1.1"
            }
        }
        None => request.version == "HTTP/1.1", // Keep-alive par défaut en HTTP/1.1
    }
}

// Function to extract the length of the content of the HTTP headers
fn parse_content_length(request: &str) -> Option<usize> {
    for line in request.lines() {
        if let Some(value) = line.strip_prefix("Content-Length: ") {
            return value.trim().parse().ok();
        }
    }
    None
}
