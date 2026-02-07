use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::action::{ActionError, ActionRequest, ActionResponse, ActionResult};

type BoxedHandler = Box<
    dyn Fn(
            serde_json::Value,
        ) -> Pin<Box<dyn Future<Output = ActionResult<serde_json::Value>> + Send>>
        + Send
        + Sync,
>;

pub struct ActionRegistry {
    handlers: HashMap<String, Arc<BoxedHandler>>,
}

impl ActionRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register<F, Fut, I, O>(&mut self, action_id: impl Into<String>, handler: F)
    where
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ActionResult<O>> + Send + 'static,
        I: for<'de> serde::Deserialize<'de> + Send + 'static,
        O: serde::Serialize + Send + 'static,
    {
        let id = action_id.into();
        let wrapped: BoxedHandler = Box::new(move |value: serde_json::Value| {
            let input: Result<I, _> = serde_json::from_value(value);
            match input {
                Ok(input) => {
                    let future = handler(input);
                    Box::pin(async move {
                        let result = future.await?;
                        serde_json::to_value(result)
                            .map_err(|e| ActionError::new(format!("Serialization error: {}", e)))
                    })
                }
                Err(e) => Box::pin(async move {
                    Err(ActionError::with_code(
                        format!("Invalid input: {}", e),
                        "INVALID_INPUT",
                    ))
                }),
            }
        });

        self.handlers.insert(id, Arc::new(wrapped));
    }

    pub fn has(&self, action_id: &str) -> bool {
        self.handlers.contains_key(action_id)
    }

    pub async fn execute(&self, request: ActionRequest) -> ActionResponse {
        match self.handlers.get(&request.action_id) {
            Some(handler) => match handler(request.payload).await {
                Ok(data) => ActionResponse::success(data),
                Err(error) => ActionResponse::error(error),
            },
            None => ActionResponse::error(ActionError::with_code(
                format!("Action '{}' not found", request.action_id),
                "ACTION_NOT_FOUND",
            )),
        }
    }

    pub fn action_ids(&self) -> impl Iterator<Item = &String> {
        self.handlers.keys()
    }
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_register_and_execute() {
        let mut registry = ActionRegistry::new();

        registry.register("greet", |name: String| async move {
            Ok(format!("Hello, {}!", name))
        });

        assert!(registry.has("greet"));
        assert!(!registry.has("unknown"));

        let request = ActionRequest {
            action_id: "greet".to_string(),
            payload: serde_json::json!("World"),
        };

        let response = registry.execute(request).await;
        assert!(response.success);
        assert_eq!(response.data.unwrap(), "Hello, World!");
    }

    #[tokio::test]
    async fn test_registry_action_not_found() {
        let registry = ActionRegistry::new();

        let request = ActionRequest {
            action_id: "missing".to_string(),
            payload: serde_json::json!({}),
        };

        let response = registry.execute(request).await;
        assert!(!response.success);
        assert!(response.error.is_some());
        assert_eq!(
            response.error.unwrap().code,
            Some("ACTION_NOT_FOUND".to_string())
        );
    }

    #[tokio::test]
    async fn test_registry_invalid_input() {
        let mut registry = ActionRegistry::new();

        #[derive(serde::Deserialize)]
        struct CreatePost {
            title: String,
            #[allow(dead_code)]
            content: String,
        }

        registry.register("create-post", |post: CreatePost| async move {
            Ok(format!("Created: {}", post.title))
        });

        let request = ActionRequest {
            action_id: "create-post".to_string(),
            payload: serde_json::json!({"title": "Test"}),
        };

        let response = registry.execute(request).await;
        assert!(!response.success);
        assert!(response.error.is_some());
    }

    #[test]
    fn test_registry_action_ids() {
        let mut registry = ActionRegistry::new();
        registry.register("action1", |_: ()| async { Ok(()) });
        registry.register("action2", |_: ()| async { Ok(()) });

        let ids: Vec<_> = registry.action_ids().collect();
        assert_eq!(ids.len(), 2);
    }
}
