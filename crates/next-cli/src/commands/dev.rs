use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use next_rs_server::{DevServer, PageRegistry, ServerConfig};
use notify::{Event, RecursiveMode, Watcher};

pub async fn run_dev_server(port: u16) -> Result<()> {
    let app_dir = find_app_dir()?;

    println!("Scanning routes in {:?}...", app_dir);

    if let Err(e) = compile_wasm_dev() {
        eprintln!("⚠ WASM compilation skipped: {}", e);
        eprintln!("  Server will run in SSR-only mode.");
    }

    let config = ServerConfig::new(&app_dir, port);
    let registry = PageRegistry::new();
    let server = DevServer::new(config, registry);
    let reload_tx = server.reload_sender();

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

    compile_tailwind();

    println!(
        "\n✓ Development server starting at http://127.0.0.1:{}",
        port
    );
    println!("  Watching for file changes...");
    println!("  Press Ctrl+C to stop\n");

    let rebuild_flag = Arc::new(AtomicBool::new(false));
    let rebuild_flag_clone = rebuild_flag.clone();

    let watch_dirs = find_watch_dirs()?;

    let mut watcher = notify::recommended_watcher(move |res: std::result::Result<Event, _>| {
        if let Ok(event) = res {
            let is_rs_file = event
                .paths
                .iter()
                .any(|p| p.extension().map(|e| e == "rs").unwrap_or(false));

            if is_rs_file {
                rebuild_flag_clone.store(true, Ordering::SeqCst);
            }
        }
    })
    .context("Failed to create file watcher")?;

    for dir in &watch_dirs {
        if dir.exists() {
            watcher
                .watch(dir, RecursiveMode::Recursive)
                .context(format!("Failed to watch {:?}", dir))?;
        }
    }

    let rebuild_flag_poller = rebuild_flag.clone();
    tokio::spawn(async move {
        let mut debounce_timer: Option<tokio::time::Instant> = None;

        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;

            if rebuild_flag_poller.load(Ordering::SeqCst) {
                rebuild_flag_poller.store(false, Ordering::SeqCst);
                debounce_timer = Some(tokio::time::Instant::now());
            }

            if let Some(timer) = debounce_timer {
                if timer.elapsed() >= Duration::from_millis(200) {
                    debounce_timer = None;
                    println!("\n📦 File changed, rebuilding...");

                    let status = Command::new("cargo").args(["build"]).status();

                    match status {
                        Ok(s) if s.success() => {
                            compile_tailwind();
                            let _ = compile_wasm_dev();
                            let _ = reload_tx.send("reload".to_string());
                            println!("✓ Build successful. Browser will reload.\n");
                        }
                        Ok(_) => {
                            println!("✗ Build failed. Fix errors and save again.\n");
                        }
                        Err(e) => {
                            println!("✗ Failed to run cargo: {}\n", e);
                        }
                    }
                }
            }
        }
    });

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

fn compile_wasm_dev() -> Result<()> {
    let has_wasm_target = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("wasm32-unknown-unknown"))
        .unwrap_or(false);

    if !has_wasm_target {
        anyhow::bail!("wasm32-unknown-unknown target not installed");
    }

    let has_wasm_bindgen = Command::new("wasm-bindgen")
        .arg("--version")
        .output()
        .is_ok();

    if !has_wasm_bindgen {
        anyhow::bail!("wasm-bindgen-cli not found");
    }

    println!("Compiling WASM (dev mode)...");

    let status = Command::new("cargo")
        .args(["build", "--target", "wasm32-unknown-unknown", "--lib"])
        .status()
        .context("Failed to run WASM build")?;

    if !status.success() {
        anyhow::bail!("WASM build failed");
    }

    let pkg_dir = PathBuf::from("pkg");
    std::fs::create_dir_all(&pkg_dir).context("Failed to create pkg directory")?;

    let pkg_name = get_package_name().unwrap_or_else(|| "app".to_string());
    let wasm_file = PathBuf::from(format!(
        "target/wasm32-unknown-unknown/debug/{}.wasm",
        pkg_name.replace('-', "_")
    ));

    if wasm_file.exists() {
        let status = Command::new("wasm-bindgen")
            .args([
                wasm_file.to_str().unwrap(),
                "--out-dir",
                pkg_dir.to_str().unwrap(),
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

    println!("  WASM compiled successfully");
    Ok(())
}

fn get_package_name() -> Option<String> {
    let content = std::fs::read_to_string("Cargo.toml").ok()?;
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

fn compile_tailwind() {
    let input = std::path::Path::new("input.css");
    if !input.exists() {
        return;
    }

    let _ = std::fs::create_dir_all("public");

    let result = Command::new("npx")
        .args(["tailwindcss", "-i", "input.css", "-o", "public/styles.css"])
        .output();

    match result {
        Ok(output) if output.status.success() => {
            println!("  ✓ Tailwind CSS compiled");
        }
        _ => {
            let result2 = Command::new("tailwindcss")
                .args(["-i", "input.css", "-o", "public/styles.css"])
                .output();
            match result2 {
                Ok(output) if output.status.success() => {
                    println!("  ✓ Tailwind CSS compiled");
                }
                _ => {}
            }
        }
    }
}

fn find_watch_dirs() -> Result<Vec<PathBuf>> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let mut dirs = Vec::new();

    for candidate in ["src", "app"] {
        let dir = cwd.join(candidate);
        if dir.exists() {
            dirs.push(dir);
        }
    }

    if dirs.is_empty() {
        dirs.push(cwd);
    }

    Ok(dirs)
}
