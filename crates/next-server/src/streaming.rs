use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use futures_core::Stream;

pub struct HtmlStream {
    parts: Vec<String>,
    current_index: usize,
    completed: bool,
}

impl HtmlStream {
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
            current_index: 0,
            completed: false,
        }
    }

    pub fn push(&mut self, html: String) {
        self.parts.push(html);
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }

    pub fn shell(title: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{}</title>
</head>
<body>
    <div id="__next">"#,
            title
        )
    }

    pub fn suspense_placeholder(id: &str) -> String {
        format!(
            r#"<template id="S:{id}"></template>
<div id="P:{id}">Loading...</div>"#,
            id = id
        )
    }

    pub fn suspense_replacement(id: &str, content: &str) -> String {
        format!(
            r#"<script>
(function(){{
    var p = document.getElementById("P:{id}");
    var t = document.getElementById("S:{id}");
    if(p && t) {{
        var d = document.createElement("div");
        d.innerHTML = {content};
        p.replaceWith(d.firstChild);
        t.remove();
    }}
}})();
</script>"#,
            id = id,
            content = serde_json::to_string(content).unwrap_or_else(|_| "\"\"".to_string())
        )
    }

    pub fn closing() -> String {
        r#"    </div>
</body>
</html>"#
            .to_string()
    }
}

impl Default for HtmlStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for HtmlStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.current_index < self.parts.len() {
            let part = self.parts[self.current_index].clone();
            self.current_index += 1;
            Poll::Ready(Some(Ok(Bytes::from(part))))
        } else if self.completed {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

pub struct StreamingRenderer {
    suspense_counter: usize,
}

impl StreamingRenderer {
    pub fn new() -> Self {
        Self {
            suspense_counter: 0,
        }
    }

    pub fn next_suspense_id(&mut self) -> String {
        let id = format!("suspense-{}", self.suspense_counter);
        self.suspense_counter += 1;
        id
    }

    pub fn render_shell(&self, title: &str) -> String {
        HtmlStream::shell(title)
    }

    pub fn render_suspense_placeholder(&mut self) -> (String, String) {
        let id = self.next_suspense_id();
        let placeholder = HtmlStream::suspense_placeholder(&id);
        (id, placeholder)
    }

    pub fn render_suspense_replacement(&self, id: &str, content: &str) -> String {
        HtmlStream::suspense_replacement(id, content)
    }

    pub fn render_closing(&self) -> String {
        HtmlStream::closing()
    }
}

impl Default for StreamingRenderer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RscStream {
    chunks: Vec<String>,
    current_index: usize,
    completed: bool,
}

impl RscStream {
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            current_index: 0,
            completed: false,
        }
    }

    pub fn push_node(&mut self, index: usize, node_json: &str) {
        self.chunks.push(format!("{}:{}\n", index, node_json));
    }

    pub fn push_client_reference(&mut self, id: &str, module: &str, export: &str) {
        self.chunks
            .push(format!("M:{}:{}:{}\n", id, module, export));
    }

    pub fn push_hint(&mut self, hint_type: &str, data: &str) {
        self.chunks.push(format!("H:{}:{}\n", hint_type, data));
    }

    pub fn push_error(&mut self, id: &str, error: &str) {
        let error_json = serde_json::json!({"message": error}).to_string();
        self.chunks.push(format!("E:{}:{}\n", id, error_json));
    }

    pub fn complete(&mut self) {
        self.completed = true;
    }

    pub fn into_chunks(self) -> Vec<String> {
        self.chunks
    }
}

impl Default for RscStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for RscStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.current_index < self.chunks.len() {
            let chunk = self.chunks[self.current_index].clone();
            self.current_index += 1;
            Poll::Ready(Some(Ok(Bytes::from(chunk))))
        } else if self.completed {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    }
}

pub struct RscStreamingRenderer {
    node_counter: usize,
}

impl RscStreamingRenderer {
    pub fn new() -> Self {
        Self { node_counter: 0 }
    }

    pub fn render_node(&mut self, stream: &mut RscStream, node: &next_rs_rsc::RscNode) {
        let node_json = serde_json::to_string(node).unwrap_or_else(|_| "null".to_string());
        stream.push_node(self.node_counter, &node_json);
        self.node_counter += 1;
    }

    pub fn render_payload(&mut self, stream: &mut RscStream, payload: &next_rs_rsc::RscPayload) {
        for node in &payload.nodes {
            self.render_node(stream, node);
        }

        for reference in &payload.client_references {
            stream.push_client_reference(&reference.id, &reference.module, &reference.export);
        }
    }

