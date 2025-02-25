use std::{collections::HashMap, os::fd::RawFd};

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub fd: RawFd, // Ajout de la propriété fd pour le stateful
}

impl HttpRequest {
    pub fn is_http_1_1(&self) -> bool {
        self.version == "HTTP/1.1"
    }

    pub fn wants_keep_alive(&self) -> bool {
        if self.is_http_1_1() {
            // En HTTP/1.1, la connexion est keep-alive par défaut
            self.headers
                .get("Connection")
                .map_or(true, |v| v.to_lowercase() != "close")
        } else {
            // En HTTP/1.0, la connexion est close par défaut
            self.headers
                .get("Connection")
                .map_or(false, |v| v.to_lowercase() == "keep-alive")
        }
    }

    /// Parse une requête HTTP brute à partir d'octets.
    pub fn from_raw(raw_request: &[u8], fd: RawFd) -> Option<Self> {
        // Trouver la fin des en-têtes (séparé par "\r\n\r\n")
        let headers_end = raw_request
            .windows(4)
            .position(|window| window == b"\r\n\r\n")?;
        let (header_bytes, body_bytes) = raw_request.split_at(headers_end + 4);

        // Convertir les en-têtes en une chaîne UTF-8 (nécessaire pour la structure HTTP)
        let headers_str = std::str::from_utf8(&header_bytes).ok()?;

        let mut lines = headers_str.split("\r\n");

        // 1. Parse la ligne de requête
        let request_line = lines.next()?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_string();
        let path = parts.next()?.to_string();
        let version = parts.next()?.to_string();

        // 2. Parse les en-têtes
        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break; // Fin des en-têtes
            }
            if let Some((key, value)) = line.split_once(": ") {
                headers.insert(key.to_string(), value.to_string());
            }
        }

        // 3. Le corps est déjà isolé sous forme d'octets
        let body = body_bytes.to_vec();

        Some(Self {
            method,
            path,
            version,
            headers,
            body,
            fd,
        })
    }

    pub fn get_cookies(&self) -> HashMap<String, String> {
        let mut cookies = HashMap::new();
        if let Some(cookie_header) = self.headers.get("Cookie") {
            // Parse cookie header
            for cookie in cookie_header.split(';') {
                if let Some((name, value)) = cookie.split_once('=') {
                    cookies.insert(name.trim().to_string(), value.trim().to_string());
                }
            }
        }
        cookies
    }
}
