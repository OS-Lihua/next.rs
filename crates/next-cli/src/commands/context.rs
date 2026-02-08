use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

pub fn generate_context() -> Result<()> {
    let app_dir = find_app_dir()?;
    let pkg_name = get_package_name().unwrap_or_else(|| "app".to_string());
    let version = get_package_version().unwrap_or_else(|| "0.1.0".to_string());

    let mut routes = Vec::new();
    scan_routes(&app_dir, "", &mut routes)?;

    let routes_json: Vec<String> = routes
        .iter()
        .map(|(path, file, kind)| {
            format!(
                r#"    {{"path": "{}", "file": "{}", "type": "{}"}}"#,
                path, file, kind
            )
        })
        .collect();

    let conventions = r#"    "page": "pub fn page() -> impl IntoNode",
    "layout": "pub fn layout(children: Node) -> impl IntoNode""#;

    let json = format!(
        r#"{{
  "framework": "next.rs",
  "version": "{}",
  "package": "{}",
  "routes": [
{}
  ],
  "conventions": {{
{}
  }}
}}"#,
        version,
        pkg_name,
        routes_json.join(",\n"),
        conventions,
    );

    fs::write(".next-context.json", &json).context("Failed to write .next-context.json")?;
    println!("✓ Generated .next-context.json ({} routes)", routes.len());
    Ok(())
}

fn scan_routes(
    dir: &std::path::Path,
    prefix: &str,
    routes: &mut Vec<(String, String, String)>,
) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(dir).context("Failed to read app directory")?;
    let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_file() && name.ends_with(".rs") {
            let route_path = if prefix.is_empty() {
                "/".to_string()
            } else {
                prefix.to_string()
            };
            let file_path = path
                .strip_prefix(std::env::current_dir().unwrap_or_default())
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();

            match name.as_str() {
                "page.rs" => routes.push((route_path, file_path, "page".to_string())),
                "layout.rs" => routes.push((route_path, file_path, "layout".to_string())),
                "route.rs" => routes.push((route_path, file_path, "api".to_string())),
                _ => {}
            }
        } else if path.is_dir() && name != "." && name != ".." {
            let child_prefix = format!("{}/{}", prefix, name);
            scan_routes(&path, &child_prefix, routes)?;
        }
    }

    Ok(())
}

fn find_app_dir() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let candidates = [cwd.join("src/app"), cwd.join("app")];
    for candidate in candidates {
        if candidate.exists() && candidate.is_dir() {
            return Ok(candidate);
        }
    }
    anyhow::bail!("No app directory found (expected src/app/ or app/)")
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

fn get_package_version() -> Option<String> {
    let content = fs::read_to_string("Cargo.toml").ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version") {
            if let Some(ver) = trimmed.split('=').nth(1) {
                return Some(ver.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}
