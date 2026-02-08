use anyhow::{Context, Result};
use std::process::Command;

pub async fn run_check(json: bool) -> Result<()> {
    if json {
        run_check_json().await
    } else {
        run_check_pretty().await
    }
}

async fn run_check_pretty() -> Result<()> {
    println!("Checking project...\n");

    let status = Command::new("cargo")
        .args(["check", "--message-format=short"])
        .status()
        .context("Failed to run cargo check")?;

    if status.success() {
        println!("\n✓ No errors found");
    }

    Ok(())
}

async fn run_check_json() -> Result<()> {
    let output = Command::new("cargo")
        .args(["check", "--message-format=json"])
        .output()
        .context("Failed to run cargo check")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut diagnostics = Vec::new();

    for line in stdout.lines() {
        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(line) {
            if msg.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                if let Some(message) = msg.get("message") {
                    let level = message
                        .get("level")
                        .and_then(|l| l.as_str())
                        .unwrap_or("unknown");

                    if level == "error" || level == "warning" {
                        let text = message
                            .get("message")
                            .and_then(|m| m.as_str())
                            .unwrap_or("");

                        let mut file = String::new();
                        let mut line_num = 0u64;
                        let mut column = 0u64;

                        if let Some(spans) = message.get("spans").and_then(|s| s.as_array()) {
                            if let Some(span) = spans.first() {
                                file = span
                                    .get("file_name")
                                    .and_then(|f| f.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                line_num =
                                    span.get("line_start").and_then(|l| l.as_u64()).unwrap_or(0);
                                column = span
                                    .get("column_start")
                                    .and_then(|c| c.as_u64())
                                    .unwrap_or(0);
                            }
                        }

                        let fix = suggest_fix(level, text, &file);

                        let mut diag = serde_json::json!({
                            "level": level,
                            "message": text,
                            "file": file,
                            "line": line_num,
                            "column": column,
                        });

                        if let Some(fix_obj) = fix {
                            diag.as_object_mut()
                                .unwrap()
                                .insert("fix".to_string(), fix_obj);
                        }

                        diagnostics.push(diag);
                    }
                }
            }
        }
    }

    let errors = diagnostics
        .iter()
        .filter(|d| d.get("level").and_then(|l| l.as_str()) == Some("error"))
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.get("level").and_then(|l| l.as_str()) == Some("warning"))
        .count();

    let result = serde_json::json!({
        "status": if errors == 0 { "ok" } else { "error" },
        "diagnostics": diagnostics,
        "summary": {
            "errors": errors,
            "warnings": warnings,
        }
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string())
    );

    Ok(())
}

fn suggest_fix(_level: &str, message: &str, file: &str) -> Option<serde_json::Value> {
    if message.contains("cannot find") && message.contains("page") && file.contains("page.rs") {
        return Some(serde_json::json!({
            "description": "Add a public page function",
            "suggestion": "pub fn page() -> impl IntoNode { div().text(\"Page content\") }"
        }));
    }
    if message.contains("cannot find") && message.contains("layout") && file.contains("layout.rs") {
        return Some(serde_json::json!({
            "description": "Add a public layout function",
            "suggestion": "pub fn layout(children: Node) -> impl IntoNode { div().child(children) }"
        }));
    }
    if message.contains("unresolved import") && message.contains("react_rs") {
        return Some(serde_json::json!({
            "description": "Add missing dependency to Cargo.toml",
            "suggestion": "Add `react-rs-core = \"0.2\"` or `react-rs-elements = \"0.2\"` to [dependencies]"
        }));
    }
    if message.contains("IntoNode") && message.contains("not in scope") {
        return Some(serde_json::json!({
            "description": "Import IntoNode trait",
            "suggestion": "use react_rs_elements::node::IntoNode;"
        }));
    }
    if message.contains("SignalExt") && message.contains("not in scope") {
        return Some(serde_json::json!({
            "description": "Import SignalExt trait for .map()",
            "suggestion": "use react_rs_elements::SignalExt;"
        }));
    }
    None
}
