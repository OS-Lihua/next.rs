use std::collections::HashMap;
use std::sync::Arc;

use react_rs_dom::render_to_string;
use react_rs_elements::html::*;
use react_rs_elements::node::{IntoNode, Node};

pub type PageRenderFn = Arc<dyn Fn(&HashMap<String, String>) -> Node + Send + Sync>;
pub type LayoutRenderFn = Arc<dyn Fn(Node) -> Node + Send + Sync>;

pub struct PageRegistry {
    pages: HashMap<String, PageRenderFn>,
    layouts: HashMap<String, LayoutRenderFn>,
}

impl PageRegistry {
    pub fn new() -> Self {
        Self {
            pages: HashMap::new(),
            layouts: HashMap::new(),
        }
    }

    pub fn register_page<F>(&mut self, route: &str, render_fn: F)
    where
        F: Fn(&HashMap<String, String>) -> Node + Send + Sync + 'static,
    {
        self.pages.insert(route.to_string(), Arc::new(render_fn));
    }

    pub fn register_layout<F>(&mut self, route: &str, render_fn: F)
    where
        F: Fn(Node) -> Node + Send + Sync + 'static,
    {
        self.layouts.insert(route.to_string(), Arc::new(render_fn));
    }

    pub fn get_page(&self, route: &str) -> Option<&PageRenderFn> {
        self.pages.get(route)
    }

    pub fn get_layout(&self, route: &str) -> Option<&LayoutRenderFn> {
        self.layouts.get(route)
    }

    pub fn has_page(&self, route: &str) -> bool {
        self.pages.contains_key(route)
    }
}

impl Default for PageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SsrRenderer;

impl SsrRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn render(
        &self,
        route_path: &str,
        params: &HashMap<String, String>,
        registry: &PageRegistry,
    ) -> String {
        let page_node = if let Some(page_fn) = registry.get_page(route_path) {
            page_fn(params)
        } else {
            div()
                .child(h1().text(format!("Route: {}", route_path)))
                .child(p().text("No page component registered for this route."))
                .into_node()
        };

        let mut content = page_node;

        if let Some(root_layout_fn) = registry.get_layout("/") {
            content = root_layout_fn(content);
        }

        let body_html = render_to_string(&content).html;
        let params_json = serde_json::to_string(params).unwrap_or_else(|_| "{}".to_string());

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>next.rs | {route}</title>
    <script>window.__NEXT_DATA__ = {{ route: "{route}", params: {params} }};</script>
</head>
<body>
    <div id="__next">{body}</div>
    <script type="module">
        console.log('next.rs hydration ready');
    </script>
</body>
</html>"#,
            route = route_path,
            params = params_json,
            body = body_html,
        )
    }

    pub fn render_not_found(&self) -> String {
        let content = div()
            .child(h1().text("404 - Page Not Found"))
            .child(p().text("The page you're looking for doesn't exist."));
        let body_html = render_to_string(&content.into_node()).html;

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>404 | next.rs</title>
</head>
<body>
    <div id="__next">{}</div>
</body>
</html>"#,
            body_html
        )
    }

    pub fn render_error(&self, error: &str) -> String {
        let content = div()
            .child(h1().text("Something went wrong"))
            .child(pre().text(error));
        let body_html = render_to_string(&content.into_node()).html;

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Error | next.rs</title>
</head>
<body>
    <div id="__next">{}</div>
</body>
</html>"#,
            body_html
        )
    }
}

impl Default for SsrRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry() -> PageRegistry {
        let mut registry = PageRegistry::new();
        registry.register_page("/", |_params| {
            div()
                .class("home")
                .child(h1().text("Welcome to next.rs"))
                .child(p().text("Home page content"))
                .into_node()
        });
        registry.register_page("/about", |_params| {
            div()
                .child(h1().text("About"))
                .child(p().text("About page content"))
                .into_node()
        });
        registry.register_page("/blog/[slug]", |params| {
            let slug = params.get("slug").map(|s| s.as_str()).unwrap_or("unknown");
            div()
                .child(h1().text(format!("Blog: {}", slug)))
                .into_node()
        });
        registry.register_layout("/", |children| {
            div()
                .class("layout")
                .child(nav().child(a().href("/").text("Home")))
                .child(main_el().child(children))
                .into_node()
        });
        registry
    }

    #[test]
    fn test_render_basic_page() {
        let registry = test_registry();
        let renderer = SsrRenderer::new();

        let html = renderer.render("/", &HashMap::new(), &registry);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("__next"));
        assert!(html.contains("Welcome to next.rs"));
        assert!(html.contains("Home page content"));
    }

    #[test]
    fn test_render_with_layout() {
        let registry = test_registry();
        let renderer = SsrRenderer::new();

        let html = renderer.render("/", &HashMap::new(), &registry);

        assert!(html.contains("class=\"layout\""));
        assert!(html.contains("<nav>"));
        assert!(html.contains("Welcome to next.rs"));
    }

    #[test]
    fn test_render_with_params() {
        let registry = test_registry();
        let renderer = SsrRenderer::new();

        let mut params = HashMap::new();
        params.insert("slug".to_string(), "hello-world".to_string());

        let html = renderer.render("/blog/[slug]", &params, &registry);

        assert!(html.contains("Blog: hello-world"));
        assert!(html.contains("hello-world"));
    }

    #[test]
    fn test_render_not_found() {
        let renderer = SsrRenderer::new();
        let html = renderer.render_not_found();

        assert!(html.contains("404"));
        assert!(html.contains("Page Not Found"));
    }

    #[test]
    fn test_render_error() {
        let renderer = SsrRenderer::new();
        let html = renderer.render_error("Test error message");

        assert!(html.contains("Something went wrong"));
        assert!(html.contains("Test error message"));
    }

    #[test]
    fn test_render_unregistered_route() {
        let registry = PageRegistry::new();
        let renderer = SsrRenderer::new();

        let html = renderer.render("/unknown", &HashMap::new(), &registry);

        assert!(html.contains("No page component registered"));
    }

    #[test]
    fn test_page_registry() {
        let registry = test_registry();

        assert!(registry.has_page("/"));
        assert!(registry.has_page("/about"));
        assert!(!registry.has_page("/nonexistent"));
    }

    #[test]
    fn test_next_data_script() {
        let registry = test_registry();
        let renderer = SsrRenderer::new();

        let mut params = HashMap::new();
        params.insert("slug".to_string(), "test".to_string());

        let html = renderer.render("/blog/[slug]", &params, &registry);

        assert!(html.contains("__NEXT_DATA__"));
        assert!(html.contains("\"slug\":\"test\""));
    }
}
