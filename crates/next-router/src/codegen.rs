use std::fs;
use std::path::{Path, PathBuf};

use crate::scanner::SpecialFile;

pub struct RouteCodegen {
    app_dir: PathBuf,
}

impl RouteCodegen {
    pub fn new(app_dir: impl Into<PathBuf>) -> Self {
        Self {
            app_dir: app_dir.into(),
        }
    }

    pub fn generate(&self) -> String {
        let mut pages = Vec::new();
        let mut layouts = Vec::new();

        self.scan_dir(&self.app_dir, "", &mut pages, &mut layouts);

        let mut code = String::new();

        code.push_str("use next_rs_server::PageRegistry;\n");
        code.push_str("use react_rs_elements::node::IntoNode;\n\n");

        for (mod_path, _route_path) in &pages {
            code.push_str(&format!(
                "#[path = \"{}\"]\nmod {};\n",
                mod_path,
                self.mod_name(mod_path)
            ));
        }
        for (mod_path, _route_path) in &layouts {
            code.push_str(&format!(
                "#[path = \"{}\"]\nmod {};\n",
                mod_path,
                self.mod_name(mod_path)
            ));
        }

        code.push_str("\npub fn auto_register() -> PageRegistry {\n");
        code.push_str("    let mut registry = PageRegistry::new();\n");

        for (mod_path, route_path) in &pages {
            let mod_name = self.mod_name(mod_path);
            code.push_str(&format!(
                "    registry.register_page(\"{}\", |_params| {}::page().into_node());\n",
                route_path, mod_name
            ));
        }

        for (mod_path, route_path) in &layouts {
            let mod_name = self.mod_name(mod_path);
            code.push_str(&format!(
                "    registry.register_layout(\"{}\", |children| {}::layout(children).into_node());\n",
                route_path, mod_name
            ));
        }

        code.push_str("    registry\n");
        code.push_str("}\n");

        code
    }

    pub fn generate_simple(&self) -> String {
        let mut pages = Vec::new();
        let mut layouts = Vec::new();

        self.scan_dir(&self.app_dir, "", &mut pages, &mut layouts);

        let mut registrations = Vec::new();

        for (file_path, route_path) in &pages {
            registrations.push(format!("(\"{}\", \"page\", \"{}\")", route_path, file_path));
        }

        for (file_path, route_path) in &layouts {
            registrations.push(format!(
                "(\"{}\", \"layout\", \"{}\")",
                route_path, file_path
            ));
        }

        let mut code = String::new();
        code.push_str("pub const ROUTE_TABLE: &[(&str, &str, &str)] = &[\n");
        for reg in &registrations {
            code.push_str(&format!("    {},\n", reg));
        }
        code.push_str("];\n");

        code
    }

    fn scan_dir(
        &self,
        dir: &Path,
        route_path: &str,
        pages: &mut Vec<(String, String)>,
        layouts: &mut Vec<(String, String)>,
    ) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };

        let current_route = if route_path.is_empty() {
            "/".to_string()
        } else {
            route_path.to_string()
        };

        let mut subdirs = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();

            if path.is_file() {
                if let Some(special) = SpecialFile::from_filename(&name) {
                    let rel_path = path
                        .strip_prefix(&self.app_dir)
                        .unwrap_or(&path)
                        .display()
                        .to_string()
                        .replace('\\', "/");

                    match special {
                        SpecialFile::Page => {
                            pages.push((rel_path, current_route.clone()));
                        }
                        SpecialFile::Layout => {
                            layouts.push((rel_path, current_route.clone()));
                        }
                        _ => {}
                    }
                }
            } else if path.is_dir() {
                subdirs.push((path, name.to_string()));
            }
        }

        for (subdir, name) in subdirs {
            let segment = if name.starts_with('(') && name.ends_with(')') {
                String::new()
            } else {
                name
            };

            let new_path = if route_path.is_empty() {
                format!("/{}", segment)
            } else {
                format!("{}/{}", route_path, segment)
            };

            self.scan_dir(&subdir, &new_path, pages, layouts);
        }
    }

    fn mod_name(&self, file_path: &str) -> String {
        file_path
            .replace('/', "_")
            .replace(".rs", "")
            .replace('-', "_")
            .replace('[', "dyn_")
            .replace(']', "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn create_test_app() -> TempDir {
        let temp = TempDir::new().unwrap();
        let app = temp.path().join("app");

        fs::create_dir_all(&app).unwrap();
        File::create(app.join("page.rs")).unwrap();
        File::create(app.join("layout.rs")).unwrap();

        fs::create_dir_all(app.join("about")).unwrap();
        File::create(app.join("about/page.rs")).unwrap();

        fs::create_dir_all(app.join("blog")).unwrap();
        File::create(app.join("blog/page.rs")).unwrap();
        File::create(app.join("blog/layout.rs")).unwrap();

        fs::create_dir_all(app.join("blog/[slug]")).unwrap();
        File::create(app.join("blog/[slug]/page.rs")).unwrap();

        temp
    }

    #[test]
    fn test_codegen_simple() {
        let temp = create_test_app();
        let app_dir = temp.path().join("app");
        let codegen = RouteCodegen::new(&app_dir);

        let code = codegen.generate_simple();

        assert!(code.contains("ROUTE_TABLE"));
        assert!(code.contains("\"/\""));
        assert!(code.contains("\"/about\""));
        assert!(code.contains("\"/blog\""));
        assert!(code.contains("\"page\""));
        assert!(code.contains("\"layout\""));
    }

    #[test]
    fn test_codegen_full() {
        let temp = create_test_app();
        let app_dir = temp.path().join("app");
        let codegen = RouteCodegen::new(&app_dir);

        let code = codegen.generate();

        assert!(code.contains("auto_register"));
        assert!(code.contains("PageRegistry"));
        assert!(code.contains("register_page"));
        assert!(code.contains("register_layout"));
        assert!(code.contains("\"/\""));
        assert!(code.contains("\"/about\""));
    }

    #[test]
    fn test_mod_name() {
        let codegen = RouteCodegen::new("/app");
        assert_eq!(codegen.mod_name("page.rs"), "page");
        assert_eq!(codegen.mod_name("about/page.rs"), "about_page");
        assert_eq!(
            codegen.mod_name("blog/[slug]/page.rs"),
            "blog_dyn_slug_page"
        );
    }
}
