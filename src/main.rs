use localhost::config::load_config;

fn main() {
    let config = load_config("config.json").expect("Failed to load configuration");
    println!("config: {:?}", config);
    if let Err(e) = config.start() {
        eprintln!("Error: {}", e);
    }
}