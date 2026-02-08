use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub async fn create_project(name: &str, template: &str) -> Result<()> {
    let project_dir = Path::new(name);

    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    println!(
        "Creating next.rs project: {} (template: {})",
        name, template
    );

    fs::create_dir_all(project_dir.join("src/app"))
        .context("Failed to create project directories")?;

    fs::create_dir_all(project_dir.join("public")).context("Failed to create public directory")?;

    create_cargo_toml(project_dir, name)?;
    create_build_rs(project_dir)?;
    create_lib_rs(project_dir)?;
    create_main_rs(project_dir, name)?;
    create_root_layout(project_dir)?;
    create_gitignore(project_dir)?;
    create_tailwind_config(project_dir)?;
    create_input_css(project_dir)?;

    match template {
        "blog" => create_blog_template(project_dir)?,
        "dashboard" => create_dashboard_template(project_dir)?,
        _ => create_home_page(project_dir)?,
    }

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

[lib]
crate-type = ["cdylib", "rlib"]

[[bin]]
name = "server"
path = "src/main.rs"

[dependencies]
next-rs-server = "0.2"
next-rs-router = "0.2"
react-rs-core = "0.2"
react-rs-elements = "0.2"
react-rs-dom = "0.2"
tokio = {{ version = "1", features = ["full"] }}
anyhow = "1"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
react-rs-wasm = "0.2"
web-sys = {{ version = "0.3", features = ["console", "Window", "Location"] }}

[build-dependencies]
next-rs-router = "0.2"
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

fn create_lib_rs(project_dir: &Path) -> Result<()> {
    let content = r#"pub mod app;

use react_rs_elements::node::{IntoNode, Node};

pub fn render_app(route: &str) -> Node {
    match route {
        "/" => app::page::page().into_node(),
        _ => app::page::page().into_node(),
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    
    #[wasm_bindgen(start)]
    pub fn hydrate() {
        let window = web_sys::window().expect("no window");
        let pathname = window.location().pathname().unwrap_or_else(|_| "/".to_string());
        
        react_rs_wasm::setup_link_interception();
        
        let node = super::render_app(&pathname);
        match react_rs_wasm::hydrate(&node, "__next") {
            Ok(_) => web_sys::console::log_1(&"Hydration successful!".into()),
            Err(_) => { let _ = react_rs_wasm::mount(&node, "__next"); }
        }
    }
}
"#;
    fs::write(project_dir.join("src/lib.rs"), content).context("Failed to write lib.rs")?;
    Ok(())
}

fn create_main_rs(project_dir: &Path, name: &str) -> Result<()> {
    let content = format!(
        r#"use next_rs_server::{{DevServer, PageRegistry, ServerConfig}};
use react_rs_elements::node::IntoNode;
use {}::app;

include!(concat!(env!("OUT_DIR"), "/routes_generated.rs"));

fn build_registry() -> PageRegistry {{
    let mut registry = PageRegistry::new();

    for &(route, kind, _file) in ROUTE_TABLE {{
        match kind {{
            "page" => {{
                let route = route.to_string();
                let route_key = route.clone();
                registry.register_page(&route_key, move |_params| {{
                    match route.as_str() {{
                        "/" => app::page::page().into_node(),
                        _ => {{
                            use react_rs_elements::html::*;
                            div().text(format!("Page: {{}}", route)).into_node()
                        }}
                    }}
                }});
            }}
            "layout" => {{
                let route = route.to_string();
                let route_key = route.clone();
                registry.register_layout(&route_key, move |children| {{
                    match route.as_str() {{
                        "/" => app::layout::layout(children).into_node(),
                        _ => children,
                    }}
                }});
            }}
            _ => {{}}
        }}
    }}

    registry
}}

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    let registry = build_registry();

    let config = ServerConfig::new("src/app", 3000);
    let server = DevServer::new(config, registry);

    println!("Starting dev server at http://{{}}", server.addr());
    server.run().await
}}
"#,
        name.replace("-", "_")
    );
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
    let content = r#"use react_rs_core::create_signal;
use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;
use react_rs_elements::SignalExt;

pub fn page() -> impl IntoNode {
    let (count, set_count) = create_signal(0);
    
    div()
        .class("container")
        .child(h1().text("Welcome to next.rs"))
        .child(p().text("Edit src/app/page.rs to get started."))
        .child(
            button()
                .text_reactive(count.map(|n| format!("Count: {}", n)))
                .on_click(move |_| { set_count.update(|n| *n += 1); })
        )
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

fn create_blog_template(project_dir: &Path) -> Result<()> {
    let home = r#"use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;

pub fn page() -> impl IntoNode {
    div()
        .class("container mx-auto p-8")
        .child(h1().class("text-3xl font-bold mb-8").text("My Blog"))
        .child(
            div().class("space-y-6")
                .child(article_card("Getting Started with next.rs", "Learn how to build web apps with Rust.", "/blog/getting-started"))
                .child(article_card("Signals and Reactivity", "Fine-grained reactivity without Virtual DOM.", "/blog/signals"))
                .child(article_card("SSR + WASM Hydration", "Server-render then hydrate in the browser.", "/blog/ssr-hydration"))
        )
}

fn article_card(title: &str, excerpt: &str, href: &str) -> impl IntoNode {
    a().href(href).child(
        div()
            .class("border rounded-lg p-6 hover:shadow-lg transition-shadow")
            .child(h2().class("text-xl font-semibold mb-2").text(title))
            .child(p().class("text-gray-600").text(excerpt))
    )
}
"#;
    fs::write(project_dir.join("src/app/page.rs"), home)?;

    fs::create_dir_all(project_dir.join("src/app/blog"))?;
    let blog_page = r#"use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;

pub fn page() -> impl IntoNode {
    div()
        .class("container mx-auto p-8")
        .child(h1().class("text-2xl font-bold mb-4").text("All Posts"))
        .child(p().text("Browse all blog posts."))
}
"#;
    fs::write(project_dir.join("src/app/blog/page.rs"), blog_page)?;
    let blog_mod = "pub mod page;\n";
    fs::write(project_dir.join("src/app/blog/mod.rs"), blog_mod)?;

    fs::create_dir_all(project_dir.join("src/app/blog/[slug]"))?;
    let slug_page = r#"use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;

pub fn page() -> impl IntoNode {
    div()
        .class("container mx-auto p-8")
        .child(h1().class("text-2xl font-bold mb-4").text("Blog Post"))
        .child(p().text("Dynamic blog post content goes here."))
        .child(a().href("/blog").class("text-blue-500 underline mt-4").text("← Back to blog"))
}
"#;
    fs::write(project_dir.join("src/app/blog/[slug]/page.rs"), slug_page)?;
    let slug_mod = "pub mod page;\n";
    fs::write(project_dir.join("src/app/blog/[slug]/mod.rs"), slug_mod)?;

    let app_mod = "pub mod layout;\npub mod page;\npub mod blog;\n";
    fs::write(project_dir.join("src/app/mod.rs"), app_mod)?;

    Ok(())
}

fn create_dashboard_template(project_dir: &Path) -> Result<()> {
    let home = r#"use react_rs_core::create_signal;
use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;
use react_rs_elements::SignalExt;

pub fn page() -> impl IntoNode {
    let (active_tab, set_active_tab) = create_signal("overview".to_string());

    div()
        .class("flex min-h-screen")
        .child(sidebar())
        .child(
            div().class("flex-1 p-8")
                .child(h1().class("text-2xl font-bold mb-6").text("Dashboard"))
                .child(
                    div().class("grid grid-cols-3 gap-4 mb-8")
                        .child(stat_card("Users", "1,234"))
                        .child(stat_card("Revenue", "$12,345"))
                        .child(stat_card("Orders", "567"))
                )
                .child(
                    div().class("flex gap-4 mb-4")
                        .child(tab_button("Overview", "overview", active_tab.clone(), set_active_tab.clone()))
                        .child(tab_button("Analytics", "analytics", active_tab.clone(), set_active_tab.clone()))
                )
                .child(
                    p().text_reactive(active_tab.map(|t| format!("Selected tab: {}", t)))
                )
        )
}

fn sidebar() -> impl IntoNode {
    nav()
        .class("w-64 bg-gray-800 text-white p-4")
        .child(h2().class("text-xl font-bold mb-6").text("Admin"))
        .child(
            ul().class("space-y-2")
                .child(li().child(a().href("/").class("block p-2 rounded hover:bg-gray-700").text("Dashboard")))
                .child(li().child(a().href("/settings").class("block p-2 rounded hover:bg-gray-700").text("Settings")))
        )
}

fn stat_card(label: &str, value: &str) -> impl IntoNode {
    div()
        .class("bg-white rounded-lg shadow p-6")
        .child(p().class("text-gray-500 text-sm").text(label))
        .child(p().class("text-2xl font-bold").text(value))
}

fn tab_button(
    label: &str,
    id: &str,
    _active: react_rs_core::ReadSignal<String>,
    set_active: react_rs_core::WriteSignal<String>,
) -> impl IntoNode {
    let id_owned = id.to_string();
    button()
        .class("px-4 py-2 rounded bg-blue-500 text-white hover:bg-blue-600")
        .text(label)
        .on_click(move |_| { set_active.set(id_owned.clone()); })
}
"#;
    fs::write(project_dir.join("src/app/page.rs"), home)?;

    fs::create_dir_all(project_dir.join("src/app/settings"))?;
    let settings_page = r#"use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;

pub fn page() -> impl IntoNode {
    div()
        .class("container mx-auto p-8")
        .child(h1().class("text-2xl font-bold mb-4").text("Settings"))
        .child(
            div().class("bg-white rounded-lg shadow p-6")
                .child(p().text("Application settings go here."))
        )
}
"#;
    fs::write(project_dir.join("src/app/settings/page.rs"), settings_page)?;
    let settings_mod = "pub mod page;\n";
    fs::write(project_dir.join("src/app/settings/mod.rs"), settings_mod)?;

    let app_mod = "pub mod layout;\npub mod page;\npub mod settings;\n";
    fs::write(project_dir.join("src/app/mod.rs"), app_mod)?;

    Ok(())
}
