use react_rs_elements::node::Node;
use react_rs_elements::Element;
use wasm_bindgen::prelude::*;
use web_sys::Document;

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static EVENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static EVENT_REGISTRY: RefCell<HashMap<usize, Box<dyn Fn()>>> = RefCell::new(HashMap::new());
}

pub struct DomNode {
    inner: web_sys::Node,
}

impl DomNode {
    pub fn new(node: web_sys::Node) -> Self {
        Self { inner: node }
    }

    pub fn inner(&self) -> &web_sys::Node {
        &self.inner
    }
}

fn get_document() -> Document {
    web_sys::window()
        .expect("no window")
        .document()
        .expect("no document")
}

pub fn render_to_dom(node: &Node) -> Result<web_sys::Node, JsValue> {
    let document = get_document();
    render_node(&document, node)
}

fn render_node(document: &Document, node: &Node) -> Result<web_sys::Node, JsValue> {
    match node {
        Node::Element(element) => render_element(document, element),
        Node::Text(text) => {
            let text_node = document.create_text_node(text);
            Ok(text_node.into())
        }
        Node::ReactiveText(reactive) => {
            let value = reactive.get();
            let text_node = document.create_text_node(&value);
            Ok(text_node.into())
        }
        Node::Fragment(children) => {
            let fragment = document.create_document_fragment();
            for child in children {
                let child_node = render_node(document, child)?;
                fragment.append_child(&child_node)?;
            }
            Ok(fragment.into())
        }
    }
}

fn render_element(document: &Document, element: &Element) -> Result<web_sys::Node, JsValue> {
    let el = document.create_element(element.tag())?;

    for attr in element.attributes() {
        let value = attr.to_static_value();
        el.set_attribute(&attr.name, &value)?;
    }

    for child in element.get_children() {
        let child_node = render_node(document, child)?;
        el.append_child(&child_node)?;
    }

    for handler in element.event_handlers() {
        let event_type = handler.event_type().to_string();
        let event_id = EVENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        el.set_attribute("data-event-id", &event_id.to_string())?;
        el.set_attribute(&format!("data-event-{}", event_type), "true")?;

        let closure = Closure::wrap(Box::new(move || {
            EVENT_REGISTRY.with(|registry| {
                if let Some(callback) = registry.borrow().get(&event_id) {
                    callback();
                }
            });
        }) as Box<dyn FnMut()>);

        el.add_event_listener_with_callback(&event_type, closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    Ok(el.into())
}

pub fn mount(node: &Node, container_id: &str) -> Result<(), JsValue> {
    let document = get_document();
    let container = document
        .get_element_by_id(container_id)
        .ok_or_else(|| JsValue::from_str(&format!("Container '{}' not found", container_id)))?;

    container.set_inner_html("");

    let dom_node = render_to_dom(node)?;
    container.append_child(&dom_node)?;

    Ok(())
}

pub fn register_event_handler<F: Fn() + 'static>(event_id: usize, handler: F) {
    EVENT_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(event_id, Box::new(handler));
    });
}

pub fn unregister_event_handler(event_id: usize) {
    EVENT_REGISTRY.with(|registry| {
        registry.borrow_mut().remove(&event_id);
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_event_registry_accessible() {
        let _ = super::EVENT_REGISTRY.try_with(|_| ());
    }
}
