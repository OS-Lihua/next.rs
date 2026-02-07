use react_rs_elements::{Element, Node};
use std::marker::PhantomData;

use crate::RscPayload;

pub struct Server;

pub struct Component<M, F>
where
    F: Fn() -> Element + 'static,
{
    id: String,
    render_fn: F,
    _marker: PhantomData<M>,
}

impl<F> Component<Server, F>
where
    F: Fn() -> Element + 'static,
{
    pub fn server(id: impl Into<String>, render_fn: F) -> Self {
        Self {
            id: id.into(),
            render_fn,
            _marker: PhantomData,
        }
    }

    pub fn render(&self) -> Element {
        (self.render_fn)()
    }

    pub fn render_to_payload(&self) -> RscPayload {
        let element = self.render();
        let node = Node::Element(element);
        crate::render_to_rsc_payload(&node)
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

pub struct ClientMarker<F>
where
    F: Fn() -> Element + 'static,
{
    id: String,
    module: String,
    render_fn: F,
}

impl<F> ClientMarker<F>
where
    F: Fn() -> Element + 'static,
{
    pub fn new(id: impl Into<String>, module: impl Into<String>, render_fn: F) -> Self {
        Self {
            id: id.into(),
            module: module.into(),
            render_fn,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn module(&self) -> &str {
        &self.module
    }

    pub fn render_fallback(&self) -> Element {
        (self.render_fn)()
    }

    pub fn to_rsc_reference(&self, props: serde_json::Value) -> crate::RscNode {
        crate::RscNode::client_ref(&self.id, props)
    }
}

pub fn server_component<F>(id: impl Into<String>, render_fn: F) -> Component<Server, F>
where
    F: Fn() -> Element + 'static,
{
    Component::server(id, render_fn)
}

pub fn client_component<F>(
    id: impl Into<String>,
    module: impl Into<String>,
    render_fn: F,
) -> ClientMarker<F>
where
    F: Fn() -> Element + 'static,
{
    ClientMarker::new(id, module, render_fn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use react_rs_elements::html::*;

    #[test]
    fn test_server_component_creation() {
        let component = server_component("article-list", || {
            div().class("articles").child(h1().text("Articles"))
        });

        assert_eq!(component.id(), "article-list");
        let element = component.render();
        assert_eq!(element.tag(), "div");
    }

    #[test]
    fn test_server_component_to_payload() {
        let component = server_component("header", || {
            header().child(nav().child(a().href("/").text("Home")))
        });

        let payload = component.render_to_payload();
        assert!(!payload.nodes.is_empty());
    }

    #[test]
    fn test_client_component_creation() {
        let component = client_component("counter", "./Counter.js", || {
            div().class("counter").text("0")
        });

        assert_eq!(component.id(), "counter");
        assert_eq!(component.module(), "./Counter.js");
    }

    #[test]
    fn test_client_component_rsc_reference() {
        let component =
            client_component("like-button", "./LikeButton.js", || button().text("Like"));

        let rsc_node = component.to_rsc_reference(serde_json::json!({"article_id": 42}));
        if let crate::RscNode::ClientReference { id, props } = rsc_node {
            assert_eq!(id, "like-button");
            assert_eq!(props["article_id"], 42);
        } else {
            panic!("Expected ClientReference");
        }
    }

    #[test]
    fn test_client_component_fallback_render() {
        let component = client_component("modal", "./Modal.js", || {
            div().class("modal").text("Loading...")
        });

        let fallback = component.render_fallback();
        assert_eq!(fallback.tag(), "div");
        assert!(fallback.has_class("modal"));
    }
}
