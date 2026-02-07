use anyhow::{Context, Result};
use next_rs_server::{DevServer, ServerConfig};
use std::path::PathBuf;

pub async fn run_dev_server(port: u16) -> Result<()> {
    let app_dir = find_app_dir()?;

    println!("Scanning routes in {:?}...", app_dir);

    let config = ServerConfig::new(&app_dir, port);
    let server = DevServer::new(config);

    let routes = server.router().routes.clone();
    println!("\nFound {} routes:", routes.len());
    for route in &routes {
        let route_type = if route.is_api() {
            "API"
        } else if route.is_dynamic() {
            "Dynamic"
        } else {
            "Static"
        };
        println!("  {} [{}]", route.path, route_type);
    }

    println!(
        "\n✓ Development server starting at http://127.0.0.1:{}",
        port
    );
    println!("  Press Ctrl+C to stop\n");

    server.run().await
}

fn find_app_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;

    let candidates = [cwd.join("src/app"), cwd.join("app")];

    for candidate in candidates {
        if candidate.exists() && candidate.is_dir() {
            return Ok(candidate);
        }
    }

    anyhow::bail!("No app directory found. Expected 'src/app' or 'app' in current directory.")
}
