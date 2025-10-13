use localhost::config::load_config;

fn main() {
    let config = load_config("config.json").expect("Failed to load configuration");
    if let Err(e) = config.start() {
        eprintln!("Error: {}", e);  
    }
}