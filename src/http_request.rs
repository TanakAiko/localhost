use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    pub fn from_raw(raw_request: &str) -> Option<Self> {
        let mut lines = raw_request.split("\r\n");

        // 1. Parse the request line
        let request_line = lines.next()?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_string();
        let path = parts.next()?.to_string();
        let version = parts.next()?.to_string();

        // 2. Parse headers
        let mut headers = HashMap::new();
        for line in lines.by_ref() {
            if line.is_empty() {
                break; // Empty line indicates end of headers
            }
            if let Some((key, value)) = line.split_once(": ") {
                headers.insert(key.to_string(), value.to_string());
            }
        }

        // 3. Parse body (optional)
        let body = lines.collect::<Vec<&str>>().join("\r\n");
        let body = if body.is_empty() { "".to_string() } else { body };

        Some(Self {
            method,
            path,
            version,
            headers,
            body,
        })
    }
}
