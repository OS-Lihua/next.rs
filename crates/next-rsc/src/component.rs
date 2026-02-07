use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Server,
    Client,
}

pub trait ServerComponent: Send + Sync {
    fn render(&self) -> Box<dyn Any + Send>;
    fn component_id(&self) -> &str;
}

pub trait ClientComponent: Send + Sync {
    fn component_id(&self) -> &str;
    fn props_json(&self) -> String;
    fn client_module(&self) -> &str;
}

pub struct ServerComponentWrapper<F>
where
    F: Fn() -> Box<dyn Any + Send> + Send + Sync,
{
    id: String,
    render_fn: F,
}

impl<F> ServerComponentWrapper<F>
where
    F: Fn() -> Box<dyn Any + Send> + Send + Sync,
{
    pub fn new(id: impl Into<String>, render_fn: F) -> Self {
        Self {
            id: id.into(),
            render_fn,
        }
    }
}

impl<F> ServerComponent for ServerComponentWrapper<F>
where
    F: Fn() -> Box<dyn Any + Send> + Send + Sync,
{
    fn render(&self) -> Box<dyn Any + Send> {
        (self.render_fn)()
    }

    fn component_id(&self) -> &str {
        &self.id
    }
}

pub struct ClientComponentRef {
    id: String,
    module: String,
    props: String,
}

impl ClientComponentRef {
    pub fn new(id: impl Into<String>, module: impl Into<String>, props: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            module: module.into(),
            props: props.into(),
        }
    }
}

impl ClientComponent for ClientComponentRef {
    fn component_id(&self) -> &str {
        &self.id
    }

    fn props_json(&self) -> String {
        self.props.clone()
    }

    fn client_module(&self) -> &str {
        &self.module
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_component_wrapper() {
        let component = ServerComponentWrapper::new("test-component", || {
            Box::new("rendered content".to_string())
        });

        assert_eq!(component.component_id(), "test-component");
        let result = component.render();
        assert!(result.downcast::<String>().is_ok());
    }

    #[test]
    fn test_client_component_ref() {
        let component =
            ClientComponentRef::new("counter", "./components/Counter.js", r#"{"initial": 0}"#);

        assert_eq!(component.component_id(), "counter");
        assert_eq!(component.client_module(), "./components/Counter.js");
        assert_eq!(component.props_json(), r#"{"initial": 0}"#);
    }
}
