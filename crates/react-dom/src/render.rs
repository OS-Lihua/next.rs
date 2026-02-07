use react_rs_elements::attributes::AttributeValue;
use react_rs_elements::node::Node;
use react_rs_elements::Element;

pub struct RenderOutput {
    pub html: String,
}

pub fn render_to_string(node: &Node) -> RenderOutput {
    RenderOutput {
        html: render_node(node),
    }
}

fn render_node(node: &Node) -> String {
    match node {
        Node::Element(element) => render_element(element),
        Node::Text(text) => escape_html(text),
        Node::ReactiveText(reactive) => escape_html(&reactive.get()),
        Node::Fragment(children) => children
            .iter()
            .map(render_node)
            .collect::<Vec<_>>()
            .join(""),
    }
}

fn render_element(element: &Element) -> String {
    let tag = element.tag();
    let attrs = render_attributes(element);
    let children = element
        .get_children()
        .iter()
        .map(render_node)
        .collect::<Vec<_>>()
        .join("");

    if is_void_element(tag) {
        format!("<{}{} />", tag, attrs)
    } else {
        format!("<{}{}>{}</{}>", tag, attrs, children, tag)
    }
}

fn render_attributes(element: &Element) -> String {
    let attrs: Vec<String> = element
        .attributes()
        .iter()
        .filter_map(|attr| match &attr.value {
            AttributeValue::String(s) => Some(format!(" {}=\"{}\"", attr.name, escape_attr(s))),
            AttributeValue::Bool(b) => {
                if *b {
                    Some(format!(" {}", attr.name))
                } else {
                    None
                }
            }
            AttributeValue::ReactiveString(reactive) => Some(format!(
                " {}=\"{}\"",
                attr.name,
                escape_attr(&reactive.get())
            )),
            AttributeValue::ReactiveBool(reactive) => {
                if reactive.get() {
                    Some(format!(" {}", attr.name))
                } else {
                    None
                }
            }
        })
        .collect();

    attrs.join("")
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use react_rs_elements::html::*;
    use react_rs_elements::node::IntoNode;

    #[test]
    fn test_render_simple_element() {
        let element = div().class("container").text("Hello");
        let output = render_to_string(&element.into_node());
        assert_eq!(output.html, "<div class=\"container\">Hello</div>");
    }

    #[test]
    fn test_render_nested_elements() {
        let element = div()
            .class("app")
            .child(h1().text("Title"))
            .child(p().text("Content"));
        let output = render_to_string(&element.into_node());
        assert_eq!(
            output.html,
            "<div class=\"app\"><h1>Title</h1><p>Content</p></div>"
        );
    }

    #[test]
    fn test_render_void_element() {
        let element = input().type_("text").placeholder("Enter name");
        let output = render_to_string(&element.into_node());
        assert_eq!(
            output.html,
            "<input type=\"text\" placeholder=\"Enter name\" />"
        );
    }

    #[test]
    fn test_render_escapes_html() {
        let element = p().text("<script>alert('xss')</script>");
        let output = render_to_string(&element.into_node());
        assert_eq!(
            output.html,
            "<p>&lt;script&gt;alert('xss')&lt;/script&gt;</p>"
        );
    }

    #[test]
    fn test_render_boolean_attribute() {
        let element = input().disabled(true);
        let output = render_to_string(&element.into_node());
        assert!(output.html.contains(" disabled"));

        let element_enabled = input().disabled(false);
        let output_enabled = render_to_string(&element_enabled.into_node());
        assert!(!output_enabled.html.contains("disabled"));
    }

    #[test]
    fn test_render_fragment() {
        let fragment = vec![span().text("A"), span().text("B")];
        let output = render_to_string(&fragment.into_node());
        assert_eq!(output.html, "<span>A</span><span>B</span>");
    }

    #[test]
    fn test_render_complex_structure() {
        let view = html().child(head().child(title().text("My App"))).child(
            body().child(
                div()
                    .id("root")
                    .child(header().child(nav().child(a().href("/").text("Home"))))
                    .child(main_el().child(h1().text("Welcome")))
                    .child(footer().text("2024")),
            ),
        );
        let output = render_to_string(&view.into_node());

        assert!(output.html.contains("<html>"));
        assert!(output.html.contains("<title>My App</title>"));
        assert!(output.html.contains("<div id=\"root\">"));
        assert!(output.html.contains("<a href=\"/\">Home</a>"));
        assert!(output.html.contains("</html>"));
    }
}
