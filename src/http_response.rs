pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl HttpResponse {
    // Create a new http_response
    pub fn new(status_code: u16, headers: Vec<(String, String)>, body: String) -> Self {
        Self {
            status_code,
            headers,
            body,
        }
    }

    // Generate a ok_response (200 OK)
    pub fn ok(body: &str) -> Self {
        Self {
            status_code: 200,
            headers: vec![
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body: body.to_string(),
        }
    }

     // Generate a bad_request_response (400 Bad Request)
     pub fn bad_request() -> Self {
        let body = "Bad Request";
        Self {
            status_code: 400,
            headers: vec![
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body: body.to_string(),
        }
    }
    // Generate a forbidden_response (403 Forbidden)
    //  Le serveur a compris la requête, mais refuse de l'exécuter à cause d'un manque de permissions.
    pub fn forbidden() -> Self {
        let body = "Forbidden";
        Self {
            status_code: 403,
            headers: vec![
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body: body.to_string(),
        }
    }
    // Generate a not_found_response (404 Not Found)
    pub fn not_found() -> Self {
        let body = "Not Found";
        Self {
            status_code: 404,
            headers: vec![
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body: body.to_string(),
        }
    }
    // Generate a method_not_allowed_response (405 Method Not Allowed)
    //  La méthode HTTP utilisée (GET, POST, PUT, DELETE, etc.) n'est pas autorisée pour cette ressource.
    pub fn method_not_allowed() -> Self {
        let body = "Method Not Allowed";
        Self {
            status_code: 405,
            headers: vec![
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body: body.to_string(),
        }
    }
    // Generate a payload_too_large_response (413 Payload Too Large)
    // La taille du corps de la requête dépasse les limites acceptées par le serveur.
    pub fn payload_too_large() -> Self {
        let body = "Payload Too Large";
        Self {
            status_code: 413,
            headers: vec![
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body: body.to_string(),
        }
    }
    // Generate a internal_server_error_response (500 Internal Server Error)
    // Une erreur générique lorsque le serveur rencontre un problème inattendu.
    pub fn internal_server_error() -> Self {
        let body = "Internal Server Error";
        Self {
            status_code: 500,
            headers: vec![
                ("Content-Type".to_string(), "text/plain".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body: body.to_string(),
        }
    }
    
    // Generate the http_response structure to a good format to send
    pub fn to_string(&self) -> String {
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\r\n");
        format!(
            "HTTP/1.1 {} {}\r\n{}\r\n\r\n{}",
            self.status_code,
            self.reason_phrase(),
            headers,
            self.body
        )
    }

    // Message corresponding to each response's status
    fn reason_phrase(&self) -> &str {
        match self.status_code {
            200 => "OK",
            404 => "Not Found",
            400 => "Bad Request",
            _ => "Unknown",
        }
    }
}
