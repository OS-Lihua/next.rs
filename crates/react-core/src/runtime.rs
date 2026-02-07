use std::cell::RefCell;
use std::collections::VecDeque;

thread_local! {
    pub static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

type EffectId = usize;
type EffectFn = Box<dyn Fn()>;

pub struct Runtime {
    effects: Vec<Option<EffectFn>>,
    current_effect: Option<EffectId>,
    pending_effects: VecDeque<EffectId>,
    is_batching: bool,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            current_effect: None,
            pending_effects: VecDeque::new(),
            is_batching: false,
        }
    }

    pub fn current_effect(&self) -> Option<EffectId> {
        self.current_effect
    }

    pub fn register_effect(&mut self, f: impl Fn() + 'static) -> EffectId {
        let id = self.effects.len();
        self.effects.push(Some(Box::new(f)));
        id
    }

    pub fn set_current_effect(&mut self, id: Option<EffectId>) -> Option<EffectId> {
        let prev = self.current_effect;
        self.current_effect = id;
        prev
    }

    pub fn get_effect(&self, id: EffectId) -> Option<&EffectFn> {
        self.effects.get(id).and_then(|e| e.as_ref())
    }

    pub fn schedule_effect(&mut self, id: EffectId) {
        if !self.pending_effects.contains(&id) {
            self.pending_effects.push_back(id);
        }
    }

    pub fn is_batching(&self) -> bool {
        self.is_batching
    }

    pub fn pop_pending_effect(&mut self) -> Option<EffectId> {
        self.pending_effects.pop_front()
    }

    pub fn start_batch(&mut self) -> bool {
        let was_batching = self.is_batching;
        self.is_batching = true;
        was_batching
    }

    pub fn end_batch(&mut self, was_batching: bool) {
        self.is_batching = was_batching;
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
