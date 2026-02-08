use crate::directive::{DirectiveInfo, DirectiveRegistry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientComponentManifest {
    pub modules: HashMap<String, ClientModuleEntry>,
}

impl Default for ClientComponentManifest {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientComponentManifest {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, id: String, entry: ClientModuleEntry) {
        self.modules.insert(id, entry);
    }

    pub fn get_module(&self, id: &str) -> Option<&ClientModuleEntry> {
        self.modules.get(id)
    }

    pub fn from_registry(registry: &DirectiveRegistry) -> Self {
        let mut manifest = Self::new();

        for info in registry.client_modules() {
            let entry = ClientModuleEntry {
                id: info.full_id(),
                name: info.export_name.clone(),
                chunks: vec![format!("{}.wasm", info.module_id)],
                async_module: true,
            };
            manifest.add_module(info.full_id(), entry);
        }

        manifest
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientModuleEntry {
    pub id: String,
    pub name: String,
    pub chunks: Vec<String>,
    #[serde(rename = "async")]
    pub async_module: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerActionManifest {
    pub actions: HashMap<String, ServerActionEntry>,
}

impl Default for ServerActionManifest {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerActionManifest {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn add_action(&mut self, id: String, entry: ServerActionEntry) {
        self.actions.insert(id, entry);
    }

    pub fn get_action(&self, id: &str) -> Option<&ServerActionEntry> {
        self.actions.get(id)
    }

    pub fn from_registry(registry: &DirectiveRegistry) -> Self {
        let mut manifest = Self::new();

        for info in registry.server_modules() {
            let entry = ServerActionEntry {
                id: info.full_id(),
                name: info.export_name.clone(),
                module: info.module_id.clone(),
            };
            manifest.add_action(info.full_id(), entry);
        }

        manifest
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerActionEntry {
    pub id: String,
    pub name: String,
    pub module: String,
}

pub struct ComponentRegistry {
    directives: Arc<DirectiveRegistry>,
    client_manifest: RwLock<ClientComponentManifest>,
    server_manifest: RwLock<ServerActionManifest>,
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            directives: Arc::new(DirectiveRegistry::new()),
            client_manifest: RwLock::new(ClientComponentManifest::new()),
            server_manifest: RwLock::new(ServerActionManifest::new()),
        }
    }

    pub fn with_registry(directives: Arc<DirectiveRegistry>) -> Self {
        Self {
            directives,
            client_manifest: RwLock::new(ClientComponentManifest::new()),
            server_manifest: RwLock::new(ServerActionManifest::new()),
        }
    }

    pub fn register_client(&self, info: DirectiveInfo) {
        self.directives.register(info.clone());
        let entry = ClientModuleEntry {
            id: info.full_id(),
            name: info.export_name.clone(),
            chunks: vec![format!("{}.wasm", info.module_id)],
            async_module: true,
        };
        self.client_manifest
            .write()
            .unwrap()
            .add_module(info.full_id(), entry);
    }

    pub fn register_server(&self, info: DirectiveInfo) {
        self.directives.register(info.clone());
        let entry = ServerActionEntry {
            id: info.full_id(),
            name: info.export_name.clone(),
            module: info.module_id.clone(),
        };
        self.server_manifest
            .write()
            .unwrap()
            .add_action(info.full_id(), entry);
    }

    pub fn client_manifest(&self) -> ClientComponentManifest {
        self.client_manifest.read().unwrap().clone()
    }

    pub fn server_manifest(&self) -> ServerActionManifest {
        self.server_manifest.read().unwrap().clone()
    }

    pub fn rebuild_manifests(&self) {
        *self.client_manifest.write().unwrap() =
            ClientComponentManifest::from_registry(&self.directives);
        *self.server_manifest.write().unwrap() =
            ServerActionManifest::from_registry(&self.directives);
    }

    pub fn is_client(&self, module_id: &str, export_name: &str) -> bool {
        self.directives.is_client(module_id, export_name)
    }

    pub fn is_server(&self, module_id: &str, export_name: &str) -> bool {
        self.directives.is_server(module_id, export_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_component_manifest() {
        let mut manifest = ClientComponentManifest::new();
        manifest.add_module(
            "./Counter#Counter".to_string(),
            ClientModuleEntry {
                id: "./Counter#Counter".to_string(),
                name: "Counter".to_string(),
                chunks: vec!["Counter.wasm".to_string()],
                async_module: true,
            },
        );

        assert!(manifest.get_module("./Counter#Counter").is_some());
        let json = manifest.to_json();
        assert!(json.contains("Counter"));
    }

    #[test]
    fn test_server_action_manifest() {
        let mut manifest = ServerActionManifest::new();
        manifest.add_action(
            "./actions#createTodo".to_string(),
            ServerActionEntry {
                id: "./actions#createTodo".to_string(),
                name: "createTodo".to_string(),
                module: "./actions".to_string(),
            },
        );

        assert!(manifest.get_action("./actions#createTodo").is_some());
        let json = manifest.to_json();
        assert!(json.contains("createTodo"));
    }

    #[test]
    fn test_component_registry() {
        let registry = ComponentRegistry::new();

        registry.register_client(DirectiveInfo::client("./Counter", "Counter"));
        registry.register_server(DirectiveInfo::server("./actions", "submitForm"));

        assert!(registry.is_client("./Counter", "Counter"));
        assert!(registry.is_server("./actions", "submitForm"));

        let client_manifest = registry.client_manifest();
        assert!(client_manifest.get_module("./Counter#Counter").is_some());

        let server_manifest = registry.server_manifest();
        assert!(server_manifest.get_action("./actions#submitForm").is_some());
    }

    #[test]
    fn test_manifest_from_registry() {
        let directives = DirectiveRegistry::new();
        directives.register_client("./Button", "Button");
        directives.register_client("./Modal", "Modal");
        directives.register_server("./api", "fetchData");

        let client_manifest = ClientComponentManifest::from_registry(&directives);
        assert_eq!(client_manifest.modules.len(), 2);

        let server_manifest = ServerActionManifest::from_registry(&directives);
        assert_eq!(server_manifest.actions.len(), 1);
    }
}