    pub fn render_suspense_fallback(
        &mut self,
        stream: &mut RscStream,
        id: &str,
        fallback: &next_rs_rsc::RscNode,
    ) {
        let fallback_json = serde_json::to_string(fallback).unwrap_or_else(|_| "null".to_string());
        stream.push_node(
            self.node_counter,
            &format!(
                r#"{{"type":"suspense","id":"{}","fallback":{}}}"#,
                id, fallback_json
            ),
        );
        self.node_counter += 1;
    }

    pub fn render_suspense_content(
        &mut self,
        stream: &mut RscStream,
        id: &str,
        content: &next_rs_rsc::RscNode,
    ) {
        let content_json = serde_json::to_string(content).unwrap_or_else(|_| "null".to_string());
        stream.chunks.push(format!("${}:{}\n", id, content_json));
    }

    pub fn node_counter(&self) -> usize {
        self.node_counter
    }
}

impl Default for RscStreamingRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_stream_shell() {
        let shell = HtmlStream::shell("Test Page");
        assert!(shell.contains("<!DOCTYPE html>"));
        assert!(shell.contains("<title>Test Page</title>"));
        assert!(shell.contains("__next"));
    }

    #[test]
    fn test_suspense_placeholder() {
        let placeholder = HtmlStream::suspense_placeholder("test-1");
        assert!(placeholder.contains("S:test-1"));
        assert!(placeholder.contains("P:test-1"));
        assert!(placeholder.contains("Loading..."));
    }

    #[test]
    fn test_suspense_replacement() {
        let replacement = HtmlStream::suspense_replacement("test-1", "<div>Content</div>");
        assert!(replacement.contains("P:test-1"));
        assert!(replacement.contains("S:test-1"));
        assert!(replacement.contains("<div>Content</div>"));
    }

    #[test]
    fn test_streaming_renderer() {
        let mut renderer = StreamingRenderer::new();

        let (id1, _) = renderer.render_suspense_placeholder();
        let (id2, _) = renderer.render_suspense_placeholder();

        assert_eq!(id1, "suspense-0");
        assert_eq!(id2, "suspense-1");
    }

    #[test]
    fn test_full_streaming_flow() {
        let mut renderer = StreamingRenderer::new();

        let shell = renderer.render_shell("Streaming Test");
        let (id, placeholder) = renderer.render_suspense_placeholder();
        let replacement = renderer.render_suspense_replacement(&id, "<p>Loaded!</p>");
        let closing = renderer.render_closing();

        assert!(shell.contains("<!DOCTYPE html>"));
        assert!(placeholder.contains("Loading..."));
        assert!(replacement.contains("<p>Loaded!</p>"));
        assert!(closing.contains("</html>"));
    }

    #[test]
    fn test_rsc_stream_creation() {
        let mut stream = RscStream::new();
        stream.push_node(0, r#"{"type":"text","value":"Hello"}"#);
        stream.complete();

        let chunks = stream.into_chunks();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].starts_with("0:"));
    }

    #[test]
    fn test_rsc_stream_client_reference() {
        let mut stream = RscStream::new();
        stream.push_client_reference("counter", "./Counter.js", "Counter");
        stream.complete();

        let chunks = stream.into_chunks();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].contains("M:counter:./Counter.js:Counter"));
    }

    #[test]
    fn test_rsc_stream_error() {
        let mut stream = RscStream::new();
        stream.push_error("component-1", "Failed to load");
        stream.complete();

        let chunks = stream.into_chunks();
        assert!(chunks[0].starts_with("E:component-1:"));
        assert!(chunks[0].contains("Failed to load"));
    }

    #[test]
    fn test_rsc_streaming_renderer() {
        let mut renderer = RscStreamingRenderer::new();
        let mut stream = RscStream::new();

        let node = next_rs_rsc::RscNode::text("Hello World");
        renderer.render_node(&mut stream, &node);

        assert_eq!(renderer.node_counter(), 1);
        let chunks = stream.into_chunks();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].contains("Hello World"));
    }

    #[test]
    fn test_rsc_streaming_renderer_payload() {
        let mut renderer = RscStreamingRenderer::new();
        let mut stream = RscStream::new();

        let mut payload = next_rs_rsc::RscPayload::new();
        payload.add_node(next_rs_rsc::RscNode::text("First"));
        payload.add_node(next_rs_rsc::RscNode::text("Second"));
        payload.add_client_reference(
            "btn".to_string(),
            "./Button.js".to_string(),
            "Button".to_string(),
        );

        renderer.render_payload(&mut stream, &payload);
        stream.complete();

        let chunks = stream.into_chunks();
        assert_eq!(chunks.len(), 3);
        assert!(chunks[0].contains("First"));
        assert!(chunks[1].contains("Second"));
        assert!(chunks[2].contains("M:btn"));
    }
}
