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
