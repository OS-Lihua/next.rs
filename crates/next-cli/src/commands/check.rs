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

                        diagnostics.push(serde_json::json!({
                            "level": level,
                            "message": text,
                            "file": file,
                            "line": line_num,
                            "column": column,
                        }));
                    }
                }
            }
        }
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&diagnostics).unwrap_or_else(|_| "[]".to_string())
    );

    Ok(())
}
