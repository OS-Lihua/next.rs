use react_rs_elements::attributes::AttributeValue;
use react_rs_elements::node::Node;
use react_rs_elements::Element;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlElement};

use crate::events::attach_event_handlers;

thread_local! {
    static CLOSURES: RefCell<Vec<Closure<dyn FnMut(web_sys::Event)>>> = RefCell::new(Vec::new());
}

fn document() -> Document {
    web_sys::window()
        .expect("no window")
        .document()
        .expect("no document")
}

pub fn mount_to_body(node: Node) {
    let body = document().body().expect("no body");
    mount(node, &body);
}

pub fn mount(node: Node, parent: &HtmlElement) {
    let dom_node = create_dom_node(&node);
    parent
        .append_child(&dom_node)
        .expect("failed to append child");
}

fn create_dom_node(node: &Node) -> web_sys::Node {
    match node {
        Node::Element(element) => create_element_node(element),
        Node::Text(text) => create_text_node(text),
        Node::ReactiveText(reactive) => create_reactive_text_node(reactive),
        Node::Fragment(children) => create_fragment_node(children),
        Node::Conditional(_, _, _)
        | Node::ReactiveList(_)
        | Node::Head(_)
        | Node::Suspense(_)
        | Node::ErrorBoundary(_) => create_text_node(""),
    }
}

fn create_element_node(element: &Element) -> web_sys::Node {
    let doc = document();
    let el = doc
        .create_element(element.tag())
        .expect("failed to create element");

    for attr in element.attributes() {
        apply_attribute(&el, &attr.name, &attr.value);
    }

    for child in element.get_children() {
        let child_node = create_dom_node(child);
        el.append_child(&child_node)
            .expect("failed to append child");
    }

    attach_event_handlers(&el, element);

    el.into()
}

fn apply_attribute(el: &web_sys::Element, name: &str, value: &AttributeValue) {
    match value {
        AttributeValue::String(s) => {
            el.set_attribute(name, s).expect("failed to set attribute");
        }
        AttributeValue::Bool(b) => {
            if *b {
                el.set_attribute(name, "").expect("failed to set attribute");
            }
        }
        AttributeValue::ReactiveString(reactive) => {
            let current = reactive.get();
            el.set_attribute(name, &current)
                .expect("failed to set attribute");

            setup_reactive_attribute(el.clone(), name.to_string(), reactive.clone());
        }
        AttributeValue::ReactiveBool(reactive) => {
            if reactive.get() {
                el.set_attribute(name, "").expect("failed to set attribute");
            }

            setup_reactive_bool_attribute(el.clone(), name.to_string(), reactive.clone());
        }
    }
}

fn setup_reactive_attribute(
    el: web_sys::Element,
    name: String,
    reactive: react_rs_elements::reactive::ReactiveValue<String>,
) {
    use react_rs_core::effect::create_effect;

    let el = Rc::new(el);
    let name = Rc::new(name);
    let reactive = Rc::new(reactive);

    let el_clone = el.clone();
    let name_clone = name.clone();
    let reactive_clone = reactive.clone();

    create_effect(move || {
        let value = reactive_clone.get();
        el_clone
            .set_attribute(&name_clone, &value)
            .expect("failed to update attribute");
    });
}

fn setup_reactive_bool_attribute(
    el: web_sys::Element,
    name: String,
    reactive: react_rs_elements::reactive::ReactiveValue<bool>,
) {
    use react_rs_core::effect::create_effect;

    let el = Rc::new(el);
    let name = Rc::new(name);
    let reactive = Rc::new(reactive);

    let el_clone = el.clone();
    let name_clone = name.clone();
    let reactive_clone = reactive.clone();

    create_effect(move || {
        if reactive_clone.get() {
            el_clone
                .set_attribute(&name_clone, "")
                .expect("failed to set attribute");
        } else {
            el_clone
                .remove_attribute(&name_clone)
                .expect("failed to remove attribute");
        }
    });
}

fn create_text_node(text: &str) -> web_sys::Node {
    document().create_text_node(text).into()
}

fn create_reactive_text_node(
    reactive: &react_rs_elements::reactive::ReactiveValue<String>,
) -> web_sys::Node {
    use react_rs_core::effect::create_effect;

    let text_node = document().create_text_node(&reactive.get());
    let text_node_ref = Rc::new(text_node.clone());
    let reactive = reactive.clone();

    create_effect(move || {
        let value = reactive.get();
        text_node_ref.set_text_content(Some(&value));
    });

    text_node.into()
}

fn create_fragment_node(children: &[Node]) -> web_sys::Node {
    let doc = document();
    let fragment = doc.create_document_fragment();

    for child in children {
        let child_node = create_dom_node(child);
        fragment
            .append_child(&child_node)
            .expect("failed to append to fragment");
    }

    fragment.into()
}
