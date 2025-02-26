use std::{collections::HashMap, os::fd::RawFd};

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub stream_fd: RawFd,
    pub listener_fd: RawFd,
}

impl HttpRequest {
    pub fn is_http_1_1(&self) -> bool {
        self.version == "HTTP/1.1"
    }

    pub fn wants_keep_alive(&self) -> bool {
        if self.is_http_1_1() {
            // In HTTP/1.1, the connection is keep-alive by default
            self.headers
                .get("Connection")
                .map_or(true, |v| v.to_lowercase() != "close")
        } else {
            // In HTTP/1.0, the connection is closed by default
            self.headers
                .get("Connection")
                .map_or(false, |v| v.to_lowercase() == "keep-alive")
        }
    }

    // Sprinkle a raw http request from bytes.
    pub fn from_raw(raw_request: &[u8], listener_fd: RawFd, stream_fd: RawFd) -> Option<Self> {
        // Find the end of the headers (separated by "\r\n\r\n")
        let headers_end = raw_request
            .windows(4)
            .position(|window| window == b"\r\n\r\n")?;
        let (header_bytes, body_bytes) = raw_request.split_at(headers_end + 4);

        // Convert the headers to a UTF-8 chain (necessary for the HTTP structure)
        let headers_str = std::str::from_utf8(&header_bytes).ok()?;

        let mut lines = headers_str.split("\r\n");

        // Sprinkle the request line
        let request_line = lines.next()?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_string();
        let path = parts.next()?.to_string();
        let version = parts.next()?.to_string();

        // Sprinkle the headers
        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(": ") {
                headers.insert(key.to_string(), value.to_string());
            }
        }

        // The body is already isolated in the form of bytes
        let body = body_bytes.to_vec();

        Some(Self {
            method,
            path,
            version,
            headers,
            body,
            listener_fd,
            stream_fd,
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
