// route.rs

use crate::cgi_handler::*;
use crate::config::RouteConfig;
use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use crate::session::SessionManager;
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use std::time::SystemTime;

/* fn handle_session(
    session_manager: &mut SessionManager,
    session_id: Option<String>,
    response: &mut HttpResponse,
) {
    println!("Session_ids:\n {:?}", session_manager.sessions);
    if let Some(session) = session_id {
        // Valider et renouveler la session existante
        if let Some(sess) = session_manager.get_session_mut(&session) {
            println!("Session existante: {:?}", sess);
            // sess.expires_at = SystemTime::now() + session_manager.session_duration;
            sess.expires_at = SystemTime::now() + Duration::from_secs(3600);
        } else {
            // Session_invalide
            response.set_cookie("session_id", "", None);
        }
    } else {

        // Créer une nouvelle session
        let session_id = session_manager.create_session();
        response.set_cookie(
            "session_id",
            &session_id,
            Some(session_manager.get_session(&session_id).unwrap().expires_at),
        );
    }
} */

fn handle_session(
    session_manager: &mut SessionManager,
    session_id: Option<String>,
    request: &HttpRequest,
) -> Result<String, HttpResponse> {
    // Ne pas vérifier la session pour ces routes spécifiques
    let public_paths = vec!["/session", "/create-session"];
    if public_paths.contains(&request.path.as_str()) {
        return Ok(String::new());
    }

    match session_id {
        Some(id) => {
            if let Some(sess) = session_manager.get_session_mut(&id) {
                // Session valide - renouveler
                sess.expires_at = SystemTime::now() + Duration::from_secs(3600);
                Ok(id)
            } else {
                // Session invalide - rediriger
                Err(HttpResponse {
                    status_code: 302,
                    headers: vec![
                        ("Location".to_string(), "/session".to_string()),
                        (
                            "Set-Cookie".to_string(),
                            "session_id=; Max-Age=0".to_string(),
                        ),
                    ],
                    body: Vec::new(),
                })
            }
        }
        None => {
            // Pas de session - rediriger
            Err(HttpResponse {
                status_code: 302,
                headers: vec![("Location".to_string(), "/session".to_string())],
                body: Vec::new(),
            })
        }
    }
}

pub fn handle_route(
    route: &RouteConfig,
    request: HttpRequest,
    error_page: Option<HashMap<u16, String>>,
) -> HttpResponse {
    // let session_id = request.get_cookies().get("session_id").cloned();
    // let mut session_manager = SessionManager::new(Duration::from_secs(3600)); // 1 heure

    let mut session_manager = SessionManager::global()
        .lock()
        .expect("Failed to lock session manager");

    // Gestion spéciale pour la création de session
    if request.path == "/create-session" && request.method == "POST" {
        let session_id = session_manager.create_session();
        return HttpResponse {
            status_code: 302,
            headers: vec![
                ("Location".to_string(), "/".to_string()),
                (
                    "Set-Cookie".to_string(),
                    format!("session_id={}; Path=/", session_id),
                ),
            ],
            body: Vec::new(),
        };
    }

    // Vérification de la session pour toutes les autres routes
    let session_id = request.get_cookies().get("session_id").cloned();
    match handle_session(&mut session_manager, session_id, &request) {
        Ok(_) => {
            if let Some(listing_enabled) = route.directory_listing {
                if listing_enabled {
                    println!("listing_enabled == true");
                    let response = HttpResponse::list_dir(request.path, error_page);
                    return response;
                }
            }
        
            if let Some(redirect_to) = &route.redirection {
                return HttpResponse {
                    status_code: 301,
                    headers: vec![("Location".to_string(), redirect_to.clone())],
                    body: String::new().into(),
                };
            }
        
            if let Some(default_file) = &route.default_file {
                let path_str = format!("./public/{}", default_file);
                let file_path = Path::new(&path_str);
                if file_path.exists() {
                    if let Some(cgi) = &route.cgi {
                        let cgi_handler = CGIHandler::new(cgi, &default_file, &request.headers);
                        return match cgi_handler.handle_request(&request) {
                            Ok(output) => HttpResponse::from_cgi_output(output, error_page),
                            Err(_) => HttpResponse::internal_server_error(error_page),
                        };
                    }
        
                    let response = HttpResponse::page_server(200, &default_file, error_page);
                    // let mut session_id = request.get_cookies().get("session_id").cloned();
                    // let id = session_id.clone().unwrap_or(String::new());
                    // if id.is_empty() {
                    //     session_id = None;
                    // }
                    // handle_session(&mut session_manager, session_id, &request);
                    return response;
                } else {
                    println!("file_path.exists() === false")
                }
            };
        
            println!("Not found (handle_route)");
            HttpResponse::not_found(error_page)
            // ... reste du code ...
        }
        Err(redirect_response) => redirect_response,
    }

   
}

// fn list_directory(dir: &str) -> Result<String, String> {
//     match fs::read_dir(dir) {
//         Ok(entries) => {
//             let mut html = String::from("<html><body><ul>");

//             for entry in entries {
//                 if let Ok(entry) = entry {
//                     let name = entry.file_name().to_string_lossy().to_string();
//                     html.push_str(&format!("<li>{}</li>", name));
//                 }
//             }

//             html.push_str("</ul></body></html>");
//             Ok(html)
//         }
//         Err(e) => Err(e.to_string()),
//     }
// }
