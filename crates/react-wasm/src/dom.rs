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

pub fn render_node_pub(document: &Document, node: &Node) -> Result<web_sys::Node, JsValue> {
    render_node(document, node)
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
        Node::Conditional(condition, then_node, else_node) => {
            use react_rs_core::effect::create_effect;

            let then_dom = render_node(document, then_node)?;
            let then_el = then_dom.dyn_ref::<web_sys::Element>().cloned();

            let else_dom = else_node
                .as_ref()
                .map(|en| render_node(document, en))
                .transpose()?;
            let else_el = else_dom
                .as_ref()
                .and_then(|n| n.dyn_ref::<web_sys::Element>().cloned());

            let container = document.create_element("span")?;
            container.set_attribute("data-cond", "")?;
            container.set_attribute("style", "display:contents")?;
            container.append_child(&then_dom)?;
            if let Some(ref ed) = else_dom {
                container.append_child(ed)?;
            }

            let show = condition.get();
            if let Some(ref el) = then_el {
                let _ = el.set_attribute("style", if show { "" } else { "display:none" });
            }
            if let Some(ref el) = else_el {
                let _ = el.set_attribute("style", if show { "display:none" } else { "" });
            }

            let condition = condition.clone();
            create_effect(move || {
                let visible = condition.get();
                if let Some(ref el) = then_el {
                    let _ = el.set_attribute("style", if visible { "" } else { "display:none" });
                }
                if let Some(ref el) = else_el {
                    let _ = el.set_attribute("style", if visible { "display:none" } else { "" });
                }
            });

            Ok(container.into())
        }
        Node::ReactiveList(list_fn) => {
            use react_rs_core::effect::create_effect;

            let container = document.create_element("span")?;
            container.set_attribute("data-list", "")?;
            container.set_attribute("style", "display:contents")?;

            for child_node in list_fn() {
                let dom_child = render_node(document, &child_node)?;
                container.append_child(&dom_child)?;
            }

            let container_rc = Rc::new(container.clone());
            let list_fn = list_fn.clone();

            create_effect(move || {
                container_rc.set_inner_html("");
                let doc = get_document();
                for child_node in list_fn() {
                    if let Ok(dom_child) = render_node(&doc, &child_node) {
                        let _ = container_rc.append_child(&dom_child);
                    }
                }
            });

            Ok(container.into())
        }
        Node::Head(_) => {
            let placeholder = document.create_text_node("");
            Ok(placeholder.into())
        }
        Node::Suspense(sus) => {
            use react_rs_core::effect::create_effect;

            let fallback_dom = render_node(document, &sus.fallback)?;
            let children_dom = render_node(document, &sus.children)?;

            let fallback_el = fallback_dom.dyn_ref::<web_sys::Element>().cloned();
            let children_el = children_dom.dyn_ref::<web_sys::Element>().cloned();

            let container = document.create_element("span")?;
            container.set_attribute("data-suspense", "")?;
            container.set_attribute("style", "display:contents")?;
            container.append_child(&fallback_dom)?;
            container.append_child(&children_dom)?;

            let loading = (sus.loading_signal)();
            if let Some(ref el) = fallback_el {
                let _ = el.set_attribute("style", if loading { "" } else { "display:none" });
            }
            if let Some(ref el) = children_el {
                let _ = el.set_attribute("style", if loading { "display:none" } else { "" });
            }

            let loading_signal = sus.loading_signal.clone();
            create_effect(move || {
                let is_loading = loading_signal();
                if let Some(ref el) = fallback_el {
                    let _ = el.set_attribute("style", if is_loading { "" } else { "display:none" });
                }
                if let Some(ref el) = children_el {
                    let _ = el.set_attribute("style", if is_loading { "display:none" } else { "" });
                }
            });

            Ok(container.into())
        }
        Node::ErrorBoundary(eb) => {
            use react_rs_core::effect::create_effect;

            let children_dom = render_node(document, &eb.children)?;
            let children_el = children_dom.dyn_ref::<web_sys::Element>().cloned();

            let container = document.create_element("span")?;
            container.set_attribute("data-error-boundary", "")?;
            container.set_attribute("style", "display:contents")?;
            container.append_child(&children_dom)?;

            let container_rc = Rc::new(container.clone());
            let error_signal = eb.error_signal.clone();
            let error_fallback = eb.error_fallback.clone();

            create_effect(move || {
                if let Some(error) = error_signal() {
                    if let Some(ref el) = children_el {
                        let _ = el.set_attribute("style", "display:none");
                    }
                    while container_rc.child_nodes().length() > 1 {
                        if let Some(last) = container_rc.last_child() {
                            let _ = container_rc.remove_child(&last);
                        }
                    }
                    let error_node = error_fallback(error);
                    let doc = get_document();
                    if let Ok(error_dom) = render_node_pub(&doc, &error_node) {
                        let _ = container_rc.append_child(&error_dom);
                    }
                } else if let Some(ref el) = children_el {
                    let _ = el.set_attribute("style", "");
                }
            });

            Ok(container.into())
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
