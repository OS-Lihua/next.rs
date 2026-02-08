use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub async fn add_page(path: &str, interactive: bool) -> Result<()> {
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
        .rsplit('/')
        .next()
        .unwrap_or("home")
        .replace(['[', ']'], "");
    let page_name = if page_name.is_empty() {
        "Home"
    } else {
        &page_name
    };

    let content = if interactive {
        format!(
            r#"use react_rs_core::create_signal;
use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;
use react_rs_elements::SignalExt;

pub fn page() -> impl IntoNode {{
    let (count, set_count) = create_signal(0);

    div()
        .child(h1().text("{title}"))
        .child(
            button()
                .class("px-4 py-2 bg-blue-500 text-white rounded")
                .text_reactive(count.map(|n| format!("Count: {{}}", n)))
                .on_click(move |_| {{ set_count.update(|n| *n += 1); }})
        )
}}
"#,
            title = capitalize(page_name),
        )
    } else {
        format!(
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
        )
    };

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

pub async fn add_component(name: &str, interactive: bool) -> Result<()> {
    let file_path = PathBuf::from(format!("src/components/{}.rs", name));

    if file_path.exists() {
        anyhow::bail!("File already exists: {:?}", file_path);
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).context("Failed to create directories")?;
    }

    let content = if interactive {
        format!(
            r#"use react_rs_core::create_signal;
use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;
use react_rs_elements::SignalExt;

pub fn {name}() -> impl IntoNode {{
    let (value, set_value) = create_signal(String::new());

    div()
        .class("{name}")
        .child(
            input()
                .type_("text")
                .placeholder("Type here...")
                .bind_value(value.clone(), set_value)
        )
        .child(p().text_reactive(value.map(|v| format!("Value: {{}}", v))))
}}
"#,
            name = name,
        )
    } else {
        format!(
            r#"use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;

pub fn {name}() -> impl IntoNode {{
    div()
        .class("{name}")
        .child(p().text("{name} component"))
}}
"#,
            name = name,
        )
    };

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
