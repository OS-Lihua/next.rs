use react_rs_elements::{Element, Node};
use serde_json::json;

use crate::payload::{RscNode, RscPayload};

pub struct RscRenderer {
    payload: RscPayload,
    client_id_counter: usize,
}

impl RscRenderer {
    pub fn new() -> Self {
        Self {
            payload: RscPayload::new(),
            client_id_counter: 0,
        }
    }

    pub fn render_element(&mut self, element: &Element) -> RscNode {
        let tag = element.tag().to_string();

        let props = self.collect_props(element);
        let children = self.render_children(element.get_children());

        RscNode::element(tag, props, children)
    }

    pub fn render_node(&mut self, node: &Node) -> RscNode {
        match node {
            Node::Element(element) => self.render_element(element),
            Node::Text(text) => RscNode::text(text),
            Node::ReactiveText(reactive) => {
                let value = reactive.get();
                RscNode::text(value)
            }
            Node::Fragment(nodes) => {
                let children: Vec<RscNode> = nodes.iter().map(|n| self.render_node(n)).collect();
                RscNode::element("fragment", json!({}), children)
            }
        }
    }

    pub fn render_to_payload(mut self, node: &Node) -> RscPayload {
        let rsc_node = self.render_node(node);
        self.payload.add_node(rsc_node);
        self.payload
    }

    pub fn register_client_component(
        &mut self,
        module: impl Into<String>,
        export: impl Into<String>,
        props: serde_json::Value,
    ) -> RscNode {
        let id = format!("client_{}", self.client_id_counter);
        self.client_id_counter += 1;

        let module_str = module.into();
        let export_str = export.into();

        self.payload
            .add_client_reference(id.clone(), module_str, export_str);

        RscNode::client_ref(id, props)
    }

    fn collect_props(&self, element: &Element) -> serde_json::Value {
        let mut props = serde_json::Map::new();

        for attr in element.attributes() {
            let value = attr.to_static_value();
            props.insert(attr.name.to_string(), json!(value));
        }

        json!(props)
    }

    fn render_children(&mut self, children: &[Node]) -> Vec<RscNode> {
        children
            .iter()
            .map(|child| self.render_node(child))
            .collect()
    }
}

impl Default for RscRenderer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_to_rsc_payload(node: &Node) -> RscPayload {
    RscRenderer::new().render_to_payload(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use react_rs_elements::html::*;

    #[test]
    fn test_render_simple_element() {
        let element = div().class("container").text("Hello World");
        let node = Node::Element(element);

        let payload = render_to_rsc_payload(&node);
        let wire = payload.to_wire_format();

        assert!(wire.contains("div"));
        assert!(wire.contains("Hello World"));
    }

    #[test]
    fn test_render_nested_elements() {
        let element = div()
            .class("app")
            .child(h1().text("Title"))
            .child(p().text("Content"));
        let node = Node::Element(element);

        let payload = render_to_rsc_payload(&node);
        assert_eq!(payload.nodes.len(), 1);

        if let RscNode::Element { children, .. } = &payload.nodes[0] {
            assert_eq!(children.len(), 2);
        } else {
            panic!("Expected Element node");
        }
    }

    #[test]
    fn test_register_client_component() {
        let mut renderer = RscRenderer::new();

        let client_node =
            renderer.register_client_component("./Counter.js", "Counter", json!({"initial": 0}));

        if let RscNode::ClientReference { id, props } = client_node {
            assert_eq!(id, "client_0");
            assert_eq!(props["initial"], 0);
        } else {
            panic!("Expected ClientReference");
        }

        assert_eq!(renderer.payload.client_references.len(), 1);
    }

    #[test]
    fn test_fragment_rendering() {
        let fragment = Node::Fragment(vec![
            Node::Text("First".to_string()),
            Node::Text("Second".to_string()),
        ]);

        let payload = render_to_rsc_payload(&fragment);
        assert_eq!(payload.nodes.len(), 1);

        if let RscNode::Element { tag, children, .. } = &payload.nodes[0] {
            assert_eq!(tag, "fragment");
            assert_eq!(children.len(), 2);
        }
    }
}
