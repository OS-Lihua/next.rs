use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use next_rs_router::Route;
use next_rs_router::Router;

use crate::ssr::{PageRegistry, SsrRenderer};

pub struct StaticGenerator {
    router: Router,
    #[allow(dead_code)]
    app_dir: PathBuf,
    output_dir: PathBuf,
    renderer: SsrRenderer,
    registry: Arc<PageRegistry>,
}

pub struct GenerationResult {
    pub pages_generated: usize,
    pub total_size_bytes: u64,
    pub files: Vec<GeneratedFile>,
}

pub struct GeneratedFile {
    pub route: String,
    pub file_path: PathBuf,
    pub size_bytes: u64,
}

impl StaticGenerator {
    pub fn new(
        router: Router,
        app_dir: PathBuf,
        output_dir: PathBuf,
        registry: Arc<PageRegistry>,
    ) -> Self {
        let renderer = SsrRenderer::new();
        Self {
            router,
            app_dir,
            output_dir,
            renderer,
            registry,
        }
    }

    pub fn generate(&self) -> anyhow::Result<GenerationResult> {
        fs::create_dir_all(&self.output_dir)?;

        let static_routes: Vec<&Route> = self
            .router
            .routes
            .iter()
            .filter(|r| !r.is_dynamic() && !r.is_api())
            .collect();

        let mut result = GenerationResult {
            pages_generated: 0,
            total_size_bytes: 0,
            files: Vec::new(),
        };

        for route in static_routes {
            let html = self.renderer.render(
                &route.path,
                &std::collections::HashMap::new(),
                &self.registry,
            );

            let file_path = self.route_to_file_path(&route.path);
            let full_path = self.output_dir.join(&file_path);

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&full_path, &html)?;

            let size = html.len() as u64;
            result.pages_generated += 1;
            result.total_size_bytes += size;
            result.files.push(GeneratedFile {
                route: route.path.clone(),
                file_path: full_path,
                size_bytes: size,
            });
        }

        self.generate_not_found(&mut result)?;

        Ok(result)
    }

    fn route_to_file_path(&self, route: &str) -> PathBuf {
        if route == "/" {
            PathBuf::from("index.html")
        } else {
            let clean_route = route.trim_start_matches('/');
            PathBuf::from(format!("{}/index.html", clean_route))
        }
    }

    fn generate_not_found(&self, result: &mut GenerationResult) -> anyhow::Result<()> {
        let html = self.renderer.render_not_found();
        let file_path = self.output_dir.join("404.html");

        fs::write(&file_path, &html)?;

        let size = html.len() as u64;
        result.pages_generated += 1;
        result.total_size_bytes += size;
        result.files.push(GeneratedFile {
            route: "404".to_string(),
            file_path,
            size_bytes: size,
        });

        Ok(())
    }
}

pub struct StaticParams {
    pub params: Vec<std::collections::HashMap<String, String>>,
}

impl StaticParams {
    pub fn new() -> Self {
        Self { params: Vec::new() }
    }

    pub fn add(&mut self, params: std::collections::HashMap<String, String>) {
        self.params.push(params);
    }

    pub fn from_slugs(param_name: &str, slugs: Vec<&str>) -> Self {
        let mut static_params = Self::new();
        for slug in slugs {
            let mut map = std::collections::HashMap::new();
            map.insert(param_name.to_string(), slug.to_string());
            static_params.add(map);
        }
        static_params
    }
}

impl Default for StaticParams {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn create_test_app() -> (TempDir, PathBuf) {
        let temp = TempDir::new().unwrap();
        let app = temp.path().join("app");

        fs::create_dir_all(&app).unwrap();
        File::create(app.join("page.rs")).unwrap();

        fs::create_dir_all(app.join("about")).unwrap();
        File::create(app.join("about/page.rs")).unwrap();

        fs::create_dir_all(app.join("blog")).unwrap();
        File::create(app.join("blog/page.rs")).unwrap();

        fs::create_dir_all(app.join("blog/[slug]")).unwrap();
        File::create(app.join("blog/[slug]/page.rs")).unwrap();

        (temp, app)
    }

    #[test]
    fn test_static_generation() {
        let (temp, app_dir) = create_test_app();
        let output_dir = temp.path().join("dist");

        let scanner = next_rs_router::RouteScanner::new(&app_dir);
        let routes = scanner.scan();
        let router = Router::from_routes(routes);

        let registry = Arc::new(PageRegistry::new());
        let generator = StaticGenerator::new(router, app_dir, output_dir.clone(), registry);
        let result = generator.generate().unwrap();

        assert_eq!(result.pages_generated, 4);

        assert!(output_dir.join("index.html").exists());
        assert!(output_dir.join("about/index.html").exists());
        assert!(output_dir.join("blog/index.html").exists());
        assert!(output_dir.join("404.html").exists());
    }

    #[test]
    fn test_route_to_file_path() {
        let temp = TempDir::new().unwrap();
        let app_dir = temp.path().join("app");
        let output_dir = temp.path().join("dist");

        let registry = Arc::new(PageRegistry::new());
        let generator = StaticGenerator::new(Router::new(), app_dir, output_dir, registry);

        assert_eq!(
            generator.route_to_file_path("/"),
            PathBuf::from("index.html")
        );
        assert_eq!(
            generator.route_to_file_path("/about"),
            PathBuf::from("about/index.html")
        );
        assert_eq!(
            generator.route_to_file_path("/blog/posts"),
            PathBuf::from("blog/posts/index.html")
        );
    }

    #[test]
    fn test_static_params() {
        let params = StaticParams::from_slugs("slug", vec!["hello", "world", "test"]);
        assert_eq!(params.params.len(), 3);
        assert_eq!(params.params[0].get("slug"), Some(&"hello".to_string()));
    }
}
