use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use multipart::server::Multipart;
use std::io::Cursor;
use std::io::Read;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn handle_post(request: HttpRequest) -> HttpResponse {
    if request.method == "GET" {
        return HttpResponse::upload_dir();
    }

    let content_type = request
        .headers
        .get("Content-Type")
        .unwrap_or(&"".to_string())
        .clone();

    if !content_type.starts_with("multipart/form-data;") {
        return HttpResponse::bad_request();
    }

    // Extract boundary
    let boundary = match extract_boundary(&content_type) {
        Some(b) => b,
        None => return HttpResponse::bad_request()
    };

    let boundary = boundary.clone();

    // Decode the body into binary
    let body_bytes = request.body;

    let mut multipart = Multipart::with_body(Cursor::new(body_bytes), boundary);

    while let Ok(Some(mut field)) = multipart.read_entry() {
        if let Some(file_name) = field.headers.filename.clone() {
            let upload_dir = Path::new("./public/upload");

            if let Err(err) = fs::create_dir_all(upload_dir) {
                eprintln!("Failed to create upload directory: {}", err);
                return HttpResponse::internal_server_error();
            }

            let save_path = upload_dir.join(file_name);

            let mut file = match File::create(save_path) {
                Ok(f) => f,
                Err(_) => return HttpResponse::bad_request(),
            };
            let mut buffer = Vec::new();
            if let Err(_) = field.data.read_to_end(&mut buffer) {
                return HttpResponse::bad_request();
            };

            if let Err(_) = file.write_all(&buffer){
                return HttpResponse::bad_request();
            };

            return HttpResponse::upload_dir();
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
