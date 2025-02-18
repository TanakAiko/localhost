use std::{
    collections::HashMap,
    io::{Error, Read, Write},
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
};

use crate::{config::RouteConfig, http_request::HttpRequest, http_response::HttpResponse};

#[derive(Debug)]
pub struct EventLoop {
    epoll_fd: RawFd,
    //connections: HashMap<RawFd, TcpStream>,
    servers: HashMap<String, Server>,
}

#[derive(Debug)]
pub struct Server {
    pub name: String,
    pub listeners: Vec<RawFd>,
    pub route_map: HashMap<String, RouteConfig>,
    pub error_pages: Option<HashMap<u16, String>>,
    pub size_limit: Option<usize>
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

    pub fn add_server(
        &mut self,
        server_name: String,
        routes: HashMap<String, RouteConfig>,
        error_pages: Option<HashMap<u16, String>>,
        size_limit: Option<usize>
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
                size_limit
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
        size_limit: Option<usize>
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
            size_limit
        });

        //println!("\nlistener.as_raw_fd(): {}", listener.as_raw_fd());
        server.listeners.push(listener.as_raw_fd());
        //println!("server.listeners: {:?}\n", server.listeners);

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
            // for ser in self.servers.iter() {
            //     println!("\n\nServer: \n{:?}\n\n", ser);
            // }
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
                                //let routes = self.route_map(event_fd);
                                if let Err(e) = self.handle_connection(&mut stream, event_fd) {
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

    // Handle the connection
    fn handle_connection(&mut self, stream: &mut TcpStream, fd: RawFd) -> std::io::Result<()> {
        //self.add_stream(stream)?;

        let mut buffer = Vec::new(); // Utilisation d'un vecteur dynamique pour accumuler les données
        let mut temp_buffer = [0; 1024];

        // Lire les données initiales (les en-têtes HTTP)
        let bytes_read = stream.read(&mut temp_buffer)?;
        buffer.extend_from_slice(&temp_buffer[..bytes_read]);

        // Parse les en-têtes pour obtenir la longueur du corps
        let request_raw = String::from_utf8_lossy(&buffer);
        let content_length = Self::parse_content_length(&request_raw).unwrap_or(0);

        // Si un corps est attendu, continuer à lire les données
        while buffer.len() < content_length {
            let bytes_read = stream.read(&mut temp_buffer)?;
            if bytes_read == 0 {
                break;
            }
            buffer.extend_from_slice(&temp_buffer[..bytes_read]);
        }

        // Get a new request
        if let Some(request) = HttpRequest::from_raw(&buffer) {
            println!(
                "\n--------------- New request ---------------\n{:?}\n",
                request
            );

            let hostname = request
                .headers
                .get("Host")
                .map(|h| h.to_string())
                .unwrap_or_else(|| "".to_string());

            let routes = Self::route_map(&self, fd, hostname.clone());
            let error_pages = Self::get_error_pages(&self, fd, hostname.clone());
            let size_limit = Self::get_size_limit(&self, fd, hostname);
            // println!("{:?}", routes);
            
            println!("request.path: {:?}", routes.get(&request.path));
            let response = match routes.get(&request.path) {
                Some(route_config) => HttpResponse::ok(request, route_config, error_pages, size_limit),
                None => HttpResponse::get_static(request, error_pages),
            };

            println!(
                "\n--------------- Response ---------------\n{:?}\n",
                response.headers
            );

            println!("response.body.len(): {}", response.body.len());
            stream.write_all(&response.to_bytes())?;
        } else {
            let error_pages = Self::get_error_pages(&self, fd, "".to_string());
            eprintln!("Failed to parse request");
            stream.write_all(&HttpResponse::bad_request(error_pages).to_bytes())?;
        }

        Ok(())
    } 

    fn route_map(&self, fd: RawFd, hostname: String) -> HashMap<String, RouteConfig> {
        let host = hostname.split_once(":").unwrap_or(("", "")).0;

        if let Some(server) = self
            .servers
            .values()
            .find(|server| server.name.to_lowercase() == host)
        {
            //println!("routes from host");
            return server.route_map.clone();
        }

        //println!("routes from fd");
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
            return server.size_limit.clone()
        }

        // Recherche par descripteur de fichier (fd)
        self.servers
            .values()
            .find(|server| server.listeners.contains(&fd))
            .and_then(|server| server.size_limit.clone())
    }

    // Fonction pour extraire la longueur du contenu des en-têtes HTTP
    fn parse_content_length(request: &str) -> Option<usize> {
        for line in request.lines() {
            if let Some(value) = line.strip_prefix("Content-Length: ") {
                return value.trim().parse().ok();
            }
        }
        None
    }
}
