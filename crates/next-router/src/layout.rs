use std::path::{Path, PathBuf};

use crate::{Layout, LayoutTree, Route};

pub struct LayoutResolver {
    app_dir: PathBuf,
}

impl LayoutResolver {
    pub fn new(app_dir: impl Into<PathBuf>) -> Self {
        Self {
            app_dir: app_dir.into(),
        }
    }

    pub fn resolve(&self, route: &Route) -> LayoutTree {
        let page = route
            .page_file
            .clone()
            .expect("Route must have a page file");

        let mut tree = LayoutTree::new(page);

        let path_segments: Vec<&str> = route.path.split('/').filter(|s| !s.is_empty()).collect();

        if let Some(root_layout) = self.find_layout(&self.app_dir) {
            tree.add_layout(Layout {
                file: root_layout,
                path: "/".to_string(),
            });
        }

        let mut current_path = self.app_dir.clone();
        let mut route_path = String::new();

        for segment in path_segments {
            current_path = current_path.join(segment);
            route_path = format!("{}/{}", route_path, segment);

            if let Some(layout_file) = self.find_layout(&current_path) {
                tree.add_layout(Layout {
                    file: layout_file,
                    path: route_path.clone(),
                });
            }
        }

        tree
    }

    fn find_layout(&self, dir: &Path) -> Option<PathBuf> {
        let candidates = ["layout.rs", "layout.tsx", "layout.js"];

        for candidate in candidates {
            let path = dir.join(candidate);
            if path.exists() {
                return Some(path);
            }
        }

        None
    }
}

pub struct RouteMetadata {
    pub loading_file: Option<PathBuf>,
    pub error_file: Option<PathBuf>,
    pub not_found_file: Option<PathBuf>,
}

impl RouteMetadata {
    pub fn from_route(route: &Route) -> Self {
        Self {
            loading_file: route.loading_file.clone(),
            error_file: route.error_file.clone(),
            not_found_file: route.not_found_file.clone(),
        }
    }

    pub fn has_loading(&self) -> bool {
        self.loading_file.is_some()
    }

    pub fn has_error_boundary(&self) -> bool {
        self.error_file.is_some()
    }

    pub fn has_not_found(&self) -> bool {
        self.not_found_file.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn create_test_structure() -> TempDir {
        let temp = TempDir::new().unwrap();
        let app = temp.path().join("app");

        fs::create_dir_all(&app).unwrap();
        File::create(app.join("layout.rs")).unwrap();
        File::create(app.join("page.rs")).unwrap();

        fs::create_dir_all(app.join("blog")).unwrap();
        File::create(app.join("blog/layout.rs")).unwrap();
        File::create(app.join("blog/page.rs")).unwrap();

        fs::create_dir_all(app.join("blog/[slug]")).unwrap();
        File::create(app.join("blog/[slug]/page.rs")).unwrap();

        temp
    }

    #[test]
    fn test_resolve_root_layout() {
        let temp = create_test_structure();
        let app_dir = temp.path().join("app");

        let resolver = LayoutResolver::new(&app_dir);

        let route = Route::new("/").with_page(app_dir.join("page.rs"));
        let tree = resolver.resolve(&route);

        assert_eq!(tree.layouts.len(), 1);
        assert_eq!(tree.layouts[0].path, "/");
    }

    #[test]
    fn test_resolve_nested_layouts() {
        let temp = create_test_structure();
        let app_dir = temp.path().join("app");

        let resolver = LayoutResolver::new(&app_dir);

        let route = Route::new("/blog").with_page(app_dir.join("blog/page.rs"));
        let tree = resolver.resolve(&route);

        assert_eq!(tree.layouts.len(), 2);
        assert_eq!(tree.layouts[0].path, "/");
        assert_eq!(tree.layouts[1].path, "/blog");
    }

    #[test]
    fn test_resolve_dynamic_route_layouts() {
        let temp = create_test_structure();
        let app_dir = temp.path().join("app");

        let resolver = LayoutResolver::new(&app_dir);

        let route = Route::new("/blog/[slug]").with_page(app_dir.join("blog/[slug]/page.rs"));
        let tree = resolver.resolve(&route);

        assert_eq!(tree.layouts.len(), 2);
        assert_eq!(tree.layouts[0].path, "/");
        assert_eq!(tree.layouts[1].path, "/blog");
    }

    #[test]
    fn test_route_metadata() {
        let mut route = Route::new("/");
        route.loading_file = Some(PathBuf::from("loading.rs"));
        route.error_file = Some(PathBuf::from("error.rs"));

        let metadata = RouteMetadata::from_route(&route);

        assert!(metadata.has_loading());
        assert!(metadata.has_error_boundary());
        assert!(!metadata.has_not_found());
    }
}
