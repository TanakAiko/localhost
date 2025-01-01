use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use crate::http_request::HttpRequest;

pub struct CGIHandler {
    pub script_path: String,
    pub cgi_executable: String,
    pub root_path: String,
    pub content_length: Option<usize>,
    pub is_chunked: bool,
}

impl CGIHandler {
    pub fn new(cgi_executable: &str, root: &str, script_path: &str, headers: &HashMap<String, String>) -> Self {
        CGIHandler {
            script_path: script_path.to_string(),
            cgi_executable: cgi_executable.to_string(),
            root_path: root.to_string(),
            content_length: headers.get("Content-Length").and_then(|l| l.parse().ok()),
            is_chunked: headers.get("Transfer-Encoding")
                .map_or(false, |t| t.to_lowercase() == "chunked"),
        }
    }

    // pub fn execute(&self, request_body: &[u8]) -> std::io::Result<Vec<u8>> {
    //     let full_path = Path::new(&self.root_path).join(&self.script_path);
        
    //     let mut child = Command::new(&self.cgi_executable)
    //         .arg(&full_path)
    //         .stdin(Stdio::piped())
    //         .stdout(Stdio::piped())
    //         .spawn()?;

    //     if let Some(mut stdin) = child.stdin.take() {
    //         stdin.write_all(request_body)?;
    //     }

    //     let output = child.wait_with_output()?;
    //     Ok(output.stdout)
    // }
    pub fn execute(&self, request_body: &[u8]) -> std::io::Result<Vec<u8>> {
        let full_path = Path::new(&self.root_path).join(&self.script_path);
        
        let mut command = Command::new(&self.cgi_executable);
        command.arg(&full_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .env("CONTENT_LENGTH", request_body.len().to_string())
            .env("CONTENT_TYPE", "application/x-www-form-urlencoded")
            .env("REQUEST_METHOD", "POST")
            .env("SCRIPT_FILENAME", full_path.to_str().unwrap_or(""))
            .env("SCRIPT_NAME", &self.script_path);

        let mut child = command.spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(request_body)?;
        }

        let output = child.wait_with_output()?;
        Ok(output.stdout)
    }

    pub fn handle_request(&self, request: &HttpRequest) -> std::io::Result<Vec<u8>> {
        if self.is_chunked {
            self.handle_chunked(&request.body)
        } else {
            self.handle_unchunked(&request.body)
        }
    }

    fn handle_chunked(&self, input: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut reader = std::io::Cursor::new(input);
        let mut body = Vec::new();
        
        while let Ok(chunk_size) = read_chunk_size(&mut reader) {
            if chunk_size == 0 {
                break;
            }
            
            let mut chunk = vec![0; chunk_size];
            reader.read_exact(&mut chunk)?;
            body.extend(chunk);
            
            // Skip CRLF
            reader.read_exact(&mut [0; 2])?;
        }
        
        self.execute(&body)
    }

    fn handle_unchunked(&self, input: &[u8]) -> std::io::Result<Vec<u8>> {
        self.execute(input)
    }
}

fn read_chunk_size(input: &mut impl Read) -> std::io::Result<usize> {
    let mut size_bytes = Vec::new();
    let mut byte = [0u8; 1];
    
    // Read until CRLF
    while input.read_exact(&mut byte).is_ok() && byte[0] != b'\n' {
        if byte[0] != b'\r' {
            size_bytes.push(byte[0]);
        }
    }
    
    // Parse hexadecimal size
    let size_str = String::from_utf8_lossy(&size_bytes);
    Ok(usize::from_str_radix(size_str.trim(), 16).unwrap_or(0))
}