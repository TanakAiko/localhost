use std::{
    collections::HashMap,
    fs::{self},
    path::Path,
    time::SystemTime,
};

use urlencoding::decode;

use crate::{
    cgi::handle_route, config::RouteConfig, delete_file::handle_delete, file_upload::handle_post,
    http_request::HttpRequest, session::Session,
};
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

    pub fn with_keep_alive(mut self, keep_alive: bool) -> Self {
        let connection_value = if keep_alive { "keep-alive" } else { "close" };
        self.headers
            .push(("Connection".to_string(), connection_value.to_string()));

        if keep_alive {
            self.headers
                .push(("Keep-Alive".to_string(), "timeout=5, max=100".to_string()));
        }

        self
    }

    pub fn with_session(mut self, session: &Session) -> Self {
        // Add or update the session cookie
        self.set_cookie("session_id", &session.id, Some(session.expires_at));
        self
    }

    pub fn set_cookie(&mut self, name: &str, value: &str, expires: Option<SystemTime>) {
        let cookie = match expires {
            Some(exp) => format!("{}={}; Expires={:?}", name, value, exp),
            None => format!("{}={}", name, value),
        };
        self.headers.push(("Set-Cookie".to_string(), cookie));
    }

    pub fn get_static(request: HttpRequest, error_page: Option<HashMap<u16, String>>) -> Self {
        if let Some((mime_type, content)) = Self::serve_static_file(&request.path) {
            return Self {
                status_code: 200,
                headers: vec![
                    ("Content-Type".to_string(), mime_type),
                    ("Content-Length".to_string(), content.len().to_string()),
                ],
                body: content,
            };
        }

        println!("Not found (get_static)");
        Self::not_found(error_page)
    }

    // Generate a ok_response (200 OK)
    pub fn ok(
        request: HttpRequest,
        route_config: &RouteConfig,
        error_page: Option<HashMap<u16, String>>,
        size_limit: Option<usize>,
    ) -> Self {
        let methodes = match route_config.accepted_methods.clone() {
            Some(methode) => methode,
            None => return Self::bad_request(error_page),
        };

        if !methodes.contains(&request.method) {
            return Self::method_not_allowed(error_page);
        }

        match request.path.as_str() {
            "/upload" => Self::handle_post_response(request, error_page, size_limit),
            "/delete" => handle_delete(request, error_page),
            _ => handle_route(route_config, request, error_page),
        }
    }

    pub fn from_cgi_output(
        output: (Vec<u8>, Vec<u8>),
        error_page: Option<HashMap<u16, String>>,
    ) -> Self {
        let (stdout, stderr) = output;
        if !stderr.is_empty() {
            match String::from_utf8(stderr) {
                Ok(body) => HttpResponse {
                    status_code: 200,
                    headers: vec![("Content-Type".to_string(), "text/html".to_string())],
                    body: body.into(),
                },
                Err(_) => HttpResponse::internal_server_error(error_page),
            }
        } else {
            match String::from_utf8(stdout) {
                Ok(body) => HttpResponse {
                    status_code: 200,
                    headers: vec![("Content-Type".to_string(), "text/html".to_string())],
                    body: body.into(),
                },
                Err(_) => HttpResponse::internal_server_error(error_page),
            }
        }
    }

    pub fn handle_post_response(
        request: HttpRequest,
        error_page: Option<HashMap<u16, String>>,
        size_limit: Option<usize>,
    ) -> Self {
        if request.body.len() > size_limit.unwrap_or(0) {
            return Self::payload_too_large(error_page);
        };

        handle_post(request, error_page)
    }

    // Generate a bad_request_response (400 Bad Request)
    pub fn bad_request(error_page: Option<HashMap<u16, String>>) -> Self {
        Self::error_template(400, "Bad Request", error_page)
    }

    // Generate a forbidden_response (403 Forbidden)
    //  The server understood the request, but refuses to execute it because of a lack of permissions.
    pub fn forbidden(error_page: Option<HashMap<u16, String>>) -> Self {
        Self::error_template(403, "Forbidden", error_page)
    }

    // Generate a not_found_response (404 Not Found)
    pub fn not_found(error_page: Option<HashMap<u16, String>>) -> Self {
        Self::error_template(404, "Not Found", error_page)
    }

    // Generate a method_not_allowed_response (405 Method Not Allowed)
    //  The HTTP method used (Get, Post, Put, Delete, etc.) is not allowed for this resource.
    pub fn method_not_allowed(error_page: Option<HashMap<u16, String>>) -> Self {
        Self::error_template(405, "Method Not Allowed", error_page)
    }

    // Generate a service_unavailable_response (503 Service Unavailable)
    pub fn service_unavailable(error_page: Option<HashMap<u16, String>>) -> Self {
        Self::error_template(503, "Service Unavailable", error_page)
    }

    // Generate a payload_too_large_response (413 Payload Too Large)
    // The size of the request body exceeds the limits accepted by the server.
    pub fn payload_too_large(error_page: Option<HashMap<u16, String>>) -> Self {
        Self::error_template(413, "Payload Too Large", error_page)
    }

    // Generate a internal_server_error_response (500 Internal Server Error)
    // A generic error when the server encounters an unexpected problem.
    pub fn internal_server_error(error_page: Option<HashMap<u16, String>>) -> Self {
        Self::error_template(500, "Internal Server Error", error_page)
    }

    fn error_template(
        status_code: u16,
        message: &str,
        error_page: Option<HashMap<u16, String>>,
    ) -> Self {
        if let Some(custom_routes) = error_page.clone() {
            // println!("error_page: {:?}", error_page);
            if let Some(custom_path) = custom_routes.get(&status_code) {
                println!("custom_path: {:?}", custom_path);
                let good_path = &format!(".{}", custom_path);
                let path = Path::new(good_path);
                if path.exists() {
                    return Self::page_server(status_code, &custom_path, error_page);
                }
            }
        }

        // Read the HTML file
        let template = match fs::read_to_string("./public/error.html") {
            Ok(temp) => temp,
            Err(_) => return Self::internal_server_error(error_page),
        };

        // Replace the reserved spaces
        let body = template
            .replace("{{status_code}}", &status_code.to_string())
            .replace("{{message}}", message);

        // Create the HTTP response
        Self {
            status_code,
            headers: vec![
                ("Content-Type".to_string(), "text/html".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body: body.into_bytes(),
        }
    }

    pub fn list_dir(dir: String, error_page: Option<HashMap<u16, String>>) -> Self {
        let template = match fs::read_to_string("./public/list_dir.html") {
            Ok(temp) => temp,
            Err(_) => return Self::internal_server_error(error_page),
        };

        let content = Self::list_content(dir);
        if content == "!existe" {
            return Self::internal_server_error(error_page);
        }

        let script = r#"
            <script>
            function deleteFile(filePath) {
                if (confirm("Are you sure you want to delete this file?")) {
                    fetch('/delete', {
                        method: 'DELETE',
                        headers: {
                            'Content-Type': 'application/json',
                        },
                        body: JSON.stringify({ path: filePath })
                    })
                    .then(response => {
                        if (response.ok) {
                            // Recharge la page en cas de succès
                            window.location.reload();
                        }
                    })
                    .catch(error => {
                        console.error('Erreur:', error);
                        alert("Une erreur s'est produite lors de la suppression");
                    });
                }
            }
            </script>
            "#;

        let body = template.replace("{{content}}", &content) + script;

        Self {
            status_code: 200,
            headers: vec![
                ("Content-Type".to_string(), "text/html".to_string()),
                ("Content-Length".to_string(), body.len().to_string()),
            ],
            body: body.into_bytes(),
        }
    }

    fn list_content(dir: String) -> String {
        let path_str = format!("./public{}", dir);
        println!("dir: {}", dir);
        let cont = match fs::read_dir(path_str) {
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

                    let mut buton = String::new();
                    if dir == "/upload" {
                        buton = format!(
                            "<button type=\"button\" class=\"delete-btn\" onclick=\"deleteFile('{}')\">
                             <i class=\"fas fa-trash-alt\"></i> Delete
                             </button>",
                            file_name
                        );
                    }

                    if file_type.is_dir() {
                        content.push_str(&format!(
                            "<div class=\"file-info\">
                            <div class=\"file-icon image-file\">
							<i class=\"fas fa-image\"></i>
						    </div>
                                <span[Folder]  class=\"file-name\" > <a class=\"file-link\" href=\"{}/{}\">{}</a>{}</span>
                            </div>
                            ",
                            dir.trim_start_matches("/"),
                            file_name,
                            file_name,
                            buton
                        ));
                    } else {
                        content.push_str(&format!(
                            "<li class=\"file-item\">
                                <div class=\"file-info\">
                                    <div class=\"file-icon image-file\">
                                        <i class=\"fas fa-image\"></i>
                                    </div>
                                    <span class=\"file-name\"> <a class=\"file-link\" href=\"{}/{}\">{}</a> </span>
                                </div>
                                <div class=\"file-actions\">
                                    {}
                                </div>
                            </li>",
                            dir.trim_start_matches("/"),
                            file_name,
                            file_name,
                            buton
                        ));
                    }
                }
                content
            }
            Err(_) => "!existe".to_string(),
        };
        cont
    }

    pub fn page_server(
        status_code: u16,
        path: &str,
        error_page: Option<HashMap<u16, String>>,
    ) -> Self {
        let real_path = format!("./public/{}", path);
        let body = match fs::read_to_string(real_path) {
            Ok(temp) => temp,
            Err(_) => return Self::internal_server_error(error_page),
        };

        Self {
            status_code,
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
            503 => "Service Unavailable",
            _ => "Unknown",
        }
    }

    fn serve_static_file(path: &str) -> Option<(String, Vec<u8>)> {
        let decoded_path = match decode(path) {
            Ok(data) => data,
            Err(_) => return None,
        };

        let file_path = format!("public{}", decoded_path);

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
                "application/octet-stream" // Default type for unknown files
            };
            Some((mime_type.to_string(), content))
        } else {
            None
        }
    }
}
