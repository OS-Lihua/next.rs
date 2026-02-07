use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub async fn create_project(name: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    println!("Creating next.rs project: {}", name);

    fs::create_dir_all(project_dir.join("src/app"))
        .context("Failed to create project directories")?;

    create_cargo_toml(project_dir, name)?;
    create_main_rs(project_dir)?;
    create_root_layout(project_dir)?;
    create_home_page(project_dir)?;

    println!("\n✓ Project created successfully!");
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  cargo run -- dev");

    Ok(())
}

fn create_cargo_toml(project_dir: &Path, name: &str) -> Result<()> {
    let content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
next-rs-router = "0.1"
next-rs-server = "0.1"
react-rs-core = "0.1"
react-rs-elements = "0.1"
react-rs-dom = "0.1"
tokio = {{ version = "1", features = ["full"] }}
"#,
        name
    );
    fs::write(project_dir.join("Cargo.toml"), content).context("Failed to write Cargo.toml")?;
    Ok(())
}

fn create_main_rs(project_dir: &Path) -> Result<()> {
    let content = r#"use next_rs_server::DevServer;

#[tokio::main]
async fn main() {
    let server = DevServer::new(3000);
    println!("Starting dev server at http://{}", server.addr());
}
"#;
    fs::write(project_dir.join("src/main.rs"), content).context("Failed to write main.rs")?;
    Ok(())
}

fn create_root_layout(project_dir: &Path) -> Result<()> {
    let content = r#"use react_rs_elements::*;

pub fn layout(children: impl IntoNode) -> impl IntoNode {
    html()
        .child(
            head()
                .child(meta().attr("charset", "utf-8"))
                .child(meta().attr("name", "viewport").attr("content", "width=device-width, initial-scale=1"))
                .child(title().text("next.rs App"))
        )
        .child(
            body().child(children)
        )
}
"#;
    fs::write(project_dir.join("src/app/layout.rs"), content)
        .context("Failed to write layout.rs")?;
    Ok(())
}

fn create_home_page(project_dir: &Path) -> Result<()> {
    let content = r#"use react_rs_elements::*;

pub fn page() -> impl IntoNode {
    main()
        .class("container")
        .child(
            h1().text("Welcome to next.rs")
        )
        .child(
            p().text("Edit src/app/page.rs to get started.")
        )
}
"#;
    fs::write(project_dir.join("src/app/page.rs"), content).context("Failed to write page.rs")?;
    Ok(())
}
