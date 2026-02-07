use crate::Route;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum SpecialFile {
    Page,
    Layout,
    Loading,
    Error,
    NotFound,
    Route,
}

impl SpecialFile {
    pub fn from_filename(name: &str) -> Option<Self> {
        match name {
            "page.rs" | "page.tsx" | "page.js" => Some(SpecialFile::Page),
            "layout.rs" | "layout.tsx" | "layout.js" => Some(SpecialFile::Layout),
            "loading.rs" | "loading.tsx" | "loading.js" => Some(SpecialFile::Loading),
            "error.rs" | "error.tsx" | "error.js" => Some(SpecialFile::Error),
            "not-found.rs" | "not-found.tsx" | "not-found.js" => Some(SpecialFile::NotFound),
            "route.rs" | "route.tsx" | "route.js" => Some(SpecialFile::Route),
            _ => None,
        }
    }
}

pub struct RouteScanner {
    app_dir: PathBuf,
}

impl RouteScanner {
    pub fn new(app_dir: impl Into<PathBuf>) -> Self {
        Self {
            app_dir: app_dir.into(),
        }
    }

    pub fn scan(&self) -> Vec<Route> {
        let mut routes = Vec::new();
        self.scan_dir(&self.app_dir, "", &mut routes);
        routes.sort_by(|a, b| a.path.cmp(&b.path));
        routes
    }

    fn scan_dir(&self, dir: &Path, route_path: &str, routes: &mut Vec<Route>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        let mut route = Route::new(if route_path.is_empty() {
            "/"
        } else {
            route_path
        });

        let mut has_page = false;
        let mut subdirs = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            if path.is_file() {
                if let Some(special) = SpecialFile::from_filename(&name) {
                    match special {
                        SpecialFile::Page => {
                            route.page_file = Some(path);
                            has_page = true;
                        }
                        SpecialFile::Layout => {
                            route.layout_file = Some(path);
                        }
                        SpecialFile::Loading => {
                            route.loading_file = Some(path);
                        }
                        SpecialFile::Error => {
                            route.error_file = Some(path);
                        }
                        SpecialFile::NotFound => {
                            route.not_found_file = Some(path);
                        }
                        SpecialFile::Route => {
                            route.route_file = Some(path);
                            has_page = true;
                        }
                    }
                }
            } else if path.is_dir() {
                subdirs.push((path, name.to_string()));
            }
        }

        if has_page {
            routes.push(route);
        }

        for (subdir, name) in subdirs {
            let segment = dir_name_to_segment(&name);
            let new_path = if route_path.is_empty() {
                format!("/{}", segment)
            } else {
                format!("{}/{}", route_path, segment)
            };
            self.scan_dir(&subdir, &new_path, routes);
        }
    }
}

fn dir_name_to_segment(name: &str) -> String {
    if name.starts_with('(') && name.ends_with(')') {
        String::new()
    } else {
        name.to_string()
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
        File::create(app.join("page.rs")).unwrap();
        File::create(app.join("layout.rs")).unwrap();

        fs::create_dir_all(app.join("about")).unwrap();
        File::create(app.join("about/page.rs")).unwrap();

        fs::create_dir_all(app.join("blog/[slug]")).unwrap();
        File::create(app.join("blog/[slug]/page.rs")).unwrap();

        fs::create_dir_all(app.join("api/users")).unwrap();
        File::create(app.join("api/users/route.rs")).unwrap();

        temp
    }

    #[test]
    fn test_scan_routes() {
        let temp = create_test_structure();
        let app_dir = temp.path().join("app");
        let scanner = RouteScanner::new(&app_dir);
        let routes = scanner.scan();

        assert_eq!(routes.len(), 4);

        let paths: Vec<&str> = routes.iter().map(|r| r.path.as_str()).collect();
        assert!(paths.contains(&"/"));
        assert!(paths.contains(&"/about"));
        assert!(paths.contains(&"/blog/[slug]"));
        assert!(paths.contains(&"/api/users"));
    }

    #[test]
    fn test_special_file_detection() {
        assert_eq!(
            SpecialFile::from_filename("page.rs"),
            Some(SpecialFile::Page)
        );
        assert_eq!(
            SpecialFile::from_filename("layout.rs"),
            Some(SpecialFile::Layout)
        );
        assert_eq!(
            SpecialFile::from_filename("route.rs"),
            Some(SpecialFile::Route)
        );
        assert_eq!(SpecialFile::from_filename("utils.rs"), None);
    }

    #[test]
    fn test_api_route() {
        let temp = create_test_structure();
        let app_dir = temp.path().join("app");
        let scanner = RouteScanner::new(&app_dir);
        let routes = scanner.scan();

        let api_route = routes.iter().find(|r| r.path == "/api/users").unwrap();
        assert!(api_route.is_api());
    }

    #[test]
    fn test_dynamic_route() {
        let temp = create_test_structure();
        let app_dir = temp.path().join("app");
        let scanner = RouteScanner::new(&app_dir);
        let routes = scanner.scan();

        let blog_route = routes.iter().find(|r| r.path == "/blog/[slug]").unwrap();
        assert!(blog_route.is_dynamic());
    }
}
