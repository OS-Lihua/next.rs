use crate::signal::{create_signal, ReadSignal, WriteSignal};

#[derive(Debug, Clone, PartialEq)]
pub enum ResourceState<T> {
    Loading,
    Ready(T),
    Error(String),
}

impl<T> ResourceState<T> {
    pub fn is_loading(&self) -> bool {
        matches!(self, ResourceState::Loading)
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, ResourceState::Ready(_))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, ResourceState::Error(_))
    }

    pub fn data(&self) -> Option<&T> {
        match self {
            ResourceState::Ready(d) => Some(d),
            _ => None,
        }
    }

    pub fn error(&self) -> Option<&str> {
        match self {
            ResourceState::Error(e) => Some(e),
            _ => None,
        }
    }
}

pub struct Resource<T: Clone + 'static> {
    state: ReadSignal<ResourceState<T>>,
    set_state: WriteSignal<ResourceState<T>>,
}

impl<T: Clone + 'static> Resource<T> {
    pub fn state(&self) -> ReadSignal<ResourceState<T>> {
        self.state.clone()
    }

    pub fn read(&self) -> ResourceState<T> {
        self.state.get()
    }

    pub fn loading(&self) -> bool {
        self.state.get().is_loading()
    }

    pub fn data(&self) -> Option<T> {
        match self.state.get() {
            ResourceState::Ready(d) => Some(d),
            _ => None,
        }
    }

    pub fn set_ready(&self, data: T) {
        self.set_state.set(ResourceState::Ready(data));
    }

    pub fn set_error(&self, error: impl Into<String>) {
        self.set_state.set(ResourceState::Error(error.into()));
    }

    pub fn set_loading(&self) {
        self.set_state.set(ResourceState::Loading);
    }
}

impl<T: Clone + 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            set_state: self.set_state.clone(),
        }
    }
}

pub fn create_resource<T: Clone + 'static>() -> Resource<T> {
    let (state, set_state) = create_signal(ResourceState::Loading);
    Resource { state, set_state }
}

pub fn create_resource_with<T: Clone + 'static>(initial: T) -> Resource<T> {
    let (state, set_state) = create_signal(ResourceState::Ready(initial));
    Resource { state, set_state }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_initial_loading() {
        let resource = create_resource::<String>();
        assert!(resource.loading());
        assert!(resource.data().is_none());
    }

    #[test]
    fn test_resource_set_ready() {
        let resource = create_resource::<String>();
        resource.set_ready("hello".to_string());
        assert!(!resource.loading());
        assert_eq!(resource.data(), Some("hello".to_string()));
    }

    #[test]
    fn test_resource_set_error() {
        let resource = create_resource::<String>();
        resource.set_error("network error");
        assert!(resource.read().is_error());
        assert_eq!(resource.read().error(), Some("network error"));
    }

    #[test]
    fn test_resource_with_initial() {
        let resource = create_resource_with(42);
        assert!(!resource.loading());
        assert_eq!(resource.data(), Some(42));
    }

    #[test]
    fn test_resource_state_transitions() {
        let resource = create_resource::<Vec<String>>();
        assert!(resource.loading());

        resource.set_ready(vec!["a".to_string()]);
        assert_eq!(resource.data().unwrap().len(), 1);

        resource.set_loading();
        assert!(resource.loading());

        resource.set_error("timeout");
        assert!(resource.read().is_error());
    }
}
