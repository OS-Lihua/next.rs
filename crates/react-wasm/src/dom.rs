use react_rs_elements::attributes::AttributeValue;
use react_rs_elements::node::Node;
use react_rs_elements::Element;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Document;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static EVENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

type EventCallback = Rc<dyn Fn(WasmEvent)>;

thread_local! {
    static EVENT_REGISTRY: RefCell<HashMap<usize, EventCallback>> = RefCell::new(HashMap::new());
    static DELEGATED_TYPES: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

pub struct WasmEvent {
    inner: web_sys::Event,
}

impl WasmEvent {
    pub fn new(event: web_sys::Event) -> Self {
        Self { inner: event }
    }

    pub fn inner(&self) -> &web_sys::Event {
        &self.inner
    }

    pub fn event_type(&self) -> String {
        self.inner.type_()
    }

    pub fn prevent_default(&self) {
        self.inner.prevent_default();
    }

    pub fn stop_propagation(&self) {
        self.inner.stop_propagation();
    }

    pub fn target_value(&self) -> Option<String> {
        self.inner.target().and_then(|t| {
            t.dyn_ref::<web_sys::HtmlInputElement>()
                .map(|e| e.value())
                .or_else(|| {
                    t.dyn_ref::<web_sys::HtmlTextAreaElement>()
                        .map(|e| e.value())
                })
                .or_else(|| t.dyn_ref::<web_sys::HtmlSelectElement>().map(|e| e.value()))
        })
    }

    pub fn target_checked(&self) -> Option<bool> {
        self.inner.target().and_then(|t| {
            t.dyn_ref::<web_sys::HtmlInputElement>()
                .map(|e| e.checked())
        })
    }
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
            use react_rs_core::effect::create_effect;

            let initial_value = reactive.get();
            let text_node = document.create_text_node(&initial_value);
            let text_node_clone: web_sys::Text = text_node.clone();
            let text_node_rc = Rc::new(text_node_clone);
            let reactive = reactive.clone();

            create_effect(move || {
                let value = reactive.get();
                text_node_rc.set_text_content(Some(&value));
            });

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
    use react_rs_core::effect::create_effect;

    let el = document.create_element(element.tag())?;

    for attr in element.attributes() {
        match &attr.value {
            AttributeValue::String(s) => {
                el.set_attribute(&attr.name, s)?;
            }
            AttributeValue::Bool(b) => {
                if *b {
                    el.set_attribute(&attr.name, "")?;
                }
            }
            AttributeValue::ReactiveString(reactive) => {
                let initial_value = reactive.get();
                el.set_attribute(&attr.name, &initial_value)?;

                let el_rc = Rc::new(el.clone());
                let name_rc = Rc::new(attr.name.clone());
                let reactive = reactive.clone();

                create_effect(move || {
                    let value = reactive.get();
                    let _ = el_rc.set_attribute(&name_rc, &value);
                });
            }
            AttributeValue::ReactiveBool(reactive) => {
                if reactive.get() {
                    el.set_attribute(&attr.name, "")?;
                }

                let el_rc = Rc::new(el.clone());
                let name_rc = Rc::new(attr.name.clone());
                let reactive = reactive.clone();

                create_effect(move || {
                    if reactive.get() {
                        let _ = el_rc.set_attribute(&name_rc, "");
                    } else {
                        let _ = el_rc.remove_attribute(&name_rc);
                    }
                });
            }
        }
    }

    for child in element.get_children() {
        let child_node = render_node(document, child)?;
        el.append_child(&child_node)?;
    }

    for handler in element.event_handlers() {
        let event_type = handler.event_type().to_string();
        let event_id = next_event_id();

        let callback = handler.take_handler_rc();

        register_event_callback(
            event_id,
            Rc::new(move |wasm_event: WasmEvent| {
                let mut react_event =
                    react_rs_elements::events::Event::new(wasm_event.inner().type_());
                if let Some(val) = wasm_event.target_value() {
                    react_event = react_event.with_target_value(val);
                }
                if let Some(checked) = wasm_event.target_checked() {
                    react_event = react_event.with_checked(checked);
                }
                callback(react_event);
            }),
        );

        el.set_attribute("data-eid", &event_id.to_string())?;
        ensure_delegated_listener(document, &event_type)?;
    }

    Ok(el.into())
}

pub fn next_event_id() -> usize {
    EVENT_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

pub fn register_event_callback(event_id: usize, callback: EventCallback) {
    EVENT_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(event_id, callback);
    });
}

pub fn ensure_delegated_listener(document: &Document, event_type: &str) -> Result<(), JsValue> {
    let already_registered = DELEGATED_TYPES.with(|types| {
        let mut types = types.borrow_mut();
        if types.contains(event_type) {
            true
        } else {
            types.insert(event_type.to_string());
            false
        }
    });

    if already_registered {
        return Ok(());
    }

    let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
        let mut target = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok());

        while let Some(el) = target {
            if let Some(eid_str) = el.get_attribute("data-eid") {
                if let Ok(eid) = eid_str.parse::<usize>() {
                    let callback =
                        EVENT_REGISTRY.with(|registry| registry.borrow().get(&eid).cloned());
                    if let Some(cb) = callback {
                        cb(WasmEvent::new(e));
                        return;
                    }
                }
            }
            target = el.parent_element();
        }
    }) as Box<dyn FnMut(web_sys::Event)>);

    document.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
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

pub fn register_event_handler<F: Fn(WasmEvent) + 'static>(event_id: usize, handler: F) {
    EVENT_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(event_id, Rc::new(handler));
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
