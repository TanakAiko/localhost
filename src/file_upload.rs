use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use multipart::server::Multipart;
use std::io::Cursor;
use std::io::Read;
use std::{fs::File, io::Write, path::Path};

pub fn handle_post(request: HttpRequest) -> HttpResponse {
    let content_type = request
        .headers
        .get("Content-Type")
        .unwrap_or(&"".to_string())
        .clone();

    if !content_type.starts_with("multipart/form-data;") {
        return HttpResponse::bad_request();
    }

    // Extract boundary
    let boundary = extract_boundary(&content_type);
    if boundary.is_none() {
        return HttpResponse::bad_request();
    }
    let boundary = boundary.unwrap().clone();

    // Decode the body into binary
    let body_bytes = request.body;

    let mut multipart = Multipart::with_body(Cursor::new(body_bytes), boundary);

    while let Ok(Some(mut field)) = multipart.read_entry() {
        if let Some(file_name) = field.headers.filename.clone() {
            let save_path = Path::new("./public/upload").join(file_name);

            let mut file = File::create(save_path).expect("Failed to create file");
            let mut buffer = Vec::new();
            field
                .data
                .read_to_end(&mut buffer)
                .expect("Failed to read file");
            file.write_all(&buffer).expect("Failed to write file");

            return HttpResponse::page_server("./public/import.html");
        }
    }

    HttpResponse::bad_request()
}


fn extract_boundary(content_type: &str) -> Option<String> {
    content_type
        .split("boundary=")
        .nth(1)
        .map(|b| b.to_string())
}
