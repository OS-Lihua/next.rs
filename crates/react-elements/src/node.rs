use crate::head::Head;
use crate::reactive::ReactiveValue;
use crate::suspense::{ErrorBoundaryData, SuspenseData};
use crate::Element;
use std::rc::Rc;

pub enum Node {
    Element(Element),
    Text(String),
    ReactiveText(ReactiveValue<String>),
    Fragment(Vec<Node>),
    Conditional(ReactiveValue<bool>, Box<Node>, Option<Box<Node>>),
    ReactiveList(Rc<dyn Fn() -> Vec<Node>>),
    Head(Head),
    Suspense(SuspenseData),
    ErrorBoundary(ErrorBoundaryData),
}

pub trait IntoNode {
    fn into_node(self) -> Node;
}

impl IntoNode for Element {
    fn into_node(self) -> Node {
        Node::Element(self)
    }
}

impl IntoNode for String {
    fn into_node(self) -> Node {
        Node::Text(self)
    }
}

impl IntoNode for &str {
    fn into_node(self) -> Node {
        Node::Text(self.to_string())
    }
}

impl<T: IntoNode> IntoNode for Vec<T> {
    fn into_node(self) -> Node {
        Node::Fragment(self.into_iter().map(|n| n.into_node()).collect())
    }
}

impl IntoNode for Node {
    fn into_node(self) -> Node {
        self
    }
}

impl IntoNode for Head {
    fn into_node(self) -> Node {
        Node::Head(self)
    }
}

pub fn each<T, F>(items: react_rs_core::signal::ReadSignal<Vec<T>>, render: F) -> Node
where
    T: Clone + 'static,
    F: Fn(&T, usize) -> Node + 'static,
{
    Node::ReactiveList(Rc::new(move || {
        items.with(|list| {
            list.iter()
                .enumerate()
                .map(|(i, item)| render(item, i))
                .collect()
        })
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html;
    use react_rs_core::create_signal;

    #[test]
    fn test_conditional_creates_node() {
        let node = html::div().text("test").show_when(true);
        assert!(matches!(node, Node::Conditional(_, _, None)));
    }

    #[test]
    fn test_conditional_else_creates_node() {
        let node = html::div()
            .text("yes")
            .show_when_else(false, html::span().text("no"));
        assert!(matches!(node, Node::Conditional(_, _, Some(_))));
    }

    #[test]
    fn test_each_creates_reactive_list() {
        let (items, _) = create_signal(vec![1, 2, 3]);
        let node = each(items, |item, _| {
            html::li().text(item.to_string()).into_node()
        });
        assert!(matches!(node, Node::ReactiveList(_)));
    }
}
