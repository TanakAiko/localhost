use std::{fs, path::Path};

use crate::{config::RouteConfig, file_upload::handle_post, http_request::HttpRequest};

#[derive(Debug)]
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

    pub fn get_static(request: HttpRequest) -> Self {
        if let Some((mime_type, content)) = Self::serve_static_file(&request.path) {
            return Self {
                status_code: 200,
                headers: vec![
                    ("Content-Type".to_string(), mime_type),
                    ("Content-Length".to_string(), content.len().to_string()),
                ],
                body: String::from_utf8(content).unwrap_or_default(), // Si binaire, utilisez directement `content`.
            };
        }

        Self::not_found()
    }

    // Generate a ok_response (200 OK)
    pub fn ok(request: HttpRequest, route_config: &RouteConfig) -> Self {
        let methodes = match route_config.accepted_methods.clone() {
            Some(methode) => methode,
            None => return Self::bad_request(),
        };

        if !methodes.contains(&request.method) {
            return Self::bad_request();
        }

        match request.path.as_str() {
            "/" => Self::page_server("./public/index.html"),
            "/uploading" => Self::handle_post_response(request),
            _ => Self {
                status_code: 200,
                headers: vec![
                    ("Content-Type".to_string(), "text/html".to_string()),
                    ("Content-Length".to_string(), request.path.len().to_string()),
                ],
                body: request.path.to_string(),
            },
        }
    }

    //im handling post here
    pub fn handle_post_response(request: HttpRequest) -> Self {
        const MAX_BODY_SIZE: usize = 1024 * 1024; //1MB

        if request.body.len() > MAX_BODY_SIZE {
            return Self::payload_too_large();
        };

        handle_post(request)
    }

    // Generate a bad_request_response (400 Bad Request)
    pub fn bad_request() -> Self {
        Self::error_template(400, "Bad Request")
    }

    // Generate a forbidden_response (403 Forbidden)
    //  Le serveur a compris la requête, mais refuse de l'exécuter à cause d'un manque de permissions.
    pub fn forbidden() -> Self {
        Self::error_template(403, "Forbidden")
    }

    // Generate a not_found_response (404 Not Found)
    pub fn not_found() -> Self {
        Self::error_template(404, "Not Found")
    }

    // Generate a method_not_allowed_response (405 Method Not Allowed)
    //  La méthode HTTP utilisée (GET, POST, PUT, DELETE, etc.) n'est pas autorisée pour cette ressource.
    pub fn method_not_allowed() -> Self {
        Self::error_template(405, "Method Not Allowed")
    }

    // Generate a payload_too_large_response (413 Payload Too Large)
    // La taille du corps de la requête dépasse les limites acceptées par le serveur.
    pub fn payload_too_large() -> Self {
        Self::error_template(413, "Payload Too Large")
    }

    // Generate a internal_server_error_response (500 Internal Server Error)
    // Une erreur générique lorsque le serveur rencontre un problème inattendu.
    pub fn internal_server_error() -> Self {
        Self::error_template(500, "Internal Server Error")
    }

    fn error_template(status_code: u16, message: &str) -> Self {
        // Lire le fichier HTML
        let template = match fs::read_to_string("./public/error.html") {
            Ok(temp) => temp,
            Err(_) => return Self::internal_server_error(),
        };

        // Remplacer les espaces réservés
        let body = template
            .replace("{{status_code}}", &status_code.to_string())
            .replace("{{message}}", message);

        // Créer la réponse HTTP
        Self {
            status_code,
            headers: vec![
                ("Content-Type".to_string(), "text/html".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body,
        }
    }

    pub fn page_server(path: &str) -> Self {
        let body = match fs::read_to_string(path) {
            Ok(temp) => temp,
            Err(_) => return Self::internal_server_error(),
        };

        Self {
            status_code: 200,
            headers: vec![
                ("Content-Type".to_string(), "text/html".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body,
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
            400 => "Bad Request",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            413 => "Payload Too Large",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }

    fn serve_static_file(path: &str) -> Option<(String, Vec<u8>)> {
        let file_path = format!("public{}", path); // Tous les fichiers statiques sont dans un dossier 'public'
        if Path::new(&file_path).exists() {
            let content = fs::read(&file_path).ok()?;
            let mime_type = if path.ends_with(".css") {
                "text/css"
            } else if path.ends_with(".js") {
                "application/javascript"
            } else if path.ends_with(".html") {
                "text/html"
            } else {
                "text/plain"
            };
            Some((mime_type.to_string(), content))
        } else {
            None
        }
    }
}
