use std::collections::HashMap;
use std::sync::Arc;
use crate::core::VideoAction;

pub struct ActionFactory {
    actions: HashMap<String, Arc<dyn VideoAction>>,
}

impl ActionFactory {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub fn register(&mut self, action: impl VideoAction + 'static) {
        self.actions.insert(action.id().to_string(), Arc::new(action));
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn VideoAction>> {
        self.actions.get(id).cloned()
    }
    
    pub fn list_actions(&self) -> Vec<String> {
        self.actions.keys().cloned().collect()
    }
}
