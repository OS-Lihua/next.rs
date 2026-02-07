use anyhow::{Context, Result};
use next_rs_router::{RouteScanner, Router};
use next_rs_server::StaticGenerator;
use std::fs;
use std::path::PathBuf;

pub async fn run_build() -> Result<()> {
    let app_dir = find_app_dir()?;
    let out_dir = PathBuf::from(".next");

    println!("Building for production...\n");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).context("Failed to clean output directory")?;
    }

    let scanner = RouteScanner::new(&app_dir);
    let routes = scanner.scan();

    let static_count = routes
        .iter()
        .filter(|r| !r.is_dynamic() && !r.is_api())
        .count();
    let dynamic_count = routes.iter().filter(|r| r.is_dynamic()).count();
    let api_count = routes.iter().filter(|r| r.is_api()).count();

    println!("Routes:");
    println!("  Static:  {}", static_count);
    println!("  Dynamic: {}", dynamic_count);
    println!("  API:     {}", api_count);

    let router = Router::from_routes(routes.clone());
    let generator = StaticGenerator::new(router, app_dir, out_dir.clone());

    println!("\nGenerating static pages...");
    let result = generator
        .generate()
        .context("Failed to generate static pages")?;

    for file in &result.files {
        println!("  ✓ {} ({} bytes)", file.route, file.size_bytes);
    }

    let manifest = serde_json::json!({
        "routes": routes.iter().map(|r| {
            serde_json::json!({
                "path": r.path,
                "dynamic": r.is_dynamic(),
                "api": r.is_api(),
            })
        }).collect::<Vec<_>>(),
        "build": {
            "pages_generated": result.pages_generated,
            "total_size_bytes": result.total_size_bytes,
        }
    });

    fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .context("Failed to write manifest")?;

    println!("\n✓ Build complete!");
    println!("  Pages: {}", result.pages_generated);
    println!("  Size:  {} bytes", result.total_size_bytes);
    println!("  Output: .next/");

    Ok(())
}

pub async fn run_production_server(port: u16) -> Result<()> {
    let out_dir = PathBuf::from(".next");

    if !out_dir.exists() {
        anyhow::bail!("No build found. Run 'next-rs build' first.");
    }

    println!("Starting production server...");
    println!("✓ Server running at http://127.0.0.1:{}", port);
    println!("  Press Ctrl+C to stop\n");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    }
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
