use crate::ExternRef;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Mutex};

pub trait CustomElement {
    fn construct(&mut self, component_ref: ExternRef);
    fn connected_callback(&mut self);
    fn disconnected_callback(&mut self);
    fn adopted_callback(&mut self);
    fn attribute_changed_callback(
        &mut self,
        name: String,
        old_value: Option<String>,
        new_value: Option<String>,
    );
    fn observed_attributes(&self) -> Vec<&'static str> {
        vec![]
    }
}

static CUSTOM_COMPONENT_STATE: Lazy<Mutex<HashMap<i64, Box<dyn CustomElement + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn add_custom_component(id: i64, component: Box<dyn CustomElement + Send + Sync>) {
    let mut state = CUSTOM_COMPONENT_STATE.lock().unwrap();
    state.insert(id, component);
}

pub fn remove_custom_component(id: i64) {
    let mut state = CUSTOM_COMPONENT_STATE.lock().unwrap();
    state.remove(&id);
}

pub fn custom_element_define<T>(_tag_name: &str)
where
    T: CustomElement + Into<ExternRef> + Default,
{
    todo!()
}
