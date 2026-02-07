use std::cell::RefCell;
use std::rc::Rc;

use react_rs_core::signal::ReadSignal;

pub trait SignalExt<T> {
    fn map<U, F>(&self, f: F) -> MappedSignal<T, U, F>
    where
        F: Fn(&T) -> U + 'static;
}

impl<T: Clone + 'static> SignalExt<T> for ReadSignal<T> {
    fn map<U, F>(&self, f: F) -> MappedSignal<T, U, F>
    where
        F: Fn(&T) -> U + 'static,
    {
        MappedSignal {
            signal: self.clone(),
            mapper: f,
        }
    }
}

pub struct MappedSignal<T, U, F>
where
    F: Fn(&T) -> U,
{
    signal: ReadSignal<T>,
    mapper: F,
}

impl<T: Clone, U, F> MappedSignal<T, U, F>
where
    F: Fn(&T) -> U,
{
    pub fn get(&self) -> U {
        self.signal.with(|v| (self.mapper)(v))
    }
}

pub enum ReactiveValue<T> {
    Static(T),
    Dynamic(Rc<RefCell<T>>),
}

impl<T: Clone> Clone for ReactiveValue<T> {
    fn clone(&self) -> Self {
        match self {
            ReactiveValue::Static(v) => ReactiveValue::Static(v.clone()),
            ReactiveValue::Dynamic(v) => ReactiveValue::Dynamic(v.clone()),
        }
    }
}

impl<T: Clone> ReactiveValue<T> {
    pub fn get(&self) -> T {
        match self {
            ReactiveValue::Static(v) => v.clone(),
            ReactiveValue::Dynamic(v) => v.borrow().clone(),
        }
    }
}

pub trait IntoReactiveString {
    fn into_reactive_string(self) -> ReactiveValue<String>;
}

impl IntoReactiveString for &str {
    fn into_reactive_string(self) -> ReactiveValue<String> {
        ReactiveValue::Static(self.to_string())
    }
}

impl IntoReactiveString for String {
    fn into_reactive_string(self) -> ReactiveValue<String> {
        ReactiveValue::Static(self)
    }
}

impl<T, F> IntoReactiveString for MappedSignal<T, String, F>
where
    T: Clone + 'static,
    F: Fn(&T) -> String + 'static,
{
    fn into_reactive_string(self) -> ReactiveValue<String> {
        use react_rs_core::effect::create_effect;

        let value = Rc::new(RefCell::new(self.get()));
        let value_clone = value.clone();

        create_effect(move || {
            let new_value = self.get();
            *value_clone.borrow_mut() = new_value;
        });

        ReactiveValue::Dynamic(value)
    }
}

pub trait IntoReactiveBool {
    fn into_reactive_bool(self) -> ReactiveValue<bool>;
}

impl IntoReactiveBool for bool {
    fn into_reactive_bool(self) -> ReactiveValue<bool> {
        ReactiveValue::Static(self)
    }
}

impl<T, F> IntoReactiveBool for MappedSignal<T, bool, F>
where
    T: Clone + 'static,
    F: Fn(&T) -> bool + 'static,
{
    fn into_reactive_bool(self) -> ReactiveValue<bool> {
        use react_rs_core::effect::create_effect;

        let value = Rc::new(RefCell::new(self.get()));
        let value_clone = value.clone();

        create_effect(move || {
            let new_value = self.get();
            *value_clone.borrow_mut() = new_value;
        });

        ReactiveValue::Dynamic(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use react_rs_core::signal::create_signal;

    #[test]
    fn test_signal_map() {
        let (count, set_count) = create_signal(5);
        let doubled = count.map(|n| n * 2);

        assert_eq!(doubled.get(), 10);

        set_count.set(10);
        assert_eq!(doubled.get(), 20);
    }

    #[test]
    fn test_mapped_signal_to_string() {
        let (count, set_count) = create_signal(0);
        let text = count.map(|n| format!("Count: {}", n));

        assert_eq!(text.get(), "Count: 0");

        set_count.set(42);
        assert_eq!(text.get(), "Count: 42");
    }

    #[test]
    fn test_reactive_value_static() {
        let value: ReactiveValue<String> = "hello".into_reactive_string();
        assert_eq!(value.get(), "hello");
    }

    #[test]
    fn test_reactive_value_dynamic() {
        let (count, set_count) = create_signal(0);
        let text = count.map(|n| format!("{}", n));
        let reactive: ReactiveValue<String> = text.into_reactive_string();

        assert_eq!(reactive.get(), "0");

        set_count.set(5);
        assert_eq!(reactive.get(), "5");
    }
}
