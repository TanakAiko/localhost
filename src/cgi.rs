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
    println!("///////////////////////////////////////////////////session_route");

    let mut session_manager = SessionManager::global()
        .lock()
        .expect("Failed to lock session manager");

    // Obtenir les routes de session
    let session_routes = SessionManager::get_default_routes();

    // Vérifier si c'est une route de session
    if let Some(session_route) = session_routes.get(&request.path) {
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
        } else {
            // Pour /session
            return HttpResponse::page_server(
                200,
                session_route
                    .default_file
                    .as_deref()
                    .unwrap_or("session.html"),
                error_page,
            );
        }
    }

    // Gestion spéciale pour la création de session
    /* if request.path == "/create-session" && request.method == "POST" {
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
    } */

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
               // let path_str = default_file;
                let file_path = Path::new(&path_str);
                if file_path.exists() {
                    if let Some(cgi) = &route.cgi {
                        let cgi_handler = CGIHandler::new(cgi, &path_str, &request.headers);
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
