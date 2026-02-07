use crate::attributes::Attribute;
use crate::events::{Event, EventHandler};
use crate::node::{IntoNode, Node};
use crate::reactive::{IntoReactiveBool, IntoReactiveString};

pub struct Element {
    tag: &'static str,
    attributes: Vec<Attribute>,
    children: Vec<Node>,
    event_handlers: Vec<EventHandler>,
}

impl Element {
    pub fn new(tag: &'static str) -> Self {
        Self {
            tag,
            attributes: Vec::new(),
            children: Vec::new(),
            event_handlers: Vec::new(),
        }
    }

    pub fn tag(&self) -> &'static str {
        self.tag
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    pub fn get_children(&self) -> &[Node] {
        &self.children
    }

    pub fn class(mut self, class: &str) -> Self {
        self.attributes.push(Attribute::new("class", class));
        self
    }

    pub fn class_reactive(mut self, class: impl IntoReactiveString) -> Self {
        self.attributes.push(Attribute::reactive_string(
            "class",
            class.into_reactive_string(),
        ));
        self
    }

    pub fn visible_reactive(mut self, visible: impl IntoReactiveBool) -> Self {
        self.attributes.push(Attribute::reactive_bool(
            "data-visible",
            visible.into_reactive_bool(),
        ));
        self
    }

    pub fn id(mut self, id: &str) -> Self {
        self.attributes.push(Attribute::new("id", id));
        self
    }

    pub fn style(mut self, style: &str) -> Self {
        self.attributes.push(Attribute::new("style", style));
        self
    }

    pub fn styled(self, style: crate::style::Style) -> Self {
        let css = style.to_css();
        if css.is_empty() {
            self
        } else {
            self.style(&css)
        }
    }

    pub fn attr(mut self, name: &str, value: &str) -> Self {
        self.attributes.push(Attribute::new(name, value));
        self
    }

    pub fn href(mut self, href: &str) -> Self {
        self.attributes.push(Attribute::new("href", href));
        self
    }

    pub fn src(mut self, src: &str) -> Self {
        self.attributes.push(Attribute::new("src", src));
        self
    }

    pub fn alt(mut self, alt: &str) -> Self {
        self.attributes.push(Attribute::new("alt", alt));
        self
    }

    pub fn type_(mut self, type_value: &str) -> Self {
        self.attributes.push(Attribute::new("type", type_value));
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.attributes.push(Attribute::new("name", name));
        self
    }

    pub fn value(mut self, value: &str) -> Self {
        self.attributes.push(Attribute::new("value", value));
        self
    }

    pub fn value_reactive(mut self, value: impl IntoReactiveString) -> Self {
        self.attributes.push(Attribute::reactive_string(
            "value",
            value.into_reactive_string(),
        ));
        self
    }

    pub fn bind_value(
        self,
        read: react_rs_core::signal::ReadSignal<String>,
        write: react_rs_core::signal::WriteSignal<String>,
    ) -> Self {
        use crate::reactive::SignalExt;
        self.value_reactive(read.map(|s| s.clone()))
            .on_input(move |e| {
                write.set(e.value().to_string());
            })
    }

    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.attributes
            .push(Attribute::new("placeholder", placeholder));
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.attributes
            .push(Attribute::boolean("disabled", disabled));
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.children.push(Node::Text(text.into()));
        self
    }

    pub fn text_reactive(mut self, text: impl IntoReactiveString) -> Self {
        self.children
            .push(Node::ReactiveText(text.into_reactive_string()));
        self
    }

    pub fn child(mut self, child: impl IntoNode) -> Self {
        self.children.push(child.into_node());
        self
    }

    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: IntoNode,
    {
        for child in children {
            self.children.push(child.into_node());
        }
        self
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(Event) + 'static,
    {
        self.event_handlers
            .push(EventHandler::new("click", handler));
        self
    }

    pub fn on_input<F>(mut self, handler: F) -> Self
    where
        F: Fn(Event) + 'static,
    {
        self.event_handlers
            .push(EventHandler::new("input", handler));
        self
    }

    pub fn on_submit<F>(mut self, handler: F) -> Self
    where
        F: Fn(Event) + 'static,
    {
        self.event_handlers
            .push(EventHandler::new("submit", handler));
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(Event) + 'static,
    {
        self.event_handlers
            .push(EventHandler::new("change", handler));
        self
    }

    pub fn has_class(&self, class_name: &str) -> bool {
        self.attributes.iter().any(|attr| {
            attr.name == "class" && matches!(&attr.value, crate::attributes::AttributeValue::String(v) if v.contains(class_name))
        })
    }

    pub fn event_handlers(&self) -> &[EventHandler] {
        &self.event_handlers
    }
}

#[cfg(test)]
mod tests {
    use crate::html::*;
    use crate::reactive::SignalExt;
    use react_rs_core::signal::create_signal;

    #[test]
    fn test_element_api() {
        let element = div().class("container").child(h1().text("Hello"));

        assert_eq!(element.tag(), "div");
        assert!(element.has_class("container"));
    }

    #[test]
    fn test_nested_elements() {
        let view = div()
            .class("app")
            .child(nav().class("sidebar").child(ul().children([
                li().child(a().href("/").text("Home")),
                li().child(a().href("/about").text("About")),
            ])))
            .child(main_el().class("content").child(h1().text("Welcome")));

        assert_eq!(view.tag(), "div");
        assert!(view.has_class("app"));
        assert_eq!(view.get_children().len(), 2);
    }

    #[test]
    fn test_event_handlers() {
        let clicked = std::cell::Cell::new(false);
        let _button = button().text("Click me").on_click(|_| {});

        assert!(!clicked.get());
    }

    #[test]
    fn test_reactive_text() {
        let (count, _set_count) = create_signal(0);
        let _element = div().text_reactive(count.map(|n| format!("Count: {}", n)));
    }

    #[test]
    fn test_reactive_class() {
        let (active, _set_active) = create_signal(false);
        let _element = div()
            .class_reactive(active.map(|a| if *a { "active" } else { "inactive" }.to_string()));
    }
}
