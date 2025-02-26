use serde_json::Value;

use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use std::fs;
use std::path::Path;

pub fn handle_delete(
    request: HttpRequest,
    error_page: Option<std::collections::HashMap<u16, String>>,
) -> HttpResponse {
    let body_text = String::from_utf8_lossy(&request.body);
    
    // Set the json contained in the body
    let json_body: Value = match serde_json::from_str(&body_text) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Erreur lors du parsing du JSON: {}", e);
            return HttpResponse::bad_request(error_page);
        }
    };

    // Extract the value associated with the "path" key
    let file_name = match json_body.get("path").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => {
            eprintln!("Aucun champ 'path' trouvé dans le body JSON");
            return HttpResponse::bad_request(error_page);
        }
    };

    let base_dir = Path::new("./public/upload");
    let file_path = base_dir.join(file_name);

    // Check that the file exists and that it is indeed a file (and not a folder)
    if !file_path.exists() || !file_path.is_file() {
        return HttpResponse::not_found(error_page);
    }

    // Try to delete the file
    match fs::remove_file(&file_path) {
        Ok(_) => {
            HttpResponse {
                status_code: 200,
                headers: Vec::new(),
                body: String::new().into(),
            }
        }
        Err(err) => {
            eprintln!(
                "Erreur lors de la suppression du fichier {}: {}",
                file_path.display(),
                err
            );
            
            // In the event of an error, you can return a 403 (if permission problem) or a 500.
            if err.kind() == std::io::ErrorKind::PermissionDenied {
                HttpResponse::forbidden(error_page)
            } else {
                HttpResponse::internal_server_error(error_page)
            }
        }
    }
}
