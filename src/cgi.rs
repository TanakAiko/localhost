use std::process::Command;
use std::fs;
use crate::config::RouteConfig;

fn handle_route(route: &RouteConfig, path: &str, body: Option<&str>) -> String {
    if let Some(cgi_path) = &route.cgi {
        if path.ends_with(".py") || path.ends_with(".php") {
            return execute_cgi(cgi_path, path, body);
        }
    }

    // Gérer les redirections
    if let Some(redirection) = &route.redirection {
        return format!("HTTP/1.1 301 Moved Permanently\r\nLocation: {}\r\n\r\n", redirection);
    }

    // Gérer le fichier par défaut ou le listing de répertoire
    if let Some(listen) = route.directory_listing {
        if listen {
            return list_directory(route.root.as_deref().unwrap_or("./"));
        }
    }

    // Autres cas: Retourner une erreur
    "HTTP/1.1 404 Not Found\r\n\r\nPage Not Found".to_string()
}

fn execute_cgi(cgi_path: &str, file_path: &str, body: Option<&str>) -> String {
    let mut command = Command::new(cgi_path);
    command.arg(file_path);

    if let Some(input) = body {
        command.arg(input);
    }

    match command.output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => format!("HTTP/1.1 500 Internal Server Error\r\n\r\n{}", e),
    }
}


fn list_directory(directory: &str) -> String {
    let paths = fs::read_dir(directory).unwrap();
    let mut listing = String::new();

    for path in paths {
        listing.push_str(&format!("<li>{}</li>", path.unwrap().path().display()));
    }

    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n
        <html><body><ul>{}</ul></body></html>",
        listing
    )
}
