use crate::reactive::ReactiveValue;
use crate::Element;

pub enum Node {
    Element(Element),
    Text(String),
    ReactiveText(ReactiveValue<String>),
    Fragment(Vec<Node>),
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
