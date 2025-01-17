use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
// use std::collections::HashMap;
// use std::io::Read;
// use std::{
//     fs::{self, File},
//     io::Write,
//     path::Path,
// };
pub fn handle_delete(request: HttpRequest, error_page: Option<HashMap<u16, String>>) -> HttpResponse {
    // Only handle DELETE requests
    if request.method != "DELETE" {
        return HttpResponse::method_not_allowed(error_page);
    }

    
    // Extract the file path from the URL
    // Remove potential URL encoding and leading/trailing slashes
    let key = request.path.split("key=").nth(1).unwrap_or("");
    // key will be "hacker.png"
    
    // Construct the full path in the upload directory
    let upload_dir = Path::new("./public/upload");
    let full_path = upload_dir.join(key);
    
    println!("{:?}", full_path);
    // Security check: Ensure the resulting path is within the upload directory
    if !full_path.starts_with(upload_dir) {
        return HttpResponse::forbidden(error_page);
    }

    // Check if file exists
    if !full_path.exists() {
        return HttpResponse::not_found(error_page);
    }

    // Attempt to delete the file
    match fs::remove_file(&full_path) {
        Ok(_) => {
            HttpResponse {
                status_code: 200,
                // status_text: "OK".to_string(),
                headers: vec![
                    ("Content-Type".to_string(), "text/html".to_string()),
                    ("Content-Length".to_string(), 0.to_string()),
                ],
                body: Vec::new(),
            }
        },
        Err(_) => HttpResponse::internal_server_error(error_page),
    }
}
    

