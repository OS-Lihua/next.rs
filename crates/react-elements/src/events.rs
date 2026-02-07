pub struct Event {
    pub event_type: String,
    pub target_value: Option<String>,
    pub checked: Option<bool>,
}

impl Event {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            target_value: None,
            checked: None,
        }
    }

    pub fn with_target_value(mut self, value: String) -> Self {
        self.target_value = Some(value);
        self
    }

    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn value(&self) -> &str {
        self.target_value.as_deref().unwrap_or("")
    }
}

pub struct EventHandler {
    pub event_type: String,
    handler: std::rc::Rc<dyn Fn(Event)>,
}

impl EventHandler {
    pub fn new<F>(event_type: impl Into<String>, handler: F) -> Self
    where
        F: Fn(Event) + 'static,
    {
        Self {
            event_type: event_type.into(),
            handler: std::rc::Rc::new(handler),
        }
    }

    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    pub fn invoke(&self, event: Event) {
        (self.handler)(event);
    }

    pub fn take_handler_rc(&self) -> std::rc::Rc<dyn Fn(Event)> {
        self.handler.clone()
    }
}
