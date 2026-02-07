use react_rs_elements::node::Node;
use react_rs_elements::Element;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::events::attach_event_handlers;

pub fn hydrate(node: Node, root: &HtmlElement) {
    hydrate_node(&node, root.first_child().as_ref());
}

fn hydrate_node(node: &Node, dom_node: Option<&web_sys::Node>) {
    let Some(dom) = dom_node else {
        return;
    };

    match node {
        Node::Element(element) => hydrate_element(element, dom),
        Node::Text(_) | Node::ReactiveText(_) => {}
        Node::Fragment(children) => {
            let mut current = dom.clone();
            for child in children {
                hydrate_node(child, Some(&current));
                if let Some(next) = current.next_sibling() {
                    current = next;
                }
            }
        }
        Node::Conditional(_, _, _)
        | Node::ReactiveList(_)
        | Node::Head(_)
        | Node::Suspense(_)
        | Node::ErrorBoundary(_) => {}
    }
}

fn hydrate_element(element: &Element, dom_node: &web_sys::Node) {
    if let Some(el) = dom_node.dyn_ref::<web_sys::Element>() {
        attach_event_handlers(el, element);

        setup_reactive_attributes(el, element);

        let mut dom_child = dom_node.first_child();
        for child in element.get_children() {
            hydrate_node(child, dom_child.as_ref());
            dom_child = dom_child.and_then(|c| c.next_sibling());
        }
    }
}

fn setup_reactive_attributes(el: &web_sys::Element, element: &Element) {
    use react_rs_core::effect::create_effect;
    use react_rs_elements::attributes::AttributeValue;
    use std::rc::Rc;

    for attr in element.attributes() {
        match &attr.value {
            AttributeValue::ReactiveString(reactive) => {
                let el = Rc::new(el.clone());
                let name = Rc::new(attr.name.clone());
                let reactive = Rc::new(reactive.clone());

                let el_clone = el.clone();
                let name_clone = name.clone();
                let reactive_clone = reactive.clone();

                create_effect(move || {
                    let value = reactive_clone.get();
                    el_clone
                        .set_attribute(&name_clone, &value)
                        .expect("failed to set attribute");
                });
            }
            AttributeValue::ReactiveBool(reactive) => {
                let el = Rc::new(el.clone());
                let name = Rc::new(attr.name.clone());
                let reactive = Rc::new(reactive.clone());

                let el_clone = el.clone();
                let name_clone = name.clone();
                let reactive_clone = reactive.clone();

                create_effect(move || {
                    if reactive_clone.get() {
                        el_clone
                            .set_attribute(&name_clone, "")
                            .expect("failed to set attribute");
                    } else {
                        let _ = el_clone.remove_attribute(&name_clone);
                    }
                });
            }
            _ => {}
        }
    }
}
