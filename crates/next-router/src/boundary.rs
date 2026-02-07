use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LoadingBoundary {
    pub file: PathBuf,
    pub route_path: String,
}

#[derive(Debug, Clone)]
pub struct ErrorBoundary {
    pub file: PathBuf,
    pub route_path: String,
}

#[derive(Debug, Clone)]
pub struct NotFoundBoundary {
    pub file: PathBuf,
    pub route_path: String,
}

#[derive(Debug, Default)]
pub struct BoundaryStack {
    pub loading: Vec<LoadingBoundary>,
    pub error: Vec<ErrorBoundary>,
    pub not_found: Option<NotFoundBoundary>,
}

impl BoundaryStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_loading(&mut self, file: PathBuf, route_path: String) {
        self.loading.push(LoadingBoundary { file, route_path });
    }

    pub fn add_error(&mut self, file: PathBuf, route_path: String) {
        self.error.push(ErrorBoundary { file, route_path });
    }

    pub fn set_not_found(&mut self, file: PathBuf, route_path: String) {
        self.not_found = Some(NotFoundBoundary { file, route_path });
    }

    pub fn closest_loading(&self) -> Option<&LoadingBoundary> {
        self.loading.last()
    }

    pub fn closest_error(&self) -> Option<&ErrorBoundary> {
        self.error.last()
    }
}

pub struct BoundaryResolver {
    app_dir: PathBuf,
}

impl BoundaryResolver {
    pub fn new(app_dir: impl Into<PathBuf>) -> Self {
        Self {
            app_dir: app_dir.into(),
        }
    }

    pub fn resolve(&self, route_path: &str) -> BoundaryStack {
        let mut stack = BoundaryStack::new();

        let path_segments: Vec<&str> = route_path.split('/').filter(|s| !s.is_empty()).collect();

        self.check_boundaries(&self.app_dir, "/", &mut stack);

        let mut current_dir = self.app_dir.clone();
        let mut current_path = String::new();

        for segment in path_segments {
            current_dir = current_dir.join(segment);
            current_path = format!("{}/{}", current_path, segment);
            self.check_boundaries(&current_dir, &current_path, &mut stack);
        }

        stack
    }

    fn check_boundaries(&self, dir: &std::path::Path, route_path: &str, stack: &mut BoundaryStack) {
        let loading_candidates = ["loading.rs", "loading.tsx", "loading.js"];
        let error_candidates = ["error.rs", "error.tsx", "error.js"];
        let not_found_candidates = ["not-found.rs", "not-found.tsx", "not-found.js"];

        for candidate in loading_candidates {
            let path = dir.join(candidate);
            if path.exists() {
                stack.add_loading(path, route_path.to_string());
                break;
            }
        }

        for candidate in error_candidates {
            let path = dir.join(candidate);
            if path.exists() {
                stack.add_error(path, route_path.to_string());
                break;
            }
        }

        for candidate in not_found_candidates {
            let path = dir.join(candidate);
            if path.exists() {
                stack.set_not_found(path, route_path.to_string());
                break;
            }
        }
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
        File::create(app.join("loading.rs")).unwrap();
        File::create(app.join("error.rs")).unwrap();
        File::create(app.join("not-found.rs")).unwrap();
        File::create(app.join("page.rs")).unwrap();

        fs::create_dir_all(app.join("dashboard")).unwrap();
        File::create(app.join("dashboard/loading.rs")).unwrap();
        File::create(app.join("dashboard/page.rs")).unwrap();

        temp
    }

    #[test]
    fn test_resolve_root_boundaries() {
        let temp = create_test_structure();
        let app_dir = temp.path().join("app");

        let resolver = BoundaryResolver::new(&app_dir);
        let stack = resolver.resolve("/");

        assert_eq!(stack.loading.len(), 1);
        assert_eq!(stack.error.len(), 1);
        assert!(stack.not_found.is_some());
    }

    #[test]
    fn test_resolve_nested_boundaries() {
        let temp = create_test_structure();
        let app_dir = temp.path().join("app");

        let resolver = BoundaryResolver::new(&app_dir);
        let stack = resolver.resolve("/dashboard");

        assert_eq!(stack.loading.len(), 2);
        assert_eq!(stack.error.len(), 1);

        let closest_loading = stack.closest_loading().unwrap();
        assert!(closest_loading.file.ends_with("dashboard/loading.rs"));
    }

    #[test]
    fn test_closest_error_boundary() {
        let temp = create_test_structure();
        let app_dir = temp.path().join("app");

        let resolver = BoundaryResolver::new(&app_dir);
        let stack = resolver.resolve("/dashboard");

        let closest_error = stack.closest_error().unwrap();
        assert!(closest_error.file.ends_with("error.rs"));
        assert_eq!(closest_error.route_path, "/");
    }
}
