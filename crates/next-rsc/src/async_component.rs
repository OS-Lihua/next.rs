use react_rs_elements::{Element, Node};
use std::future::Future;
use std::pin::Pin;

use crate::RscPayload;

pub type AsyncRenderFn =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = Element> + Send>> + Send + Sync>;

pub struct AsyncServerComponent {
    id: String,
    render_fn: AsyncRenderFn,
}

impl AsyncServerComponent {
    pub fn new<F, Fut>(id: impl Into<String>, render_fn: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Element> + Send + 'static,
    {
        Self {
            id: id.into(),
            render_fn: Box::new(move || Box::pin(render_fn())),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn render(&self) -> Element {
        (self.render_fn)().await
    }

    pub async fn render_to_payload(&self) -> RscPayload {
        let element = self.render().await;
        let node = Node::Element(element);
        crate::render_to_rsc_payload(&node)
    }
}

pub fn async_server_component<F, Fut>(id: impl Into<String>, render_fn: F) -> AsyncServerComponent
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Element> + Send + 'static,
{
    AsyncServerComponent::new(id, render_fn)
}

pub struct SuspenseWrapper {
    id: String,
    fallback: Element,
    content_future: Pin<Box<dyn Future<Output = Element> + Send>>,
}

impl SuspenseWrapper {
    pub fn new<F, Fut>(id: impl Into<String>, fallback: Element, content_fn: F) -> Self
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Element> + Send + 'static,
    {
        Self {
            id: id.into(),
            fallback,
            content_future: Box::pin(content_fn()),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn fallback(&self) -> &Element {
        &self.fallback
    }

    pub async fn resolve(self) -> Element {
        self.content_future.await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use react_rs_elements::html::*;

    async fn mock_fetch_data() -> Vec<String> {
        vec!["Item 1".to_string(), "Item 2".to_string()]
    }

    #[test]
    fn test_async_server_component_creation() {
        let _component = async_server_component("data-list", || async {
            let _data = mock_fetch_data().await;
            div().text("Loaded")
        });
    }

    #[tokio::test]
    async fn test_async_server_component_render() {
        let component = async_server_component("test-async", || async {
            div().class("async-content").text("Async Rendered")
        });

        let element = component.render().await;
        assert_eq!(element.tag(), "div");
        assert!(element.has_class("async-content"));
    }

    #[tokio::test]
    async fn test_async_server_component_to_payload() {
        let component =
            async_server_component("test-payload", || async { p().text("Hello from async") });

        let payload = component.render_to_payload().await;
        assert!(!payload.nodes.is_empty());
    }

    #[tokio::test]
    async fn test_suspense_wrapper() {
        let wrapper = SuspenseWrapper::new("suspense-1", div().text("Loading..."), || async {
            div().text("Content loaded")
        });

        assert_eq!(wrapper.id(), "suspense-1");
        assert_eq!(wrapper.fallback().tag(), "div");

        let content = wrapper.resolve().await;
        assert_eq!(content.tag(), "div");
    }

    #[tokio::test]
    async fn test_async_data_fetching() {
        let component = async_server_component("articles", || async {
            let data = mock_fetch_data().await;
            let mut list = ul();
            for item in data {
                list = list.child(li().text(&item));
            }
            list
        });

        let element = component.render().await;
        assert_eq!(element.tag(), "ul");
    }
}
