use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub async fn add_page(path: &str) -> Result<()> {
    let clean_path = path.trim_start_matches('/');
    let file_path = if clean_path.is_empty() {
        PathBuf::from("src/app/page.rs")
    } else {
        PathBuf::from(format!("src/app/{}/page.rs", clean_path))
    };

    if file_path.exists() {
        anyhow::bail!("File already exists: {:?}", file_path);
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).context("Failed to create directories")?;
    }

    let page_name = clean_path
        .split('/')
        .last()
        .unwrap_or("home")
        .replace(['[', ']'], "");
    let page_name = if page_name.is_empty() {
        "Home"
    } else {
        &page_name
    };

    let content = format!(
        r#"use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;

pub fn page() -> impl IntoNode {{
    div()
        .child(h1().text("{title}"))
        .child(p().text("Edit {path}"))
}}
"#,
        title = capitalize(page_name),
        path = file_path.display(),
    );

    fs::write(&file_path, content).context("Failed to write page file")?;
    println!("✓ Created {}", file_path.display());
    Ok(())
}

pub async fn add_layout(path: &str) -> Result<()> {
    let clean_path = path.trim_start_matches('/');
    let file_path = if clean_path.is_empty() {
        PathBuf::from("src/app/layout.rs")
    } else {
        PathBuf::from(format!("src/app/{}/layout.rs", clean_path))
    };

    if file_path.exists() {
        anyhow::bail!("File already exists: {:?}", file_path);
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).context("Failed to create directories")?;
    }

    let content = r#"use react_rs_elements::html::*;
use react_rs_elements::node::{IntoNode, Node};

pub fn layout(children: Node) -> impl IntoNode {
    div()
        .child(children)
}
"#;

    fs::write(&file_path, content).context("Failed to write layout file")?;
    println!("✓ Created {}", file_path.display());
    Ok(())
}

pub async fn add_component(name: &str) -> Result<()> {
    let file_path = PathBuf::from(format!("src/components/{}.rs", name));

    if file_path.exists() {
        anyhow::bail!("File already exists: {:?}", file_path);
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).context("Failed to create directories")?;
    }

    let content = format!(
        r#"use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;

pub fn {name}() -> impl IntoNode {{
    div()
        .class("{name}")
        .child(p().text("{name} component"))
}}
"#,
        name = name,
    );

    fs::write(&file_path, content).context("Failed to write component file")?;
    println!("✓ Created {}", file_path.display());
    Ok(())
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
