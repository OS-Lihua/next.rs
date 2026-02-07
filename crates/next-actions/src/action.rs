use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

pub type ActionResult<T> = Result<T, ActionError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionError {
    pub message: String,
    pub code: Option<String>,
}

impl ActionError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
        }
    }

    pub fn with_code(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: Some(code.into()),
        }
    }
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ActionError {}

pub trait ServerAction: Send + Sync {
    type Input: for<'de> Deserialize<'de> + Send;
    type Output: Serialize + Send;

    fn action_id(&self) -> &str;

    fn execute(
        &self,
        input: Self::Input,
    ) -> Pin<Box<dyn Future<Output = ActionResult<Self::Output>> + Send>>;
}

pub struct Action<I, O, F>
where
    I: for<'de> Deserialize<'de> + Send,
    O: Serialize + Send,
    F: Fn(I) -> Pin<Box<dyn Future<Output = ActionResult<O>> + Send>> + Send + Sync,
{
    id: String,
    handler: F,
    _phantom: std::marker::PhantomData<(I, O)>,
}

impl<I, O, F> Action<I, O, F>
where
    I: for<'de> Deserialize<'de> + Send,
    O: Serialize + Send,
    F: Fn(I) -> Pin<Box<dyn Future<Output = ActionResult<O>> + Send>> + Send + Sync,
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

    pub async fn call(&self, input: I) -> ActionResult<O> {
        (self.handler)(input).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    pub action_id: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<ActionError>,
}

impl ActionResponse {
    pub fn success<T: Serialize>(data: T) -> Self {
        Self {
            success: true,
            data: serde_json::to_value(data).ok(),
            error: None,
        }
    }

    pub fn error(error: ActionError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_error() {
        let error = ActionError::new("Something went wrong");
        assert_eq!(error.message, "Something went wrong");
        assert!(error.code.is_none());

        let error_with_code = ActionError::with_code("Not found", "404");
        assert_eq!(error_with_code.code, Some("404".to_string()));
    }

    #[test]
    fn test_action_response_success() {
        let response = ActionResponse::success(serde_json::json!({"id": 1}));
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_action_response_error() {
        let response = ActionResponse::error(ActionError::new("Failed"));
        assert!(!response.success);
        assert!(response.data.is_none());
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_action_call() {
        let action = Action::new("test-action", |input: String| {
            Box::pin(async move { Ok(format!("Hello, {}!", input)) })
        });

        let result = action.call("World".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, World!");
    }
}
