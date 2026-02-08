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

    fs::create_dir_all(project_dir.join("public")).context("Failed to create public directory")?;

    create_cargo_toml(project_dir, name)?;
    create_build_rs(project_dir)?;
    create_main_rs(project_dir)?;
    create_root_layout(project_dir)?;
    create_home_page(project_dir)?;
    create_gitignore(project_dir)?;
    create_tailwind_config(project_dir)?;
    create_input_css(project_dir)?;

    println!("\n✓ Project created successfully!");
    println!("\nNext steps:");
    println!("  cd {}", name);
    println!("  next dev");

    Ok(())
}

fn create_cargo_toml(project_dir: &Path, name: &str) -> Result<()> {
    let content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
next-rs-server = "0.1"
next-rs-router = "0.1"
react-rs-core = "0.1"
react-rs-elements = "0.1"
react-rs-dom = "0.1"
tokio = {{ version = "1", features = ["full"] }}
anyhow = "1"

[build-dependencies]
next-rs-router = "0.1"
"#,
        name
    );
    fs::write(project_dir.join("Cargo.toml"), content).context("Failed to write Cargo.toml")?;
    Ok(())
}

fn create_build_rs(project_dir: &Path) -> Result<()> {
    let content = r#"use next_rs_router::RouteCodegen;
use std::fs;
use std::path::Path;

fn main() {
    let app_dir = Path::new("src/app");
    if !app_dir.exists() {
        return;
    }

    let codegen = RouteCodegen::new(app_dir);
    let code = codegen.generate_simple();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("routes_generated.rs");
    fs::write(&dest, code).expect("Failed to write generated routes");

    println!("cargo::rerun-if-changed=src/app");
}
"#;
    fs::write(project_dir.join("build.rs"), content).context("Failed to write build.rs")?;
    Ok(())
}

fn create_main_rs(project_dir: &Path) -> Result<()> {
    let content = r#"mod app;

use next_rs_server::{DevServer, PageRegistry, ServerConfig};
use react_rs_elements::node::IntoNode;

include!(concat!(env!("OUT_DIR"), "/routes_generated.rs"));

fn build_registry() -> PageRegistry {
    let mut registry = PageRegistry::new();

    for &(route, kind, _file) in ROUTE_TABLE {
        match kind {
            "page" => {
                let route = route.to_string();
                registry.register_page(&route, move |_params| {
                    match route.as_str() {
                        "/" => app::page::page().into_node(),
                        _ => {
                            use react_rs_elements::html::*;
                            div().text(format!("Page: {}", route)).into_node()
                        }
                    }
                });
            }
            "layout" => {
                let route = route.to_string();
                registry.register_layout(&route, move |children| {
                    match route.as_str() {
                        "/" => app::layout::layout(children).into_node(),
                        _ => children,
                    }
                });
            }
            _ => {}
        }
    }

    registry
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let registry = build_registry();

    let config = ServerConfig::new("src/app", 3000);
    let server = DevServer::new(config, registry);

    println!("Starting dev server at http://{}", server.addr());
    server.run().await
}
"#;
    fs::write(project_dir.join("src/main.rs"), content).context("Failed to write main.rs")?;

    fs::create_dir_all(project_dir.join("src/app")).context("Failed to create app dir")?;
    let mod_content = "pub mod layout;\npub mod page;\n";
    fs::write(project_dir.join("src/app/mod.rs"), mod_content)
        .context("Failed to write app/mod.rs")?;

    Ok(())
}

fn create_root_layout(project_dir: &Path) -> Result<()> {
    let content = r#"use react_rs_elements::html::*;
use react_rs_elements::node::{IntoNode, Node};

pub fn layout(children: Node) -> impl IntoNode {
    div()
        .class("app")
        .child(
            header()
                .child(nav().child(a().href("/").text("next.rs")))
        )
        .child(main_el().child(children))
        .child(
            footer().text("Built with next.rs")
        )
}
"#;
    fs::write(project_dir.join("src/app/layout.rs"), content)
        .context("Failed to write layout.rs")?;
    Ok(())
}

fn create_home_page(project_dir: &Path) -> Result<()> {
    let content = r#"use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;

pub fn page() -> impl IntoNode {
    div()
        .class("container")
        .child(h1().text("Welcome to next.rs"))
        .child(p().text("Edit src/app/page.rs to get started."))
        .child(
            p().text("Pure Rust API. No macros. AI-friendly.")
        )
}
"#;
    fs::write(project_dir.join("src/app/page.rs"), content).context("Failed to write page.rs")?;
    Ok(())
}

fn create_gitignore(project_dir: &Path) -> Result<()> {
    let content = "/target\n/.next\n/public/styles.css\n";
    fs::write(project_dir.join(".gitignore"), content).context("Failed to write .gitignore")?;
    Ok(())
}

fn create_tailwind_config(project_dir: &Path) -> Result<()> {
    let content = r#"/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/**/*.rs"],
  theme: {
    extend: {},
  },
  plugins: [],
}
"#;
    fs::write(project_dir.join("tailwind.config.js"), content)
        .context("Failed to write tailwind.config.js")?;
    Ok(())
}

fn create_input_css(project_dir: &Path) -> Result<()> {
    let content = "@tailwind base;\n@tailwind components;\n@tailwind utilities;\n";
    fs::write(project_dir.join("input.css"), content).context("Failed to write input.css")?;
    Ok(())
}
