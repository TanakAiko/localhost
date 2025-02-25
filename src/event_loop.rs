use crate::{config::RouteConfig, http_request::HttpRequest, http_response::HttpResponse};
use std::{
    collections::{HashMap, VecDeque},
    io::{Error, Read, Write},
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
};

#[derive(Debug)]
pub struct EventLoop {
    epoll_fd: RawFd,
    connections: HashMap<RawFd, TcpStream>,
    servers: HashMap<String, Server>,
    request_queues: HashMap<RawFd, RequestQueue>,
}

#[derive(Debug)]
pub struct Server {
    pub name: String,
    pub listeners: Vec<RawFd>,
    pub route_map: HashMap<String, RouteConfig>,
    pub error_pages: Option<HashMap<u16, String>>,
    pub size_limit: Option<usize>,
}

#[derive(Debug)]
struct RequestQueue {
    requests: VecDeque<HttpRequest>,
    max_queued: usize,
}

impl RequestQueue {
    fn new(max_queued: usize) -> Self {
        Self {
            requests: VecDeque::new(),
            max_queued,
        }
    }

    fn push(&mut self, request: HttpRequest) -> Result<(), HttpResponse> {
        if self.requests.len() >= self.max_queued {
            return Err(HttpResponse::internal_server_error(None));
        }
        self.requests.push_back(request);
        Ok(())
    }

    fn pop(&mut self) -> Option<HttpRequest> {
        self.requests.pop_front()
    }
}

impl EventLoop {
    // Create a new server
    /* pub fn new() -> std::io::Result<Self> {
        let epoll_fd = unsafe { libc::epoll_create1(0) };
        if epoll_fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self {
            epoll_fd,
            connections: HashMap::new(),
            servers: HashMap::new(),
        })
    } */

    // Vérifier l'état des queues
    fn check_queues(&self) -> usize {
        self.request_queues.values().map(|q| q.requests.len()).sum()
    }

