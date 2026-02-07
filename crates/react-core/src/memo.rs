use crate::effect::create_effect;
use crate::signal::{create_signal, ReadSignal};

pub struct Memo<T> {
    read: ReadSignal<T>,
}

impl<T: Clone> Memo<T> {
    pub fn get(&self) -> T {
        self.read.get()
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.read.with(f)
    }
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            read: self.read.clone(),
        }
    }
}

pub fn create_memo<T, F>(f: F) -> Memo<T>
where
    F: Fn() -> T + 'static,
    T: PartialEq + Clone + 'static,
{
    let initial = f();
    let (read, write) = create_signal(initial);

    create_effect(move || {
        let new_value = f();
        write.update(|current| {
            if *current != new_value {
                *current = new_value;
            }
        });
    });

    Memo { read }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::create_signal;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_memo_basic() {
        let (count, set_count) = create_signal(2);
        let doubled = create_memo(move || count.get() * 2);

        assert_eq!(doubled.get(), 4);

        set_count.set(5);
        assert_eq!(doubled.get(), 10);
    }

    #[test]
    fn test_memo_caching() {
        let (count, set_count) = create_signal(1);
        let compute_count = Rc::new(RefCell::new(0));
        let compute_count_clone = compute_count.clone();

        let doubled = create_memo(move || {
            *compute_count_clone.borrow_mut() += 1;
            count.get() * 2
        });

        assert_eq!(doubled.get(), 2);
        assert_eq!(doubled.get(), 2);
        let initial_count = *compute_count.borrow();

        set_count.set(2);
        assert_eq!(doubled.get(), 4);
        assert_eq!(*compute_count.borrow(), initial_count + 1);
    }

    #[test]
    fn test_memo_only_updates_on_change() {
        let (count, set_count) = create_signal(1);
        let compute_count = Rc::new(RefCell::new(0));
        let compute_count_clone = compute_count.clone();

        let is_even = create_memo(move || {
            *compute_count_clone.borrow_mut() += 1;
            count.get() % 2 == 0
        });

        assert_eq!(is_even.get(), false);
        let count_after_init = *compute_count.borrow();

        set_count.set(3);
        assert_eq!(is_even.get(), false);
        assert_eq!(*compute_count.borrow(), count_after_init + 1);

        set_count.set(4);
        assert_eq!(is_even.get(), true);
        assert_eq!(*compute_count.borrow(), count_after_init + 2);
    }

    #[test]
    fn test_memo_chain() {
        let (count, set_count) = create_signal(1);

        let doubled = {
            let count = count.clone();
            create_memo(move || count.get() * 2)
        };

        let quadrupled = {
            let doubled = doubled.clone();
            create_memo(move || doubled.get() * 2)
        };

        assert_eq!(count.get(), 1);
        assert_eq!(doubled.get(), 2);
        assert_eq!(quadrupled.get(), 4);

        set_count.set(3);

        assert_eq!(count.get(), 3);
        assert_eq!(doubled.get(), 6);
        assert_eq!(quadrupled.get(), 12);
    }
}
