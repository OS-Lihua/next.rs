use crate::runtime::RUNTIME;

pub fn create_effect<F>(f: F)
where
    F: Fn() + 'static,
{
    let effect_id = RUNTIME.with(|rt| rt.borrow_mut().register_effect(f));
    run_effect(effect_id);
}

pub(crate) fn run_effect(id: usize) {
    RUNTIME.with(|rt| {
        let prev = rt.borrow_mut().set_current_effect(Some(id));

        let effect_ptr: Option<*const dyn Fn()> = rt
            .borrow()
            .get_effect(id)
            .map(|e| e.as_ref() as *const dyn Fn());

        if let Some(effect) = effect_ptr {
            unsafe { (*effect)() };
        }

        rt.borrow_mut().set_current_effect(prev);
    });
}

pub(crate) fn flush_effects() {
    loop {
        let effect_id = RUNTIME.with(|rt| rt.borrow_mut().pop_pending_effect());
        match effect_id {
            Some(id) => run_effect(id),
            None => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::create_signal;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_effect_runs_immediately() {
        let ran = Rc::new(RefCell::new(false));
        let ran_clone = ran.clone();

        create_effect(move || {
            *ran_clone.borrow_mut() = true;
        });

        assert!(*ran.borrow());
    }

    #[test]
    fn test_effect_auto_run() {
        let (count, set_count) = create_signal(0);
        let effect_ran = Rc::new(RefCell::new(0));
        let effect_ran_clone = effect_ran.clone();

        create_effect(move || {
            let _ = count.get();
            *effect_ran_clone.borrow_mut() += 1;
        });

        assert_eq!(*effect_ran.borrow(), 1);
        set_count.set(1);
        assert_eq!(*effect_ran.borrow(), 2);
    }

    #[test]
    fn test_effect_only_runs_when_tracked_signal_changes() {
        let (count, set_count) = create_signal(0);
        let (other, _set_other) = create_signal(100);
        let effect_ran = Rc::new(RefCell::new(0));
        let effect_ran_clone = effect_ran.clone();

        create_effect(move || {
            let _ = count.get();
            let _ = other.get_untracked();
            *effect_ran_clone.borrow_mut() += 1;
        });

        assert_eq!(*effect_ran.borrow(), 1);

        set_count.set(1);
        assert_eq!(*effect_ran.borrow(), 2);

        set_count.set(2);
        assert_eq!(*effect_ran.borrow(), 3);
    }

    #[test]
    fn test_multiple_effects() {
        let (count, set_count) = create_signal(0);
        let effect1_ran = Rc::new(RefCell::new(0));
        let effect2_ran = Rc::new(RefCell::new(0));

        let e1 = effect1_ran.clone();
        let c1 = count.clone();
        create_effect(move || {
            let _ = c1.get();
            *e1.borrow_mut() += 1;
        });

        let e2 = effect2_ran.clone();
        create_effect(move || {
            let _ = count.get();
            *e2.borrow_mut() += 1;
        });

        assert_eq!(*effect1_ran.borrow(), 1);
        assert_eq!(*effect2_ran.borrow(), 1);

        set_count.set(1);

        assert_eq!(*effect1_ran.borrow(), 2);
        assert_eq!(*effect2_ran.borrow(), 2);
    }
}
