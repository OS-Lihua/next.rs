#[macro_export]
macro_rules! use_client {
    ($module:expr => $name:ident, $render:expr) => {{
        $crate::global_registry().register_client($module, stringify!($name));
        $crate::client_component(stringify!($name), $module, $render)
    }};

    ($module:expr => fn $name:ident() $body:block) => {{
        $crate::global_registry().register_client($module, stringify!($name));
        $crate::client_component(stringify!($name), $module, || $body)
    }};

    ($name:ident, $module:expr, $render:expr) => {{
        $crate::global_registry().register_client($module, stringify!($name));
        $crate::client_component(stringify!($name), $module, $render)
    }};
}

#[macro_export]
macro_rules! use_server {
    ($module:expr => $name:ident, $handler:expr) => {{
        $crate::global_registry().register_server($module, stringify!($name));
        $crate::ServerActionWrapper::new(stringify!($name), $handler)
    }};

    ($name:ident, $handler:expr) => {{
        let module = concat!(module_path!(), "/", file!());
        $crate::global_registry().register_server(module, stringify!($name));
        $crate::ServerActionWrapper::new(stringify!($name), $handler)
    }};
}

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

pub type ServerActionResult<T> = Result<T, ServerActionError>;

#[derive(Debug, Clone)]
pub struct ServerActionError {
    pub message: String,
}

impl ServerActionError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ServerActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ServerActionError {}

pub struct ServerActionWrapper<I, O, F>
where
    I: for<'de> Deserialize<'de> + Send,
    O: Serialize + Send,
    F: Fn(I) -> Pin<Box<dyn Future<Output = ServerActionResult<O>> + Send>> + Send + Sync,
{
    id: String,
    handler: F,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O, F> ServerActionWrapper<I, O, F>
where
    I: for<'de> Deserialize<'de> + Send,
    O: Serialize + Send,
    F: Fn(I) -> Pin<Box<dyn Future<Output = ServerActionResult<O>> + Send>> + Send + Sync,
{
    pub fn new(id: impl Into<String>, handler: F) -> Self {
        Self {
            id: id.into(),
            handler,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn call(&self, input: I) -> ServerActionResult<O> {
        (self.handler)(input).await
    }

    pub fn to_action_reference(&self) -> ActionReference {
        ActionReference {
            id: self.id.clone(),
            bound_args: None,
        }
    }

    pub fn bind<A: Serialize>(&self, args: A) -> ActionReference {
        ActionReference {
            id: self.id.clone(),
            bound_args: serde_json::to_value(args).ok(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionReference {
    pub id: String,
    pub bound_args: Option<serde_json::Value>,
}

impl ActionReference {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn has_bound_args(&self) -> bool {
        self.bound_args.is_some()
    }
}

#[cfg(test)]
mod tests {
    use crate::global_registry;
    use react_rs_elements::html::*;

    #[test]
    fn test_use_client_macro() {
        let counter = use_client!("./Counter.wasm" => Counter, || {
            div().class("counter").text("0")
        });

        assert_eq!(counter.id(), "Counter");
        assert_eq!(counter.module(), "./Counter.wasm");
        assert!(global_registry().is_client("./Counter.wasm", "Counter"));
    }

    #[test]
    fn test_use_client_macro_alternative_syntax() {
        let button = use_client!(Button, "./Button.wasm", || { button().text("Click me") });

        assert_eq!(button.id(), "Button");
        assert!(global_registry().is_client("./Button.wasm", "Button"));
    }

    #[tokio::test]
    async fn test_use_server_macro() {
        let create_todo = use_server!("./actions" => createTodo, |title: String| {
            Box::pin(async move {
                Ok(format!("Created: {}", title))
            })
        });

        assert_eq!(create_todo.id(), "createTodo");
        assert!(global_registry().is_server("./actions", "createTodo"));

        let result = create_todo.call("Buy milk".to_string()).await;
        assert_eq!(result.unwrap(), "Created: Buy milk");
    }

    #[test]
    fn test_action_reference() {
        let action = use_server!("./actions" => deleteItem, |id: u64| {
            Box::pin(async move {
                Ok(format!("Deleted item {}", id))
            })
        });

        let reference = action.to_action_reference();
        assert_eq!(reference.id(), "deleteItem");
        assert!(!reference.has_bound_args());

        let bound_ref = action.bind(serde_json::json!({"item_id": 42}));
        assert!(bound_ref.has_bound_args());
    }
}
