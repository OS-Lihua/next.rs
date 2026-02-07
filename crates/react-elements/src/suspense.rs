use crate::node::{IntoNode, Node};
use react_rs_core::resource::{Resource, ResourceState};
use std::rc::Rc;

pub struct SuspenseData {
    pub fallback: Box<Node>,
    pub children: Box<Node>,
    pub loading_signal: Rc<dyn Fn() -> bool>,
}

pub struct ErrorBoundaryData {
    pub error_fallback: Rc<dyn Fn(String) -> Node>,
    pub children: Box<Node>,
    pub error_signal: Rc<dyn Fn() -> Option<String>>,
}

pub fn suspense<T: Clone + 'static>(
    resource: &Resource<T>,
    fallback: impl IntoNode,
    children: impl IntoNode,
) -> Node {
    let state = resource.state();
    Node::Suspense(SuspenseData {
        fallback: Box::new(fallback.into_node()),
        children: Box::new(children.into_node()),
        loading_signal: Rc::new(move || state.get().is_loading()),
    })
}

pub fn error_boundary<T: Clone + 'static>(
    resource: &Resource<T>,
    error_fallback: impl Fn(String) -> Node + 'static,
    children: impl IntoNode,
) -> Node {
    let state = resource.state();
    Node::ErrorBoundary(ErrorBoundaryData {
        error_fallback: Rc::new(error_fallback),
        children: Box::new(children.into_node()),
        error_signal: Rc::new(move || match state.get() {
            ResourceState::Error(e) => Some(e),
            _ => None,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html;
    use react_rs_core::resource::create_resource;

    #[test]
    fn test_suspense_creates_node() {
        let resource = create_resource::<String>();
        let node = suspense(
            &resource,
            html::p().text("Loading..."),
            html::div().text("Content"),
        );
        assert!(matches!(node, Node::Suspense(_)));
    }

    #[test]
    fn test_error_boundary_creates_node() {
        let resource = create_resource::<String>();
        let node = error_boundary(
            &resource,
            |err| html::p().text(format!("Error: {}", err)).into_node(),
            html::div().text("Content"),
        );
        assert!(matches!(node, Node::ErrorBoundary(_)));
    }
}
