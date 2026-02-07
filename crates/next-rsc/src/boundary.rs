use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ServerBoundary {
    component_ids: HashSet<String>,
}

impl ServerBoundary {
    pub fn new() -> Self {
        Self {
            component_ids: HashSet::new(),
        }
    }

    pub fn register(&mut self, component_id: impl Into<String>) {
        self.component_ids.insert(component_id.into());
    }

    pub fn is_server_component(&self, component_id: &str) -> bool {
        self.component_ids.contains(component_id)
    }

    pub fn server_components(&self) -> impl Iterator<Item = &String> {
        self.component_ids.iter()
    }
}

impl Default for ServerBoundary {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ClientBoundary {
    modules: Vec<ClientModule>,
}

#[derive(Debug, Clone)]
pub struct ClientModule {
    pub id: String,
    pub path: String,
    pub exports: Vec<String>,
}

impl ClientBoundary {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn register_module(&mut self, id: impl Into<String>, path: impl Into<String>) {
        self.modules.push(ClientModule {
            id: id.into(),
            path: path.into(),
            exports: Vec::new(),
        });
    }

    pub fn register_export(&mut self, module_id: &str, export_name: impl Into<String>) {
        if let Some(module) = self.modules.iter_mut().find(|m| m.id == module_id) {
            module.exports.push(export_name.into());
        }
    }

    pub fn get_module(&self, id: &str) -> Option<&ClientModule> {
        self.modules.iter().find(|m| m.id == id)
    }

    pub fn all_modules(&self) -> &[ClientModule] {
        &self.modules
    }

    pub fn client_manifest(&self) -> serde_json::Value {
        serde_json::json!({
            "modules": self.modules.iter().map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "path": m.path,
                    "exports": m.exports,
                })
            }).collect::<Vec<_>>()
        })
    }
}

impl Default for ClientBoundary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_boundary() {
        let mut boundary = ServerBoundary::new();
        boundary.register("ArticleList");
        boundary.register("UserProfile");

        assert!(boundary.is_server_component("ArticleList"));
        assert!(boundary.is_server_component("UserProfile"));
        assert!(!boundary.is_server_component("Counter"));
    }

    #[test]
    fn test_client_boundary() {
        let mut boundary = ClientBoundary::new();
        boundary.register_module("counter", "./components/Counter.js");
        boundary.register_export("counter", "Counter");
        boundary.register_export("counter", "useCounter");

        let module = boundary.get_module("counter").unwrap();
        assert_eq!(module.path, "./components/Counter.js");
        assert_eq!(module.exports.len(), 2);
    }

    #[test]
    fn test_client_manifest() {
        let mut boundary = ClientBoundary::new();
        boundary.register_module("mod1", "./mod1.js");
        boundary.register_export("mod1", "Component1");

        let manifest = boundary.client_manifest();
        assert!(manifest["modules"].is_array());
        assert_eq!(manifest["modules"][0]["id"], "mod1");
    }
}
