use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Directive {
    UseClient,
    UseServer,
}

impl Directive {
    pub fn as_str(&self) -> &'static str {
        match self {
            Directive::UseClient => "use client",
            Directive::UseServer => "use server",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDirectiveError;

impl std::fmt::Display for ParseDirectiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid directive")
    }
}

impl std::error::Error for ParseDirectiveError {}

impl FromStr for Directive {
    type Err = ParseDirectiveError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().trim_matches('"').trim_matches('\'') {
            "use client" => Ok(Directive::UseClient),
            "use server" => Ok(Directive::UseServer),
            _ => Err(ParseDirectiveError),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DirectiveInfo {
    pub directive: Directive,
    pub module_id: String,
    pub export_name: String,
}

impl DirectiveInfo {
    pub fn client(module_id: impl Into<String>, export_name: impl Into<String>) -> Self {
        Self {
            directive: Directive::UseClient,
            module_id: module_id.into(),
            export_name: export_name.into(),
        }
    }

    pub fn server(module_id: impl Into<String>, export_name: impl Into<String>) -> Self {
        Self {
            directive: Directive::UseServer,
            module_id: module_id.into(),
            export_name: export_name.into(),
        }
    }

    pub fn is_client(&self) -> bool {
        self.directive == Directive::UseClient
    }

    pub fn is_server(&self) -> bool {
        self.directive == Directive::UseServer
    }

    pub fn full_id(&self) -> String {
        format!("{}#{}", self.module_id, self.export_name)
    }
}

pub struct DirectiveRegistry {
    directives: RwLock<HashMap<String, DirectiveInfo>>,
}

impl Default for DirectiveRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectiveRegistry {
    pub fn new() -> Self {
        Self {
            directives: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, info: DirectiveInfo) {
        let key = info.full_id();
        self.directives.write().unwrap().insert(key, info);
    }

    pub fn register_client(&self, module_id: impl Into<String>, export_name: impl Into<String>) {
        self.register(DirectiveInfo::client(module_id, export_name));
    }

    pub fn register_server(&self, module_id: impl Into<String>, export_name: impl Into<String>) {
        self.register(DirectiveInfo::server(module_id, export_name));
    }

    pub fn get(&self, module_id: &str, export_name: &str) -> Option<DirectiveInfo> {
        let key = format!("{}#{}", module_id, export_name);
        self.directives.read().unwrap().get(&key).cloned()
    }

    pub fn is_client(&self, module_id: &str, export_name: &str) -> bool {
        self.get(module_id, export_name)
            .map(|info| info.is_client())
            .unwrap_or(false)
    }

    pub fn is_server(&self, module_id: &str, export_name: &str) -> bool {
        self.get(module_id, export_name)
            .map(|info| info.is_server())
            .unwrap_or(false)
    }

    pub fn client_modules(&self) -> Vec<DirectiveInfo> {
        self.directives
            .read()
            .unwrap()
            .values()
            .filter(|info| info.is_client())
            .cloned()
            .collect()
    }

    pub fn server_modules(&self) -> Vec<DirectiveInfo> {
        self.directives
            .read()
            .unwrap()
            .values()
            .filter(|info| info.is_server())
            .cloned()
            .collect()
    }
}

static GLOBAL_REGISTRY: std::sync::OnceLock<Arc<DirectiveRegistry>> = std::sync::OnceLock::new();

pub fn global_registry() -> &'static Arc<DirectiveRegistry> {
    GLOBAL_REGISTRY.get_or_init(|| Arc::new(DirectiveRegistry::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directive_from_str() {
        assert_eq!("use client".parse(), Ok(Directive::UseClient));
        assert_eq!("\"use client\"".parse(), Ok(Directive::UseClient));
        assert_eq!("use server".parse(), Ok(Directive::UseServer));
        assert_eq!("'use server'".parse(), Ok(Directive::UseServer));
        assert!("use strict".parse::<Directive>().is_err());
    }

    #[test]
    fn test_directive_info() {
        let client = DirectiveInfo::client("./Counter", "Counter");
        assert!(client.is_client());
        assert!(!client.is_server());
        assert_eq!(client.full_id(), "./Counter#Counter");

        let server = DirectiveInfo::server("./actions", "submitForm");
        assert!(server.is_server());
        assert!(!server.is_client());
        assert_eq!(server.full_id(), "./actions#submitForm");
    }

    #[test]
    fn test_directive_registry() {
        let registry = DirectiveRegistry::new();

        registry.register_client("./Counter", "Counter");
        registry.register_client("./Button", "Button");
        registry.register_server("./actions", "createTodo");

        assert!(registry.is_client("./Counter", "Counter"));
        assert!(registry.is_client("./Button", "Button"));
        assert!(registry.is_server("./actions", "createTodo"));

        assert!(!registry.is_server("./Counter", "Counter"));
        assert!(!registry.is_client("./actions", "createTodo"));

        let clients = registry.client_modules();
        assert_eq!(clients.len(), 2);

        let servers = registry.server_modules();
        assert_eq!(servers.len(), 1);
    }

    #[test]
    fn test_global_registry() {
        let registry = global_registry();
        registry.register_client("./test-module", "TestComponent");
        assert!(registry.is_client("./test-module", "TestComponent"));
    }
}
