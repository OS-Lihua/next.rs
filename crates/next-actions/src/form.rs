use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FormData {
    fields: HashMap<String, FormValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FormValue {
    Text(String),
    File(FileData),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub name: String,
    pub size: u64,
    pub content_type: String,
    pub data: Vec<u8>,
}

impl FormData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.fields
            .insert(key.into(), FormValue::Text(value.into()));
    }

    pub fn set_multiple(&mut self, key: impl Into<String>, values: Vec<String>) {
        self.fields.insert(key.into(), FormValue::Multiple(values));
    }

    pub fn set_file(&mut self, key: impl Into<String>, file: FileData) {
        self.fields.insert(key.into(), FormValue::File(file));
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        match self.fields.get(key) {
            Some(FormValue::Text(s)) => Some(s),
            _ => None,
        }
    }

    pub fn get_all(&self, key: &str) -> Vec<&str> {
        match self.fields.get(key) {
            Some(FormValue::Text(s)) => vec![s],
            Some(FormValue::Multiple(v)) => v.iter().map(|s| s.as_str()).collect(),
            _ => vec![],
        }
    }

    pub fn get_file(&self, key: &str) -> Option<&FileData> {
        match self.fields.get(key) {
            Some(FormValue::File(f)) => Some(f),
            _ => None,
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.fields.keys()
    }

    pub fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (key, value) in &self.fields {
            let json_value = match value {
                FormValue::Text(s) => serde_json::Value::String(s.clone()),
                FormValue::Multiple(v) => serde_json::Value::Array(
                    v.iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
                FormValue::File(f) => serde_json::json!({
                    "name": f.name,
                    "size": f.size,
                    "contentType": f.content_type,
                }),
            };
            map.insert(key.clone(), json_value);
        }
        serde_json::Value::Object(map)
    }
}

#[derive(Debug, Clone)]
pub struct FormAction {
    action_id: String,
    method: FormMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormMethod {
    Post,
    Get,
}

impl FormAction {
    pub fn new(action_id: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            method: FormMethod::Post,
        }
    }

    pub fn with_method(mut self, method: FormMethod) -> Self {
        self.method = method;
        self
    }

    pub fn action_id(&self) -> &str {
        &self.action_id
    }

    pub fn method(&self) -> FormMethod {
        self.method
    }

    pub fn action_url(&self) -> String {
        format!("/_actions/{}", self.action_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_data_text() {
        let mut form = FormData::new();
        form.set("name", "John");
        form.set("email", "john@example.com");

        assert_eq!(form.get("name"), Some("John"));
        assert_eq!(form.get("email"), Some("john@example.com"));
        assert_eq!(form.get("missing"), None);
    }

    #[test]
    fn test_form_data_multiple() {
        let mut form = FormData::new();
        form.set_multiple("tags", vec!["rust".to_string(), "web".to_string()]);

        let tags = form.get_all("tags");
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"rust"));
        assert!(tags.contains(&"web"));
    }

    #[test]
    fn test_form_data_file() {
        let mut form = FormData::new();
        form.set_file(
            "avatar",
            FileData {
                name: "photo.jpg".to_string(),
                size: 1024,
                content_type: "image/jpeg".to_string(),
                data: vec![1, 2, 3],
            },
        );

        let file = form.get_file("avatar").unwrap();
        assert_eq!(file.name, "photo.jpg");
        assert_eq!(file.size, 1024);
    }

    #[test]
    fn test_form_action() {
        let action = FormAction::new("create-post");
        assert_eq!(action.action_id(), "create-post");
        assert_eq!(action.method(), FormMethod::Post);
        assert_eq!(action.action_url(), "/_actions/create-post");
    }

    #[test]
    fn test_form_data_to_json() {
        let mut form = FormData::new();
        form.set("name", "Test");

        let json = form.to_json();
        assert_eq!(json["name"], "Test");
    }
}
