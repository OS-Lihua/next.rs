use next_rs_rsc::{RscNode, RscPayload};
use react_rs_elements::html::*;
use react_rs_elements::node::Node;
use react_rs_elements::Element;
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

pub struct ClientComponentRegistry {
    components: HashMap<String, Box<dyn Fn(Value) -> Element>>,
}

impl ClientComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn register<F>(&mut self, id: impl Into<String>, factory: F)
    where
        F: Fn(Value) -> Element + 'static,
    {
        self.components.insert(id.into(), Box::new(factory));
    }

    pub fn get(&self, id: &str) -> Option<&dyn Fn(Value) -> Element> {
        self.components.get(id).map(|b| b.as_ref())
    }

    pub fn has(&self, id: &str) -> bool {
        self.components.contains_key(id)
    }

    pub fn component_ids(&self) -> Vec<&String> {
        self.components.keys().collect()
    }
}

impl Default for ClientComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RscRuntime {
    registry: ClientComponentRegistry,
}

impl RscRuntime {
    pub fn new() -> Self {
        Self {
            registry: ClientComponentRegistry::new(),
        }
    }

    pub fn with_registry(registry: ClientComponentRegistry) -> Self {
        Self { registry }
    }

    pub fn register_component<F>(&mut self, id: impl Into<String>, factory: F)
    where
        F: Fn(Value) -> Element + 'static,
    {
        self.registry.register(id, factory);
    }

    pub fn parse_payload(&self, wire_format: &str) -> Result<RscPayload, String> {
        let mut payload = RscPayload::new();

        for line in wire_format.lines() {
            if line.starts_with("M:") {
                continue;
            }

            if let Some(colon_pos) = line.find(':') {
                let json_str = &line[colon_pos + 1..];
                if let Ok(node) = serde_json::from_str::<RscNode>(json_str) {
                    payload.add_node(node);
                }
            }
        }

        Ok(payload)
    }

    pub fn render_payload(&self, payload: &RscPayload) -> Node {
        let elements: Vec<Node> = payload
            .nodes
            .iter()
            .map(|node| self.render_rsc_node(node))
            .collect();

        if elements.len() == 1 {
            elements.into_iter().next().unwrap()
        } else {
            Node::Fragment(elements)
        }
    }

    fn render_rsc_node(&self, rsc_node: &RscNode) -> Node {
        match rsc_node {
            RscNode::Element {
                tag,
                props,
                children,
            } => {
                let mut element = create_element_by_tag(tag);

                if let Some(obj) = props.as_object() {
                    for (key, value) in obj {
                        if let Some(v) = value.as_str() {
                            element = element.attr(key, v);
                        }
                    }
                }

                for child in children {
                    let child_node = self.render_rsc_node(child);
                    element = element.child(child_node);
                }

                Node::Element(element)
            }
            RscNode::Text { value } => Node::Text(value.clone()),
            RscNode::ClientReference { id, props } => {
                if let Some(factory) = self.registry.get(id) {
                    Node::Element(factory(props.clone()))
                } else {
                    Node::Element(
                        div()
                            .attr("data-client-placeholder", id)
                            .text(format!("Loading {}...", id)),
                    )
                }
            }
            RscNode::Suspense {
                id,
                fallback,
                children,
            } => {
                let fallback_node = self.render_rsc_node(fallback);
                let children_nodes: Vec<Node> =
                    children.iter().map(|c| self.render_rsc_node(c)).collect();

                Node::Element(
                    div()
                        .attr("data-suspense-id", id)
                        .child(fallback_node)
                        .children(children_nodes),
                )
            }
        }
    }

    pub fn registry(&self) -> &ClientComponentRegistry {
        &self.registry
    }
}

impl Default for RscRuntime {
    fn default() -> Self {
        Self::new()
    }
}

