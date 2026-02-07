pub struct Event {
    pub event_type: String,
}

impl Event {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
        }
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
