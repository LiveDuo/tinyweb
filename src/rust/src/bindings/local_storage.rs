
use crate::js::JsFunction;
use crate::allocations::get_string_from_allocation;

pub fn local_storage_set(key: &str, value: &str) {
    let local_storage_set = JsFunction::register(r#"
        function(key, value){
            localStorage.setItem(key, value);
        }"#);
    local_storage_set.invoke(&[key.into(), value.into()]);
}

pub fn local_storage_remove(key: &str) {
    let local_storage_remove = JsFunction::register(r#"
        function(key){
            localStorage.removeItem(key);
        }"#);
    local_storage_remove.invoke(&[key.into()]);
}

pub fn local_storage_get(key: &str) -> Option<String> {
    let local_storage_get = JsFunction::register(r#"
        function(key){
            const text = localStorage.getItem(key);
            if(text === null){
                return 0;
            }
            const allocationId = writeStringToMemory(text);
            return allocationId;
        }"#);
    let text_allocation_id = local_storage_get.invoke(&[key.into()]);
    if text_allocation_id == 0 {
        return None;
    }
    let text = get_string_from_allocation(text_allocation_id as u32);
    Some(text)
}

pub fn local_storage_clear() {
    let local_storage_clear = JsFunction::register(r#"
        function(){
            localStorage.clear();
        }"#);
    local_storage_clear.invoke(&[]);
}
