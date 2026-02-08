use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

thread_local! {
    pub static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

pub type EffectId = usize;
pub type ScopeId = usize;
type EffectFn = Rc<dyn Fn()>;
type CleanupFn = Box<dyn FnOnce()>;

struct Scope {
    effects: Vec<EffectId>,
    children: Vec<ScopeId>,
    #[allow(dead_code)]
    parent: Option<ScopeId>,
    disposed: bool,
}

pub struct Runtime {
    effects: Vec<Option<EffectFn>>,
    effect_cleanups: Vec<Vec<CleanupFn>>,
    effect_disposed: Vec<bool>,
    current_effect: Option<EffectId>,
    pending_effects: VecDeque<EffectId>,
    is_batching: bool,
    scopes: Vec<Scope>,
    current_scope: Option<ScopeId>,
}

impl Runtime {
    pub fn new() -> Self {
        let root_scope = Scope {
            effects: Vec::new(),
            children: Vec::new(),
            parent: None,
            disposed: false,
        };
        Self {
            effects: Vec::new(),
            effect_cleanups: Vec::new(),
            effect_disposed: Vec::new(),
            current_effect: None,
            pending_effects: VecDeque::new(),
            is_batching: false,
            scopes: vec![root_scope],
            current_scope: Some(0),
        }
    }

    pub fn current_effect(&self) -> Option<EffectId> {
        self.current_effect
    }

    pub fn register_effect(&mut self, f: impl Fn() + 'static) -> EffectId {
        let id = self.effects.len();
        self.effects.push(Some(Rc::new(f)));
        self.effect_cleanups.push(Vec::new());
        self.effect_disposed.push(false);

        if let Some(scope_id) = self.current_scope {
            if scope_id < self.scopes.len() {
                self.scopes[scope_id].effects.push(id);
            }
        }

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

    pub fn clone_effect(&self, id: EffectId) -> Option<EffectFn> {
        if self.is_effect_disposed(id) {
            return None;
        }
        self.effects.get(id).and_then(|e| e.as_ref().map(Rc::clone))
    }

    pub fn schedule_effect(&mut self, id: EffectId) {
        if self.is_effect_disposed(id) {
            return;
        }
        if !self.pending_effects.contains(&id) {
            self.pending_effects.push_back(id);
        }
    }

    pub fn is_batching(&self) -> bool {
        self.is_batching
    }

    pub fn pop_pending_effect(&mut self) -> Option<EffectId> {
        loop {
            match self.pending_effects.pop_front() {
                Some(id) if self.is_effect_disposed(id) => continue,
                other => return other,
            }
        }
    }

    pub fn start_batch(&mut self) -> bool {
        let was_batching = self.is_batching;
        self.is_batching = true;
        was_batching
    }

    pub fn end_batch(&mut self, was_batching: bool) {
        self.is_batching = was_batching;
    }

    pub fn is_effect_disposed(&self, id: EffectId) -> bool {
        self.effect_disposed.get(id).copied().unwrap_or(true)
    }

    pub fn create_scope(&mut self) -> ScopeId {
        let id = self.scopes.len();
        let parent = self.current_scope;
        self.scopes.push(Scope {
            effects: Vec::new(),
            children: Vec::new(),
            parent,
            disposed: false,
        });
        if let Some(parent_id) = parent {
            if parent_id < self.scopes.len() {
                self.scopes[parent_id].children.push(id);
            }
        }
        self.current_scope = Some(id);
        id
    }

    pub fn set_current_scope(&mut self, scope: Option<ScopeId>) -> Option<ScopeId> {
        let prev = self.current_scope;
        self.current_scope = scope;
        prev
    }

    pub fn dispose_scope(&mut self, scope_id: ScopeId) {
        if scope_id >= self.scopes.len() || self.scopes[scope_id].disposed {
            return;
        }

        let children: Vec<ScopeId> = self.scopes[scope_id].children.clone();
        for child_id in children {
            self.dispose_scope(child_id);
        }

        let effects: Vec<EffectId> = self.scopes[scope_id].effects.clone();
        for effect_id in effects {
            self.dispose_effect(effect_id);
        }

        self.scopes[scope_id].disposed = true;
    }

    fn dispose_effect(&mut self, effect_id: EffectId) {
        if effect_id >= self.effect_disposed.len() {
            return;
        }
        self.run_cleanups(effect_id);
        self.effect_disposed[effect_id] = true;
        if let Some(slot) = self.effects.get_mut(effect_id) {
            *slot = None;
        }
    }

    pub fn add_cleanup(&mut self, f: impl FnOnce() + 'static) {
        if let Some(effect_id) = self.current_effect {
            if effect_id < self.effect_cleanups.len() {
                self.effect_cleanups[effect_id].push(Box::new(f));
            }
        }
    }

    pub fn run_cleanups(&mut self, effect_id: EffectId) {
        if effect_id < self.effect_cleanups.len() {
            let cleanups = std::mem::take(&mut self.effect_cleanups[effect_id]);
            for cleanup in cleanups {
                cleanup();
            }
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
