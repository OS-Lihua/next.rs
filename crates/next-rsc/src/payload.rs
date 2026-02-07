use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RscPayload {
    pub nodes: Vec<RscNode>,
    pub client_references: Vec<RscRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RscNode {
    #[serde(rename = "element")]
    Element {
        tag: String,
        props: serde_json::Value,
        children: Vec<RscNode>,
    },
    #[serde(rename = "text")]
    Text { value: String },
    #[serde(rename = "client")]
    ClientReference {
        id: String,
        props: serde_json::Value,
    },
    #[serde(rename = "suspense")]
    Suspense {
        id: String,
        fallback: Box<RscNode>,
        children: Vec<RscNode>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RscRef {
    pub id: String,
    pub module: String,
    pub export: String,
}

impl RscPayload {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            client_references: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: RscNode) {
        self.nodes.push(node);
    }

    pub fn add_client_reference(&mut self, id: String, module: String, export: String) {
        self.client_references.push(RscRef { id, module, export });
    }

    pub fn to_wire_format(&self) -> String {
        let mut lines = Vec::new();

        for (i, node) in self.nodes.iter().enumerate() {
            let node_json = serde_json::to_string(node).unwrap_or_else(|_| "null".to_string());
            lines.push(format!("{}:{}", i, node_json));
        }

        for reference in &self.client_references {
            lines.push(format!(
                "M:{}:{}:{}",
                reference.id, reference.module, reference.export
            ));
        }

        lines.join("\n")
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::json!({}))
    }
}

impl Default for RscPayload {
    fn default() -> Self {
        Self::new()
    }
}

impl RscNode {
    pub fn element(
        tag: impl Into<String>,
        props: serde_json::Value,
        children: Vec<RscNode>,
    ) -> Self {
        Self::Element {
            tag: tag.into(),
            props,
            children,
        }
    }

    pub fn text(value: impl Into<String>) -> Self {
        Self::Text {
            value: value.into(),
        }
    }

    pub fn client_ref(id: impl Into<String>, props: serde_json::Value) -> Self {
        Self::ClientReference {
            id: id.into(),
            props,
        }
    }

    pub fn suspense(id: impl Into<String>, fallback: RscNode, children: Vec<RscNode>) -> Self {
        Self::Suspense {
            id: id.into(),
            fallback: Box::new(fallback),
            children,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsc_payload_creation() {
        let mut payload = RscPayload::new();

        payload.add_node(RscNode::element(
            "div",
            serde_json::json!({"class": "container"}),
            vec![RscNode::text("Hello World")],
        ));

        payload.add_client_reference(
            "counter".to_string(),
            "./Counter.js".to_string(),
            "Counter".to_string(),
        );

        assert_eq!(payload.nodes.len(), 1);
        assert_eq!(payload.client_references.len(), 1);
    }

    #[test]
    fn test_wire_format() {
        let mut payload = RscPayload::new();
        payload.add_node(RscNode::text("Hello"));

        let wire = payload.to_wire_format();
        assert!(wire.contains("0:"));
        assert!(wire.contains("Hello"));
    }

    #[test]
    fn test_client_reference_node() {
        let node = RscNode::client_ref("counter", serde_json::json!({"initial": 5}));

        if let RscNode::ClientReference { id, props } = node {
            assert_eq!(id, "counter");
            assert_eq!(props["initial"], 5);
        } else {
            panic!("Expected ClientReference");
        }
    }

    #[test]
    fn test_suspense_node() {
        let node = RscNode::suspense(
            "async-data",
            RscNode::text("Loading..."),
            vec![RscNode::text("Loaded content")],
        );

        if let RscNode::Suspense {
            id,
            fallback,
            children,
        } = node
        {
            assert_eq!(id, "async-data");
            assert!(matches!(*fallback, RscNode::Text { .. }));
            assert_eq!(children.len(), 1);
        } else {
            panic!("Expected Suspense");
        }
    }

    #[test]
    fn test_json_serialization() {
        let mut payload = RscPayload::new();
        payload.add_node(RscNode::element("div", serde_json::json!({}), vec![]));

        let json = payload.to_json();
        assert!(json["nodes"].is_array());
    }
}