fn create_element_by_tag(tag: &str) -> Element {
    match tag {
        "div" => div(),
        "span" => span(),
        "p" => p(),
        "h1" => h1(),
        "h2" => h2(),
        "h3" => h3(),
        "h4" => h4(),
        "h5" => h5(),
        "h6" => h6(),
        "a" => a(),
        "button" => button(),
        "input" => input(),
        "form" => form(),
        "ul" => ul(),
        "ol" => ol(),
        "li" => li(),
        "nav" => nav(),
        "header" => header(),
        "footer" => footer(),
        "main" => main_el(),
        "section" => section(),
        "article" => article(),
        "aside" => aside(),
        "img" => img(),
        "table" => table(),
        "tr" => tr(),
        "td" => td(),
        "th" => th(),
        "thead" => thead(),
        "tbody" => tbody(),
        "pre" => pre(),
        "code" => code(),
        "blockquote" => div().attr("data-tag", "blockquote"),
        "strong" => strong(),
        "em" => em(),
        "br" => br(),
        "hr" => hr(),
        "label" => label(),
        "select" => select(),
        "option" => option(),
        "textarea" => textarea(),
        "fragment" => div().attr("data-fragment", "true"),
        _ => div().attr("data-unknown-tag", tag),
    }
}

#[wasm_bindgen]
pub async fn fetch_rsc_payload(url: &str) -> Result<JsValue, JsValue> {
    let window = web_sys::window().ok_or("no window")?;

    let opts = web_sys::RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(web_sys::RequestMode::Cors);

    let request = web_sys::Request::new_with_str_and_init(url, &opts)?;
    request.headers().set("Accept", "text/x-component")?;

    let resp_value =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: web_sys::Response = resp_value.dyn_into()?;

    let text = wasm_bindgen_futures::JsFuture::from(resp.text()?).await?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_registry() {
        let mut registry = ClientComponentRegistry::new();
        registry.register("counter", |_props| button().text("Count: 0"));

        assert!(registry.has("counter"));
        assert!(!registry.has("unknown"));
    }

    #[test]
    fn test_rsc_runtime_creation() {
        let runtime = RscRuntime::new();
        assert!(runtime.registry().component_ids().is_empty());
    }

    #[test]
    fn test_register_and_render_client_component() {
        let mut runtime = RscRuntime::new();
        runtime.register_component("test-button", |props| {
            let label = props
                .get("label")
                .and_then(|v| v.as_str())
                .unwrap_or("Click");
            button().text(label)
        });

        let node = RscNode::ClientReference {
            id: "test-button".to_string(),
            props: serde_json::json!({"label": "Submit"}),
        };

        let rendered = runtime.render_rsc_node(&node);
        if let Node::Element(el) = rendered {
            assert_eq!(el.tag(), "button");
        } else {
            panic!("Expected Element");
        }
    }

    #[test]
    fn test_render_element_node() {
        let runtime = RscRuntime::new();

        let node = RscNode::Element {
            tag: "div".to_string(),
            props: serde_json::json!({"class": "container"}),
            children: vec![RscNode::Text {
                value: "Hello".to_string(),
            }],
        };

        let rendered = runtime.render_rsc_node(&node);
        if let Node::Element(el) = rendered {
            assert_eq!(el.tag(), "div");
        } else {
            panic!("Expected Element");
        }
    }

    #[test]
    fn test_parse_wire_format() {
        let runtime = RscRuntime::new();

        let wire = r#"0:{"type":"text","value":"Hello"}"#;
        let payload = runtime.parse_payload(wire).unwrap();

        assert_eq!(payload.nodes.len(), 1);
    }

    #[test]
    fn test_create_element_by_tag() {
        assert_eq!(create_element_by_tag("div").tag(), "div");
        assert_eq!(create_element_by_tag("span").tag(), "span");
        assert_eq!(create_element_by_tag("button").tag(), "button");
        assert_eq!(create_element_by_tag("unknown").tag(), "div");
    }
}
