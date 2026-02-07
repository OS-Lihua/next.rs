use std::marker::PhantomData;

pub trait IntoView {
    type View;
    fn into_view(self) -> Self::View;
}

impl<T> IntoView for T
where
    T: Sized,
{
    type View = T;
    fn into_view(self) -> Self::View {
        self
    }
}

pub struct Component<P, F, V>
where
    F: Fn(P) -> V,
    V: IntoView,
{
    render: F,
    _phantom: PhantomData<P>,
}

impl<P, F, V> Component<P, F, V>
where
    F: Fn(P) -> V,
    V: IntoView,
{
    pub fn new(render: F) -> Self {
        Self {
            render,
            _phantom: PhantomData,
        }
    }

    pub fn call(&self, props: P) -> V::View {
        (self.render)(props).into_view()
    }
}

pub fn component<P, F, V>(render: F) -> Component<P, F, V>
where
    F: Fn(P) -> V,
    V: IntoView,
{
    Component::new(render)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_component() {
        fn greeting(name: String) -> String {
            format!("Hello, {}!", name)
        }

        let comp = component(greeting);
        let result = comp.call("World".to_string());
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_component_with_props_struct() {
        #[derive(Clone)]
        struct ButtonProps {
            label: String,
            disabled: bool,
        }

        fn button_view(props: ButtonProps) -> String {
            if props.disabled {
                format!("[{}] (disabled)", props.label)
            } else {
                format!("[{}]", props.label)
            }
        }

        let button = component(button_view);

        let result = button.call(ButtonProps {
            label: "Click me".to_string(),
            disabled: false,
        });
        assert_eq!(result, "[Click me]");

        let result_disabled = button.call(ButtonProps {
            label: "Submit".to_string(),
            disabled: true,
        });
        assert_eq!(result_disabled, "[Submit] (disabled)");
    }

    #[test]
    fn test_closure_component() {
        let prefix = "Greeting: ";
        let greet = component(move |name: String| format!("{}{}", prefix, name));

        assert_eq!(greet.call("Alice".to_string()), "Greeting: Alice");
    }
}
