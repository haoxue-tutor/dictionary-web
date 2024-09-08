use haoxue_dict::Dictionary;
use leptos::*;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub struct DictContext {
    dict: RwSignal<Option<Arc<Dictionary>>>,
}

impl Default for DictContext {
    fn default() -> Self {
        Self {
            dict: create_rw_signal(None),
        }
    }
}

impl DictContext {
    pub fn get(&self) -> Option<Arc<Dictionary>> {
        self.dict.get()
    }

    pub fn set(&self, dict: Dictionary) {
        self.dict.set(Some(Arc::new(dict)));
    }

    pub fn provide_context() {
        provide_context(Self::default());
    }

    pub fn use_context() -> Self {
        use_context::<Self>().expect("DictContext should be provided")
    }
}
