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

    let config = ServerConfig::new(&app_dir, port);
    let registry = PageRegistry::new();
    let server = DevServer::new(config, registry);

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
                            println!("✓ Build successful. Reload the browser to see changes.\n");
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
