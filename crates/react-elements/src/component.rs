use crate::node::Node;

pub trait Component {
    type Props;
    fn render(props: Self::Props) -> Node;
}

pub fn component<P, F>(props: P, render: F) -> Node
where
    F: FnOnce(P) -> Node,
{
    render(props)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html;
    use crate::node::IntoNode;

    struct Greeting;

    impl Component for Greeting {
        type Props = String;
        fn render(name: String) -> Node {
            html::div().text(format!("Hello, {}!", name)).into_node()
        }
    }

    #[test]
    fn test_component_trait() {
        let node = Greeting::render("World".to_string());
        match node {
            Node::Element(el) => assert_eq!(el.tag(), "div"),
            _ => panic!("Expected Element node"),
        }
    }

    #[test]
    fn test_component_helper() {
        let node = component("Rust", |name: &str| {
            html::h1().text(format!("Hello, {}!", name)).into_node()
        });
        match node {
            Node::Element(el) => assert_eq!(el.tag(), "h1"),
            _ => panic!("Expected Element node"),
        }
    }
}
