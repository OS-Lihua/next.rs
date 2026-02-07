use std::collections::HashMap;
use std::path::PathBuf;

use next_rs_router::LayoutTree;

pub struct SsrRenderer {
    app_dir: PathBuf,
}

impl SsrRenderer {
    pub fn new(app_dir: PathBuf) -> Self {
        Self { app_dir }
    }

    pub fn render(&self, layout_tree: &LayoutTree, params: &HashMap<String, String>) -> String {
        let page_path = &layout_tree.page;
        let layouts = &layout_tree.layouts;

        let page_name = page_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("page");

        let route_path = page_path
            .parent()
            .and_then(|p| p.strip_prefix(&self.app_dir).ok())
            .map(|p| format!("/{}", p.display()))
            .unwrap_or_else(|| "/".to_string())
            .replace("\\", "/");

        let params_json = serde_json::to_string(params).unwrap_or_else(|_| "{}".to_string());

        let layout_info: Vec<String> = layouts
            .iter()
            .map(|l| format!("Layout at {}", l.path))
            .collect();

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
    <div id="__next">
        <!-- SSR Content -->
        <div data-layouts="{layout_count}">
            {layout_stack}
            <main data-page="{page}">
                <h1>Route: {route}</h1>
                <p>Page: {page}</p>
                <pre>Params: {params}</pre>
            </main>
        </div>
    </div>
    <script type="module">
        // Hydration will happen here
        console.log('next.rs hydration ready');
    </script>
</body>
</html>"#,
            route = route_path,
            params = params_json,
            page = page_name,
            layout_count = layouts.len(),
            layout_stack = layout_info.join("\n            "),
        )
    }

    pub fn render_not_found(&self) -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>404 | next.rs</title>
</head>
<body>
    <div id="__next">
        <main>
            <h1>404 - Page Not Found</h1>
            <p>The page you're looking for doesn't exist.</p>
        </main>
    </div>
</body>
</html>"#
            .to_string()
    }

    pub fn render_error(&self, error: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Error | next.rs</title>
</head>
<body>
    <div id="__next">
        <main>
            <h1>Something went wrong</h1>
            <pre>{}</pre>
        </main>
    </div>
</body>
</html>"#,
            error
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use next_rs_router::Layout;
    use std::path::PathBuf;

    #[test]
    fn test_render_basic_page() {
        let app_dir = PathBuf::from("/app");
        let renderer = SsrRenderer::new(app_dir.clone());

        let mut tree = LayoutTree::new(PathBuf::from("/app/page.rs"));
        tree.add_layout(Layout {
            file: PathBuf::from("/app/layout.rs"),
            path: "/".to_string(),
        });

        let params = HashMap::new();
        let html = renderer.render(&tree, &params);

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("__next"));
        assert!(html.contains("data-layouts=\"1\""));
    }

    #[test]
    fn test_render_with_params() {
        let app_dir = PathBuf::from("/app");
        let renderer = SsrRenderer::new(app_dir);

        let tree = LayoutTree::new(PathBuf::from("/app/blog/[slug]/page.rs"));
        let mut params = HashMap::new();
        params.insert("slug".to_string(), "hello-world".to_string());

        let html = renderer.render(&tree, &params);

        assert!(html.contains("hello-world"));
    }

    #[test]
    fn test_render_not_found() {
        let renderer = SsrRenderer::new(PathBuf::from("/app"));
        let html = renderer.render_not_found();

        assert!(html.contains("404"));
        assert!(html.contains("Page Not Found"));
    }

    #[test]
    fn test_render_error() {
        let renderer = SsrRenderer::new(PathBuf::from("/app"));
        let html = renderer.render_error("Test error message");

        assert!(html.contains("Something went wrong"));
        assert!(html.contains("Test error message"));
    }
}
