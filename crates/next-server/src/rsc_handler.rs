use std::collections::HashMap;
use std::path::PathBuf;

use next_rs_rsc::{ClientBoundary, RscNode, RscPayload, ServerBoundary};
use serde_json::json;

pub struct RscHandler {
    app_dir: PathBuf,
    server_boundary: ServerBoundary,
    client_boundary: ClientBoundary,
}

impl RscHandler {
    pub fn new(app_dir: PathBuf) -> Self {
        Self {
            app_dir,
            server_boundary: ServerBoundary::new(),
            client_boundary: ClientBoundary::new(),
        }
    }

    pub fn render_route(&self, route_path: &str, params: &HashMap<String, String>) -> RscPayload {
        let mut payload = RscPayload::new();

        let route_info = json!({
            "path": route_path,
            "params": params,
        });

        let page_node = RscNode::element(
            "div",
            json!({"data-rsc-root": true}),
            vec![
                RscNode::element(
                    "script",
                    json!({"type": "application/json", "id": "__RSC_DATA__"}),
                    vec![RscNode::text(route_info.to_string())],
                ),
                RscNode::element(
                    "main",
                    json!({"data-page": route_path}),
                    vec![
                        RscNode::element(
                            "h1",
                            json!({}),
                            vec![RscNode::text(format!("Route: {}", route_path))],
                        ),
                        RscNode::element(
                            "pre",
                            json!({}),
                            vec![RscNode::text(format!("Params: {:?}", params))],
                        ),
                    ],
                ),
            ],
        );

        payload.add_node(page_node);
        payload
    }

    pub fn render_to_wire_format(
        &self,
        route_path: &str,
        params: &HashMap<String, String>,
    ) -> String {
        let payload = self.render_route(route_path, params);
        payload.to_wire_format()
    }

    pub fn render_to_json(
        &self,
        route_path: &str,
        params: &HashMap<String, String>,
    ) -> serde_json::Value {
        let payload = self.render_route(route_path, params);
        payload.to_json()
    }

    pub fn register_server_component(&mut self, component_id: impl Into<String>) {
        self.server_boundary.register(component_id);
    }

    pub fn register_client_module(&mut self, id: impl Into<String>, path: impl Into<String>) {
        self.client_boundary.register_module(id, path);
    }

    pub fn client_manifest(&self) -> serde_json::Value {
        self.client_boundary.client_manifest()
    }

    pub fn app_dir(&self) -> &PathBuf {
        &self.app_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_rsc_handler_creation() {
        let handler = RscHandler::new(PathBuf::from("/app"));
        assert_eq!(handler.app_dir(), &PathBuf::from("/app"));
    }

    #[test]
    fn test_render_route_to_payload() {
        let handler = RscHandler::new(PathBuf::from("/app"));
        let mut params = HashMap::new();
        params.insert("slug".to_string(), "hello".to_string());

        let payload = handler.render_route("/blog/[slug]", &params);
        assert!(!payload.nodes.is_empty());
    }

    #[test]
    fn test_render_to_wire_format() {
        let handler = RscHandler::new(PathBuf::from("/app"));
        let params = HashMap::new();

        let wire = handler.render_to_wire_format("/", &params);
        assert!(wire.contains("div"));
        assert!(wire.contains("data-rsc-root"));
    }

    #[test]
    fn test_register_components() {
        let mut handler = RscHandler::new(PathBuf::from("/app"));

        handler.register_server_component("ArticleList");
        handler.register_client_module("counter", "./Counter.js");

        let manifest = handler.client_manifest();
        assert!(manifest["modules"].is_array());
    }

    #[test]
    fn test_render_to_json() {
        let handler = RscHandler::new(PathBuf::from("/app"));
        let params = HashMap::new();

        let json = handler.render_to_json("/about", &params);
        assert!(json["nodes"].is_array());
    }
}