    // Nettoyer les queues inactives
    fn cleanup_queues(&mut self) {
        self.request_queues
            .retain(|_, queue| !queue.requests.is_empty());
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
                                // println!("New request from: {:?}", addr);
                                //let routes = self.route_map(event_fd);
                                if let Err(e) = self.handle_connection(&mut stream, event_fd) {
                                    eprintln!("Error handling connection: {:?}", e);

                                    // 🔸 Fermer proprement la connexion
                                    if let Err(shutdown_err) =
                                        stream.shutdown(std::net::Shutdown::Both)
                                    {
                                        eprintln!(
                                            "Error shutting down connection: {:?}",
                                            shutdown_err
                                        );
                                    }

                                    // 🔸 Supprimer de la liste des connexions suivies
                                    self.connections.remove(&event_fd);

                                    // 🔸 Désenregistrer de epoll si nécessaire
                                    unsafe {
                                        libc::epoll_ctl(
                                            self.epoll_fd,
                                            libc::EPOLL_CTL_DEL,
                                            event_fd,
                                            std::ptr::null_mut(),
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

    // Handle the connection
    /*     fn handle_connection(&mut self, stream: &mut TcpStream, fd: RawFd) -> std::io::Result<()> {
           //self.add_stream(stream)?;

           let mut buffer = Vec::new(); // Utilisation d'un vecteur dynamique pour accumuler les données
           let mut temp_buffer = [0; 1024];
           let mut _path = "";

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
           if let Some(request) = HttpRequest::from_raw(&buffer, fd) {
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

               let response = match routes.get(&request.path) {
                   Some(route_config) => {
                       HttpResponse::ok(request, route_config, error_pages, size_limit)
                   }
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
    */

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
            return server.size_limit.clone();
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

    fn read_request(&self, stream: &mut TcpStream, listener_fd: RawFd) -> std::io::Result<HttpRequest> {
        let mut buffer = Vec::new();
        let mut temp_buffer = [0; 1024];

        // Lecture initiale pour les en-têtes
        let bytes_read = stream.read(&mut temp_buffer)?;
        if bytes_read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Connection closed by peer",
            ));
        }
        buffer.extend_from_slice(&temp_buffer[..bytes_read]);

        // Parse les en-têtes pour obtenir la longueur du corps
        let new_buff = buffer.clone();
        let request_raw = String::from_utf8_lossy(&new_buff);
        let content_length = Self::parse_content_length(&request_raw).unwrap_or(0);

        // Continue la lecture si un corps est attendu
        while buffer.len() < content_length + request_raw.find("\r\n\r\n").unwrap_or(0) + 4 {
            let bytes_read = stream.read(&mut temp_buffer)?;
            if bytes_read == 0 {
                break;
            }
            buffer.extend_from_slice(&temp_buffer[..bytes_read]);
        }

        match HttpRequest::from_raw(&buffer, listener_fd) {
            Some(request) => Ok(request),
            None => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid HTTP request",
            )),
        }
    }

    fn check_connection_headers(&self, request: &HttpRequest) -> bool {
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

    fn process_request(&mut self, request: HttpRequest) -> HttpResponse {
        let hostname = request
            .headers
            .get("Host")
            .map(|h| h.to_string())
            .unwrap_or_default();

        // println!("hostname: {:?}", hostname);
        // println!("request.fd: {:?}", request.fd);
        // println!("self: {:?}", self);

        let routes = Self::route_map(&self, request.fd, hostname.clone());
        let error_pages = Self::get_error_pages(&self, request.fd, hostname.clone());
        let size_limit = Self::get_size_limit(&self, request.fd, hostname);

        println!("routes: {:?}", routes);
        println!("request.path: {:?}", request.path);
        match routes.get(&request.path) {
            Some(route_config) => HttpResponse::ok(request, route_config, error_pages, size_limit),
            None => HttpResponse::get_static(request, error_pages),
        }
    }

    

    // Mise à jour du handle_connection pour utiliser les nouvelles fonctions
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

    // 3. Modifiez handle_connection pour utiliser la queue
    fn handle_connection(&mut self, stream: &mut TcpStream, fd: RawFd) -> std::io::Result<()> {
        println!("handle_connection -------------");
        // Créer une nouvelle queue si elle n'existe pas
        if !self.request_queues.contains_key(&fd) {
            self.request_queues.insert(fd, RequestQueue::new(100)); // 100 est la taille max de la queue
        }

        let mut keep_alive = true;

        while keep_alive {
            stream.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;

            // Lire la requête et l'ajouter à la queue
            match self.read_request(stream, fd) {
                Ok(request) => {
                    // println!("Request OOOOKKKKKKKKKKKKKKKKKKK: {:?}", request);
                    keep_alive = self.check_connection_headers(&request);
                    println!("keep_alive: 1");
                    // Traiter la requête immédiatement au lieu d'utiliser une queue
                    let mut requests_to_process = Vec::new();
                    println!("keep_alive: 2");

                    
                    // Collecter les requêtes de la queue
                    if let Some(queue) = self.request_queues.get_mut(&fd) {
                    println!("keep_alive: 3");

                        if queue.push(request).is_ok() {
                    println!("keep_alive: 6");

                            while let Some(req) = queue.pop() {
                                requests_to_process.push(req);
                            }
                        } else {
                            // La queue est pleine
                            let error_response = HttpResponse::internal_server_error(None);
                            stream.write_all(&error_response.to_bytes())?;
                            keep_alive = false;
                            continue;
                        }
                    }
                    println!("keep_alive: 4");

    
                    // Traiter toutes les requêtes collectées
                    for req in requests_to_process {
                        let response = self.process_request(req);
                        let final_response = add_connection_headers(response, keep_alive);
                        stream.write_all(&final_response.to_bytes())?;
                    }
                    println!("keep_alive: 5");
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    println!("Timeout de lecture de la requête");
                    keep_alive = false;
                }
                Err(e) => {
                    eprintln!("Erreur de lecture de la requête: {:?}", e);
                    return Err(e);
                }
            }
        }

        // Nettoyer la queue quand la connexion est fermée
        self.request_queues.remove(&fd);
        Ok(())
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