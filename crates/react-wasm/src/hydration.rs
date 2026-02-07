use react_rs_elements::attributes::AttributeValue;
use react_rs_elements::node::Node;
use react_rs_elements::Element;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element as WebElement};

use std::rc::Rc;

#[derive(Debug)]
pub enum HydrationError {
    ContainerNotFound(String),
    NodeMismatch { expected: String, found: String },
    ChildCountMismatch { expected: usize, found: usize },
    JsError(String),
}

impl From<JsValue> for HydrationError {
    fn from(value: JsValue) -> Self {
        HydrationError::JsError(format!("{:?}", value))
    }
}

pub type HydrationResult<T> = Result<T, HydrationError>;

fn get_document() -> Document {
    web_sys::window()
        .expect("no window")
        .document()
        .expect("no document")
}

pub fn hydrate(node: &Node, container_id: &str) -> HydrationResult<()> {
    let document = get_document();
    let container = document
        .get_element_by_id(container_id)
        .ok_or_else(|| HydrationError::ContainerNotFound(container_id.to_string()))?;

    let children = container.child_nodes();

    if children.length() == 0 {
        return Err(HydrationError::ChildCountMismatch {
            expected: 1,
            found: 0,
        });
    }

    if let Some(first_child) = children.get(0) {
        hydrate_node(node, &first_child)?;
    }

    Ok(())
}

fn hydrate_node(virtual_node: &Node, dom_node: &web_sys::Node) -> HydrationResult<()> {
    match virtual_node {
        Node::Element(element) => hydrate_element(element, dom_node),
        Node::Text(_) => Ok(()),
        Node::ReactiveText(reactive) => {
            use react_rs_core::effect::create_effect;

            if let Some(text_node) = dom_node.dyn_ref::<web_sys::Text>() {
                let text_node_rc = Rc::new(text_node.clone());
                let reactive = reactive.clone();

                create_effect(move || {
                    let value = reactive.get();
                    text_node_rc.set_text_content(Some(&value));
                });
            }
            Ok(())
        }
        Node::Fragment(children) => {
            for (i, child) in children.iter().enumerate() {
                if let Some(dom_child) = dom_node.child_nodes().get(i as u32) {
                    hydrate_node(child, &dom_child)?;
                }
            }
            Ok(())
        }
        Node::Conditional(condition, then_node, else_node) => {
            use react_rs_core::effect::create_effect;

            let dom_element = dom_node.dyn_ref::<web_sys::Element>().ok_or_else(|| {
                HydrationError::NodeMismatch {
                    expected: "conditional-container".to_string(),
                    found: "non-element".to_string(),
                }
            })?;

            let children = dom_element.child_nodes();
            if let Some(first) = children.get(0) {
                hydrate_node(then_node, &first)?;
            }
            if let Some(else_n) = else_node {
                if let Some(second) = children.get(1) {
                    hydrate_node(else_n, &second)?;
                }
            }

            let cond_children = dom_element.child_nodes();
            let then_el: Option<Rc<web_sys::Element>> = cond_children
                .get(0)
                .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
                .map(Rc::new);
            let else_el: Option<Rc<web_sys::Element>> = cond_children
                .get(1)
                .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
                .map(Rc::new);

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

            Ok(())
        }
        Node::ReactiveList(list_fn) => {
            use react_rs_core::effect::create_effect;

            let dom_element = dom_node.dyn_ref::<web_sys::Element>().ok_or_else(|| {
                HydrationError::NodeMismatch {
                    expected: "list-container".to_string(),
                    found: "non-element".to_string(),
                }
            })?;

            let container_rc = Rc::new(dom_element.clone());
            let list_fn = list_fn.clone();

            create_effect(move || {
                container_rc.set_inner_html("");
                let doc = get_document();
                for child_node in list_fn() {
                    if let Ok(dom_child) = crate::dom::render_node_pub(&doc, &child_node) {
                        let _ = container_rc.append_child(&dom_child);
                    }
                }
            });

            Ok(())
        }
        Node::Head(_) | Node::Suspense(_) | Node::ErrorBoundary(_) => Ok(()),
    }
}

fn hydrate_element(element: &Element, dom_node: &web_sys::Node) -> HydrationResult<()> {
    use react_rs_core::effect::create_effect;

    let dom_element: &WebElement =
        dom_node
            .dyn_ref()
            .ok_or_else(|| HydrationError::NodeMismatch {
                expected: element.tag().to_string(),
                found: "non-element".to_string(),
            })?;

    let dom_tag = dom_element.tag_name().to_lowercase();
    if dom_tag != element.tag() {
        return Err(HydrationError::NodeMismatch {
            expected: element.tag().to_string(),
            found: dom_tag,
        });
    }

    for attr in element.attributes() {
        match &attr.value {
            AttributeValue::ReactiveString(reactive) => {
                let el_rc = Rc::new(dom_element.clone());
                let name_rc = Rc::new(attr.name.clone());
                let reactive = reactive.clone();

                create_effect(move || {
                    let value = reactive.get();
                    let _ = el_rc.set_attribute(&name_rc, &value);
                });
            }
            AttributeValue::ReactiveBool(reactive) => {
                let el_rc = Rc::new(dom_element.clone());
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
            _ => {}
        }
    }

    for handler in element.event_handlers() {
        let event_type = handler.event_type().to_string();
        let event_id = crate::dom::next_event_id();

        let callback = handler.take_handler_rc();

        crate::dom::register_event_callback(
            event_id,
            Rc::new(move |wasm_event: crate::dom::WasmEvent| {
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

        dom_element
            .set_attribute("data-eid", &event_id.to_string())
            .map_err(HydrationError::from)?;

        let document = get_document();
        crate::dom::ensure_delegated_listener(&document, &event_type)
            .map_err(HydrationError::from)?;
    }

    let virtual_children = element.get_children();
    let dom_children = dom_node.child_nodes();

    for (i, virtual_child) in virtual_children.iter().enumerate() {
        if let Some(dom_child) = dom_children.get(i as u32) {
            hydrate_node(virtual_child, &dom_child)?;
        }
    }

    Ok(())
}

pub fn hydrate_client_components(container_id: &str) -> HydrationResult<Vec<String>> {
    let document = get_document();
    let container = document
        .get_element_by_id(container_id)
        .ok_or_else(|| HydrationError::ContainerNotFound(container_id.to_string()))?;

    let client_elements = container
        .query_selector_all("[data-client]")
        .map_err(HydrationError::from)?;

    let mut hydrated = Vec::new();

    for i in 0..client_elements.length() {
        if let Some(node) = client_elements.get(i) {
            if let Some(el) = node.dyn_ref::<WebElement>() {
                if let Some(component_id) = el.get_attribute("data-component-id") {
                    hydrated.push(component_id);
                }
            }
        }
    }

    Ok(hydrated)
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_hydration_error_from_js() {
        let js_err = JsValue::from_str("test error");
        let err = HydrationError::from(js_err);
        assert!(matches!(err, HydrationError::JsError(_)));
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn test_hydration_error_types() {
        let err = HydrationError::ContainerNotFound("test".to_string());
        assert!(matches!(err, HydrationError::ContainerNotFound(_)));

        let err = HydrationError::NodeMismatch {
            expected: "div".to_string(),
            found: "span".to_string(),
        };
        assert!(matches!(err, HydrationError::NodeMismatch { .. }));
    }
}
