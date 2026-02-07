use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static CONTEXT_MAP: RefCell<HashMap<TypeId, Box<dyn Any>>> = RefCell::new(HashMap::new());
}

pub fn provide_context<T: Clone + 'static>(value: T) {
    CONTEXT_MAP.with(|map| {
        map.borrow_mut().insert(TypeId::of::<T>(), Box::new(value));
    });
}

pub fn use_context<T: Clone + 'static>() -> Option<T> {
    CONTEXT_MAP.with(|map| {
        map.borrow()
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    })
}

pub fn use_context_or<T: Clone + 'static>(default: T) -> T {
    use_context::<T>().unwrap_or(default)
}

pub fn clear_context<T: 'static>() {
    CONTEXT_MAP.with(|map| {
        map.borrow_mut().remove(&TypeId::of::<T>());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct Theme {
        dark_mode: bool,
        primary_color: String,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct UserInfo {
        name: String,
        id: u32,
    }

    fn cleanup() {
        clear_context::<Theme>();
        clear_context::<UserInfo>();
        clear_context::<i32>();
    }

    #[test]
    fn test_provide_and_use_context() {
        cleanup();

        let theme = Theme {
            dark_mode: true,
            primary_color: "blue".to_string(),
        };

        provide_context(theme.clone());

        let retrieved = use_context::<Theme>();
        assert_eq!(retrieved, Some(theme));

        cleanup();
    }

    #[test]
    fn test_context_not_found() {
        cleanup();

        let result = use_context::<UserInfo>();
        assert_eq!(result, None);

        cleanup();
    }

    #[test]
    fn test_multiple_contexts() {
        cleanup();

        let theme = Theme {
            dark_mode: false,
            primary_color: "green".to_string(),
        };
        let user = UserInfo {
            name: "Alice".to_string(),
            id: 42,
        };

        provide_context(theme.clone());
        provide_context(user.clone());

        assert_eq!(use_context::<Theme>(), Some(theme));
        assert_eq!(use_context::<UserInfo>(), Some(user));

        cleanup();
    }

    #[test]
    fn test_use_context_or_default() {
        cleanup();

        let default_value = 100;
        let result = use_context_or::<i32>(default_value);
        assert_eq!(result, 100);

        provide_context(42i32);
        let result = use_context_or::<i32>(default_value);
        assert_eq!(result, 42);

        cleanup();
    }

    #[test]
    fn test_context_override() {
        cleanup();

        provide_context(10i32);
        assert_eq!(use_context::<i32>(), Some(10));

        provide_context(20i32);
        assert_eq!(use_context::<i32>(), Some(20));

        cleanup();
    }
}
