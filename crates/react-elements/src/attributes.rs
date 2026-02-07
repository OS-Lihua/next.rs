use crate::reactive::ReactiveValue;

pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

pub enum AttributeValue {
    String(String),
    Bool(bool),
    ReactiveString(ReactiveValue<String>),
    ReactiveBool(ReactiveValue<bool>),
}

impl Attribute {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: AttributeValue::String(value.into()),
        }
    }

    pub fn boolean(name: impl Into<String>, value: bool) -> Self {
        Self {
            name: name.into(),
            value: AttributeValue::Bool(value),
        }
    }

    pub fn reactive_string(name: impl Into<String>, value: ReactiveValue<String>) -> Self {
        Self {
            name: name.into(),
            value: AttributeValue::ReactiveString(value),
        }
    }

    pub fn reactive_bool(name: impl Into<String>, value: ReactiveValue<bool>) -> Self {
        Self {
            name: name.into(),
            value: AttributeValue::ReactiveBool(value),
        }
    }

    pub fn to_static_value(&self) -> String {
        match &self.value {
            AttributeValue::String(s) => s.clone(),
            AttributeValue::Bool(b) => b.to_string(),
            AttributeValue::ReactiveString(r) => r.get(),
            AttributeValue::ReactiveBool(r) => r.get().to_string(),
        }
    }
}
