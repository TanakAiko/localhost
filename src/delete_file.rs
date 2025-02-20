use serde_json::Value;

use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use std::fs;
use std::path::Path;

pub fn handle_delete(
    request: HttpRequest,
    error_page: Option<std::collections::HashMap<u16, String>>,
) -> HttpResponse {
    // println!("******\nrequest: {:?}", request);
    let body_text = String::from_utf8_lossy(&request.body);
    println!("Body as text: '{}'", body_text);
    // On suppose que le chemin demandé correspond au chemin relatif dans le répertoire "./public".
    // Par exemple, pour une requête DELETE sur "/uploads/file.txt", on cherchera "./public/uploads/file.txt".

    // Parser le JSON contenu dans le body
    let json_body: Value = match serde_json::from_str(&body_text) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Erreur lors du parsing du JSON: {}", e);
            return HttpResponse::bad_request(error_page);
        }
    };

    // Extraire la valeur associée à la clé "path"
    let file_name = match json_body.get("path").and_then(|v| v.as_str()) {
        Some(name) => name,
        None => {
            eprintln!("Aucun champ 'path' trouvé dans le body JSON");
            return HttpResponse::bad_request(error_page);
        }
    };

    println!("file_name: {}", file_name);

    let base_dir = Path::new("./public/upload");
    let file_path = base_dir.join(file_name);

    // Vérifier que le fichier existe et qu'il s'agit bien d'un fichier (et non d'un dossier)
    if !file_path.exists() || !file_path.is_file() {
        return HttpResponse::not_found(error_page);
    }

    // Tenter de supprimer le fichier
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
            // En cas d'erreur, on peut renvoyer un 403 (si problème de permission) ou un 500.
            if err.kind() == std::io::ErrorKind::PermissionDenied {
                HttpResponse::forbidden(error_page)
            } else {
                HttpResponse::internal_server_error(error_page)
            }
        }
    }
}
