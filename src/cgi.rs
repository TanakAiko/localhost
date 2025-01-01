use crate::cgi_handler::*;
use crate::config::RouteConfig;
use crate::http_request::HttpRequest;
use crate::http_response::HttpResponse;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn handle_route(
    route: &RouteConfig,
    request: HttpRequest,
    error_page: Option<HashMap<u16, String>>,
) -> HttpResponse {
    if let Some(listing_enabled) = route.directory_listing {
        if listing_enabled {
            return match list_directory(route.root.as_deref().unwrap_or("./")) {
                Ok(listing) => HttpResponse {
                    status_code: 200,
                    headers: vec![("Content-Type".to_string(), "text/html".to_string())],
                    body: listing.into(),
                },
                Err(_) => HttpResponse::internal_server_error(error_page),
            };
        }
    }

    // if let Some(cgi_path) = &route.cgi {
    // let root = route.root.as_deref().unwrap_or("./");
    if let Some(root) = &route.root {
        if let Some(default_file) = &route.default_file {
            let file_path = Path::new(root).join(default_file);
            if file_path.exists() {
                if let Some(cgi) = &route.cgi {
                    let cgi_handler = CGIHandler::new(cgi, root, &default_file, &request.headers);
                    return match cgi_handler.handle_request(&request) {
                        Ok(output) => HttpResponse::from_cgi_output(output, error_page),
                        Err(_) => HttpResponse::internal_server_error(error_page),
                    };
                }

                // return match execute_cgi(cgi_path, &file_path.to_string_lossy(), None) {
                //     Ok(output) => HttpResponse {
                //         status_code: 200,
                //         headers: vec![("Content-Type".to_string(), "text/html".to_string())],
                //         body: output.into(),
                //     },
                //     Err(_) => HttpResponse::internal_server_error(error_page),
                // };
            }
        } else {
            // prendre le nom du fichier a partir de l'url
            let file_name = Path::new(&request.path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");

            if !file_name.is_empty() && (file_name.ends_with(".py") || file_name.ends_with(".php"))
            {
                let script_path = Path::new(root).join(file_name);
                if script_path.exists() {
                    if let Some(cgi) = &route.cgi {
                        let cgi_handler = CGIHandler::new(cgi, root, file_name, &request.headers);
                        return match cgi_handler.handle_request(&request) {
                            Ok(output) => HttpResponse::from_cgi_output(output, error_page),
                            Err(_) => HttpResponse::internal_server_error(error_page),
                        };
                    }
                }
            }
        }
    }

    if let Some(redirect_to) = &route.redirection {
        return HttpResponse {
            status_code: 301,
            headers: vec![("Location".to_string(), redirect_to.clone())],
            body: String::new().into(),
        };
    }
    let path_str = &format!(
        ".{}/{}",
        route.root.clone().unwrap_or("".to_string()),
        route.default_file.clone().unwrap_or("".to_string())
    );
    // println!("\npath_str: {}\n", path_str);

    let file_path = Path::new(path_str);

    if file_path.exists() {
        return HttpResponse::page_server(200,path_str,error_page);
    }

    // HttpResponse {
    //     status_code: 200,
    //     headers: vec![
    //         ("Content-Type".to_string(), "text/html".to_string()),
    //         ("Content-Length".to_string(), request.path.len().to_string()),
    //     ],
    //     body: request.path.into_bytes(),
    // }
    // HttpResponse::
    HttpResponse::not_found(error_page)
}

fn list_directory(dir: &str) -> Result<String, String> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            let mut html = String::from("<html><body><ul>");

            for entry in entries {
                if let Ok(entry) = entry {
                    let name = entry.file_name().to_string_lossy().to_string();
                    html.push_str(&format!("<li>{}</li>", name));
                }
            }

            html.push_str("</ul></body></html>");
            Ok(html)
        }
        Err(e) => Err(e.to_string()),
    }
}

// fn execute_cgi(cgi_path: &str, file_path: &str, body: Option<&str>) -> Result<String, String> {
//     let mut cmd = Command::new(cgi_path);
//     cmd.arg(file_path);

//     if let Some(input) = body {
//         cmd.arg(input);
//     }

//     match cmd.output() {
//         Ok(output) => {
//             if output.status.success() {
//                 Ok(String::from_utf8_lossy(&output.stdout).to_string())
//             } else {
//                 Err(String::from_utf8_lossy(&output.stderr).to_string())
//             }
//         }
//         Err(e) => Err(e.to_string()),
//     }
// }
