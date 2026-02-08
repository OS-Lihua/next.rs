use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use anyhow::{Context, Result};
use next_rs_router::{RouteScanner, Router};
use next_rs_server::{PageRegistry, StaticGenerator};

pub async fn run_build() -> Result<()> {
    let app_dir = find_app_dir()?;
    let out_dir = PathBuf::from(".next");

    println!("Building for production...\n");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).context("Failed to clean output directory")?;
    }
    fs::create_dir_all(&out_dir).context("Failed to create output directory")?;

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

    println!("\nBuilding server binary...");
    build_server_binary().context("Failed to build server binary")?;

    println!("Compiling client WASM...");
    match build_client_wasm(&out_dir) {
        Ok(_) => println!("  ✓ WASM compiled successfully"),
        Err(e) => println!("  ⚠ WASM compilation skipped: {}", e),
    }

    let router = Router::from_routes(routes.clone());
    let registry = Arc::new(PageRegistry::new());
    let generator = StaticGenerator::new(router, app_dir, out_dir.clone(), registry);

    compile_tailwind_production(&out_dir);

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

fn build_server_binary() -> Result<()> {
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("Server build failed");
    }

    println!("  ✓ Server binary built");
    Ok(())
}

fn build_client_wasm(out_dir: &std::path::Path) -> Result<()> {
    let has_wasm_target = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("wasm32-unknown-unknown"))
        .unwrap_or(false);

    if !has_wasm_target {
        anyhow::bail!(
            "wasm32-unknown-unknown target not installed.\n\
             Install with: rustup target add wasm32-unknown-unknown"
        );
    }

    let has_wasm_bindgen = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .is_ok();

    if !has_wasm_bindgen {
        anyhow::bail!(
            "wasm-bindgen-cli not found.\n\
             Install with: cargo install wasm-bindgen-cli"
        );
    }

    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target",
            "wasm32-unknown-unknown",
            "--lib",
        ])
        .status()
        .context("Failed to run WASM build")?;

    if !status.success() {
        anyhow::bail!("WASM build failed");
    }

    let wasm_out = out_dir.join("pkg");
    fs::create_dir_all(&wasm_out).context("Failed to create WASM output directory")?;

    let pkg_name = get_package_name().unwrap_or_else(|| "app".to_string());
    let wasm_file = PathBuf::from(format!(
        "target/wasm32-unknown-unknown/release/{}.wasm",
        pkg_name.replace('-', "_")
    ));

    if wasm_file.exists() {
        let status = Command::new("wasm-bindgen")
            .args([
                wasm_file.to_str().unwrap(),
                "--out-dir",
                wasm_out.to_str().unwrap(),
                "--target",
                "web",
                "--no-typescript",
            ])
            .status()
            .context("Failed to run wasm-bindgen")?;

        if !status.success() {
            anyhow::bail!("wasm-bindgen failed");
        }
    }

    Ok(())
}

fn compile_tailwind_production(out_dir: &std::path::Path) {
    let input = std::path::Path::new("input.css");
    if !input.exists() {
        return;
    }

    let css_dir = out_dir.join("static/css");
    let _ = fs::create_dir_all(&css_dir);
    let output_css = css_dir.join("styles.css");

    let result = Command::new("npx")
        .args([
            "tailwindcss",
            "-i",
            "input.css",
            "-o",
            output_css.to_str().unwrap_or(""),
            "--minify",
        ])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            println!("  ✓ Tailwind CSS compiled (minified)");
        }
        _ => {
            let result2 = Command::new("tailwindcss")
                .args([
                    "-i",
                    "input.css",
                    "-o",
                    output_css.to_str().unwrap_or(""),
                    "--minify",
                ])
                .output();
            match result2 {
                Ok(output) if output.status.success() => {
                    println!("  ✓ Tailwind CSS compiled (minified)");
                }
                _ => {
                    println!("  ⚠ Tailwind CSS not available (install: npm i -D tailwindcss)");
                }
            }
        }
    }
}

fn get_package_name() -> Option<String> {
    let content = fs::read_to_string("Cargo.toml").ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name") {
            if let Some(name) = trimmed.split('=').nth(1) {
                return Some(name.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

pub async fn run_production_server(port: u16) -> Result<()> {
    let out_dir = PathBuf::from(".next");

    if !out_dir.exists() {
        anyhow::bail!("No build found. Run 'next build' first.");
    }

    let manifest_path = out_dir.join("manifest.json");
    if !manifest_path.exists() {
        anyhow::bail!("Build manifest not found. Run 'next build' first.");
    }

    println!("Starting production server...");
    println!("✓ Server running at http://127.0.0.1:{}", port);
    println!("  Serving from .next/");
    println!("  Press Ctrl+C to stop\n");

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let out_dir = Arc::new(out_dir);

    loop {
        let (stream, _) = listener.accept().await?;
        let io = hyper_util::rt::TokioIo::new(stream);
        let out_dir = out_dir.clone();

        tokio::spawn(async move {
            let service = hyper::service::service_fn(move |req| {
                let out_dir = out_dir.clone();
                async move { serve_static_file(&out_dir, req).await }
            });

            if let Err(e) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service)
                .await
            {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}

async fn serve_static_file(
    out_dir: &std::path::Path,
    req: hyper::Request<hyper::body::Incoming>,
) -> std::result::Result<hyper::Response<http_body_util::Full<bytes::Bytes>>, hyper::Error> {
    use bytes::Bytes;
    use http_body_util::Full;
    use hyper::{Response, StatusCode};

    let path = req.uri().path();

    let file_path = if path == "/" {
        out_dir.join("index.html")
    } else {
        let clean = path.trim_start_matches('/');
        let candidate = out_dir.join(format!("{}/index.html", clean));
        if candidate.exists() {
            candidate
        } else {
            out_dir.join(clean)
        }
    };

    if !file_path.starts_with(out_dir) {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Full::new(Bytes::from("Forbidden")))
            .unwrap());
    }

    match fs::read(&file_path) {
        Ok(content) => {
            let content_type = match file_path.extension().and_then(|e| e.to_str()) {
                Some("html") => "text/html; charset=utf-8",
                Some("js") => "application/javascript",
                Some("wasm") => "application/wasm",
                Some("css") => "text/css",
                Some("json") => "application/json",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("svg") => "image/svg+xml",
                Some("ico") => "image/x-icon",
                _ => "application/octet-stream",
            };

            let cache_control = if content_type.starts_with("text/html") {
                "no-cache"
            } else {
                "public, max-age=31536000, immutable"
            };

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", content_type)
                .header("Cache-Control", cache_control)
                .body(Full::new(Bytes::from(content)))
                .unwrap())
        }
        Err(_) => {
            let not_found = out_dir.join("404.html");
            let body =
                fs::read_to_string(&not_found).unwrap_or_else(|_| "404 Not Found".to_string());

            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header("Content-Type", "text/html; charset=utf-8")
                .body(Full::new(Bytes::from(body)))
                .unwrap())
        }
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
