use std::{
    fs::{self},
    path::Path,
};

use urlencoding::decode;

use crate::{config::RouteConfig, file_upload::handle_post, http_request::HttpRequest};

#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    // Create a new http_response
    pub fn new(status_code: u16, headers: Vec<(String, String)>, body: Vec<u8>) -> Self {
        Self {
            status_code,
            headers,
            body,
        }
    }

    pub fn get_static(request: HttpRequest) -> Self {
        if let Some((mime_type, content)) = Self::serve_static_file(&request.path) {
            //println!("content.len(): {:?}", content.len());

            // let name = format!("output.{}", mime_type.split_once("/").unwrap().1);
            // println!("name: {}", name);
            // let mut file = File::create(name).unwrap();
            // file.write_all(&content).unwrap();

            return Self {
                status_code: 200,
                headers: vec![
                    ("Content-Type".to_string(), mime_type),
                    ("Content-Length".to_string(), content.len().to_string()),
                ],
                body: content,
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
            //"/" => Self::page_server("./public/index.html"),
            "/upload" => Self::handle_post_response(request),
            _ => {
                println!("route_config: {:?}", route_config);
                let path_str = &format!(
                    ".{}/{}",
                    route_config.root.clone().unwrap_or("".to_string()),
                    route_config.default_file.clone().unwrap_or("".to_string())
                );
                println!("\npath_str: {}\n", path_str);

                let file_path = Path::new(path_str);

                if file_path.exists() {
                    return Self::page_server(path_str);
                }

                Self {
                    status_code: 200,
                    headers: vec![
                        ("Content-Type".to_string(), "text/html".to_string()),
                        ("Content-Length".to_string(), request.path.len().to_string()),
                    ],
                    body: request.path.into_bytes(),
                }
            }
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
            body: body.into_bytes(),
        }
    }

    pub fn upload_dir() -> Self {
        let template = match fs::read_to_string("./public/import.html") {
            Ok(temp) => temp,
            Err(_) => return Self::internal_server_error(),
        };

        let content = Self::list_upload_content();

        let body = template.replace("{{content}}", &content);

        Self {
            status_code: 200,
            headers: vec![
                ("Content-Type".to_string(), "text/html".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body: body.into_bytes(),
        }
    }

    fn list_upload_content() -> String {
        let cont = match fs::read_dir("./public/upload") {
            Ok(entries) => {
                let mut content = String::new();
                for entry in entries {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(_) => return "".to_string(),
                    };
                    let file_name = entry.file_name();
                    let file_name = file_name.to_string_lossy();

                    let file_type = match entry.file_type() {
                        Ok(ft) => ft,
                        Err(_) => return "".to_string(),
                    };

                    if file_type.is_dir() {
                        content.push_str(&format!(
                            "<li>[Folder] <a href=\"upload/{0}/\">{0}</a></li>",
                            file_name
                        ));
                    } else {
                        content.push_str(&format!(
                            "<li><a href=\"upload/{0}\">{0}</a></li>",
                            file_name
                        ));
                    }
                }
                content
            }
            Err(_) => "".to_string(),
        };
        cont
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
            body: body.into_bytes(),
        }
    }

    // Generate the http_response structure to a good format to send
    pub fn to_bytes(&self) -> Vec<u8> {
        let headers = self
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}\r\n", k, v))
            .collect::<String>();

        let response_text = format!(
            "HTTP/1.1 {} {}\r\n{}\r\n",
            self.status_code,
            self.reason_phrase(),
            headers
        );

        let mut response_bytes = response_text.into_bytes();

        response_bytes.extend_from_slice(&self.body);

        response_bytes
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
        let decoded_path = match decode(path) {
            Ok(data) => data,
            Err(_) => return None,
        };

        let file_path = format!("public{}", decoded_path);
        println!("file_path: '{}'", file_path);
        if Path::new(&file_path).exists() {
            let content = fs::read(&file_path).ok()?;
            let mime_type = if path.ends_with(".css") {
                "text/css"
            } else if path.ends_with(".js") {
                "application/javascript"
            } else if path.ends_with(".html") {
                "text/html"
            } else if path.ends_with(".png") {
                "image/png"
            } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
                "image/jpeg"
            } else if path.ends_with(".gif") {
                "image/gif"
            } else if path.ends_with(".svg") {
                "image/svg+xml"
            } else if path.ends_with(".txt") {
                "text/plain"
            } else if path.ends_with(".pdf") {
                "application/pdf"
            } else if path.ends_with(".doc") || path.ends_with(".docx") {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            } else if path.ends_with(".xls") || path.ends_with(".xlsx") {
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            } else {
                "application/octet-stream" // Type par défaut pour les fichiers inconnus
            };
            Some((mime_type.to_string(), content))
        } else {
            None
        }
    }
}
