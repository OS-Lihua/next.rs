use std::cell::RefCell;
use std::rc::Rc;

use crate::effect::flush_effects;
use crate::runtime::RUNTIME;

type SubscriberId = usize;

struct SignalInner<T> {
    value: T,
    subscribers: Vec<SubscriberId>,
    version: u64,
}

pub struct ReadSignal<T> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub struct WriteSignal<T> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

impl<T> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

pub fn create_signal<T>(value: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let inner = Rc::new(RefCell::new(SignalInner {
        value,
        subscribers: Vec::new(),
        version: 0,
    }));

    (
        ReadSignal {
            inner: inner.clone(),
        },
        WriteSignal { inner },
    )
}

impl<T: Clone> ReadSignal<T> {
    pub fn get(&self) -> T {
        self.track();
        self.inner.borrow().value.clone()
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.track();
        f(&self.inner.borrow().value)
    }

    pub fn get_untracked(&self) -> T {
        self.inner.borrow().value.clone()
    }

    fn track(&self) {
        RUNTIME.with(|rt| {
            if let Some(effect_id) = rt.borrow().current_effect() {
                let mut inner = self.inner.borrow_mut();
                if !inner.subscribers.contains(&effect_id) {
                    inner.subscribers.push(effect_id);
                }
            }
        });
    }
}

impl<T> WriteSignal<T> {
    pub fn set(&self, value: T) {
        {
            let mut inner = self.inner.borrow_mut();
            inner.value = value;
            inner.version += 1;
        }
        self.notify_subscribers();
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        {
            let mut inner = self.inner.borrow_mut();
            f(&mut inner.value);
            inner.version += 1;
        }
        self.notify_subscribers();
    }

    fn notify_subscribers(&self) {
        let subscribers: Vec<SubscriberId> = { self.inner.borrow().subscribers.clone() };

        let should_flush = RUNTIME.with(|rt| {
            let mut rt = rt.borrow_mut();
            for subscriber_id in subscribers {
                rt.schedule_effect(subscriber_id);
            }
            !rt.is_batching()
        });

        if should_flush {
            flush_effects();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_read_write() {
        let (read, write) = create_signal(0);
        assert_eq!(read.get_untracked(), 0);
        write.set(5);
        assert_eq!(read.get_untracked(), 5);
    }

    #[test]
    fn test_signal_update() {
        let (read, write) = create_signal(vec![1, 2]);
        write.update(|v| v.push(3));
        assert_eq!(read.get_untracked(), vec![1, 2, 3]);
    }

    #[test]
    fn test_signal_with() {
        let (read, _write) = create_signal(String::from("hello"));
        let len = read.with(|s| s.len());
        assert_eq!(len, 5);
    }
}
