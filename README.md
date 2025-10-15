# Localhost - Rust HTTP Server

<div align="center">

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![HTTP](https://img.shields.io/badge/HTTP-005571?style=for-the-badge&logo=http&logoColor=white)
![JSON](https://img.shields.io/badge/JSON-000000?style=for-the-badge&logo=json&logoColor=white)
![CGI](https://img.shields.io/badge/CGI-4B8BBE?style=for-the-badge&logo=python&logoColor=white)

</div>

A lightweight, configurable HTTP server implementation written in Rust from scratch, featuring support for static file serving, file uploads, CGI execution, session management, and custom routing.

> **📚 Educational Project**: This server was built from scratch as a learning project to understand HTTP protocol implementation, network programming, and Rust systems programming. It's designed to demonstrate core web server concepts without relying on high-level frameworks.

## Screenshot

<div align="center">
  <img src="public/Screenshot from 2025-10-15 23-11-02.png" alt="Server rendering a custom page" width="800">
  <p><i>Example of the server rendering a custom page</i></p>
</div>

## ✨ Features

- **Multi-port support** - Run multiple server instances on different ports
- **Static file serving** - Serve HTML, CSS, JavaScript, and other static assets
- **File upload handling** - Support for multipart form data uploads
- **CGI support** - Execute Python, PHP, and other CGI scripts
- **Session management** - Built-in session handling with UUID generation
- **File deletion** - HTTP DELETE method support
- **JSON configuration** - Easy server configuration via `config.json`
- **Custom routing** - Define routes with specific HTTP methods and default files
- **Error pages** - Customizable error pages for different HTTP status codes
- **Request size limits** - Configurable client body size limits

## 📂 Project Structure

```
localhost/
├── Cargo.toml          # Project dependencies and metadata
├── config.json         # Server configuration file
├── public/             # Static files directory
│   ├── index.html
│   ├── style.css
│   ├── errors/         # Custom error pages
│   └── cgi-bin/        # CGI scripts
└── src/
    ├── main.rs         # Application entry point
    ├── lib.rs          # Library module exports
    ├── config.rs       # Configuration loading and parsing
    ├── event_loop.rs   # Main server event loop
    ├── http_request.rs # HTTP request parsing
    ├── http_response.rs# HTTP response generation
    ├── file_upload.rs  # File upload handling
    ├── cgi.rs          # CGI execution logic
    ├── cgi_handler.rs  # CGI request handling
    ├── session.rs      # Session management
    ├── delete_file.rs  # File deletion handler
    └── request_queue.rs# Request queue management
```

## 🚀 Getting Started

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd localhost
```

2. Build the project:
```bash
cargo build --release
```

### Configuration

Edit the `config.json` file to configure your server:

```json
{
    "servers": [
        {
            "name": "server1",
            "client_body_size_limit": 1048576,
            "addr": "127.0.0.1",
            "ports": ["8080", "8081"],
            "error_pages": {
                "404": "errors/404.html",
                "500": "errors/500.html"
            },
            "routes": {
                "/": {
                    "accepted_methods": ["GET", "POST"],
                    "default_file": "index.html"
                },
                "/upload": {
                    "accepted_methods": ["POST"],
                    "upload_path": "uploads/"
                }
            }
        }
    ]
}
```

### Running the Server

```bash
# Development mode
cargo run

# Production mode (optimized)
cargo run --release
```

The server will start on the configured ports (default: 8080, 8081).

## 💡 Usage Examples

### Serving Static Files

Place your static files in the `public/` directory. The server will automatically serve them based on your routing configuration.

```bash
# Access the default page
curl http://localhost:8080/

# Access specific files
curl http://localhost:8080/style.css
```

### File Upload

```bash
curl -X POST -F "file=@myfile.txt" http://localhost:8080/upload
```

### CGI Scripts

Place CGI scripts in `public/cgi-bin/` and make them executable:

```bash
# Python CGI example
curl http://localhost:8080/cgi-bin/script.py

# PHP CGI example
curl http://localhost:8080/cgi-bin/template.php
```

### File Deletion

```bash
curl -X DELETE http://localhost:8080/path/to/file.txt
```

## 📦 Dependencies

- **serde & serde_json** - JSON serialization/deserialization for configuration
- **multipart** - Multipart form data parsing for file uploads
- **urlencoding** - URL encoding/decoding utilities
- **uuid** - UUID generation for session management
- **lazy_static** - Static variable initialization
- **libc** - Low-level system calls

## 🛠️ Development

### Building

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release
```

### Testing

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

## ⚙️ Configuration Options

| Option | Type | Description |
|--------|------|-------------|
| `name` | string | Server instance name |
| `addr` | string | IP address to bind to |
| `ports` | array | List of ports to listen on |
| `client_body_size_limit` | number | Maximum request body size in bytes |
| `error_pages` | object | Custom error page paths |
| `routes` | object | Route configuration with methods and handlers |

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

<div align="center">

**⭐ Star this repository if you found it helpful! ⭐**

Made with ❤️ from 🇸🇳

</div>