
use std::rc::Rc;
use std::sync::Mutex;
use std::collections::HashMap;

use crate::js::{ExternRef, InvokeParam, JsFunction};

use crate::bindings::utils::{get_property_f64, get_property_i64};
use crate::allocations::get_string_from_allocation;


pub fn console_log(message: &str) {
    let code = "function(message){ console.log(message); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::String(message)]);
}

pub fn console_error(message: &str) {
    let code = "function(message){ console.error(message); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::String(message)]);
}

pub fn console_warn(message: &str) {
    let code = "function(message){ console.warn(message); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::String(message)]);
}

pub fn console_time(label: &str) {
    let code = "function(label){ console.time(label); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::String(label)]);
}

pub fn console_time_end(label: &str) {
    let code = "function(label){ console.timeEnd(label); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::String(label)]);
}


pub fn local_storage_set(key: &str, value: &str) {
    let code = "function(key, value){ localStorage.setItem(key, value); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::String(key), InvokeParam::String(value)]);
}

pub fn local_storage_remove(key: &str) {
    let code = "function(key){ localStorage.removeItem(key); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::String(key)]);
}

pub fn local_storage_get(key: &str) -> Option<String> {
    let code = r#"
        function(key){
            const text = localStorage.getItem(key);
            if(text === null){
                return 0;
            }
            const buffer = (new TextEncoder()).encode(text);
            const allocationId = writeBufferToMemory(buffer);
            return allocationId;
        }"#;
    let text_allocation_id = JsFunction::invoke_and_return(code, &[InvokeParam::String(key)]);
    if text_allocation_id == 0 {
        return None;
    }
    let text = get_string_from_allocation(text_allocation_id as u32);
    Some(text)
}

pub fn local_storage_clear() {
    let code = "function(){ localStorage.clear(); }";
    JsFunction::invoke_and_return(code, &[]);
}


thread_local! {
    static TIMEOUT_HANDLERS: Mutex<HashMap<u32, Box<dyn FnMut() + 'static>>> = Default::default();
}

#[no_mangle]
pub extern "C" fn web_one_time_empty_handler(id: i64) {
    TIMEOUT_HANDLERS.with(|h| {
        if let Some(mut handler) = h.lock().unwrap().remove(&(id as u32)) {
            handler();
        }
    });
}

pub fn set_timeout(handler: impl FnMut() + 'static, ms: impl Into<f64>) -> f64 {
    let code = r#"
        function(ms){
            const handler = () => {
                wasmModule.instance.exports.web_one_time_empty_handler(id);
                deallocate(id);
            };
            const id = allocate(handler);
            const handle = window.setTimeout(handler, ms);
            return {id,handle};
        }"#;
    let obj_handle = JsFunction::invoke_and_return_object(code, &[InvokeParam::Float64(ms.into())]);
    let function_handle = get_property_i64(&obj_handle, "id");
    let timer_handle = get_property_f64(&obj_handle, "handle");
    TIMEOUT_HANDLERS.with(|h| {
        h.lock().unwrap().insert(function_handle as u32, Box::new(handler));
    });
    timer_handle
}

pub fn clear_timeout(interval_id: impl Into<f64>) {
    let code = "function(interval_id){ window.clearTimeout(interval_id); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::Float64(interval_id.into())]);
}

pub fn history_push_state(title: &str, url: &str) {
    let code = "function(title, url) { window.history.pushState({}, title, url); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::String(title), InvokeParam::String(url)]);
}

pub fn history_replace_state(title: &str, url: &str) {
    let code = "function(title, url) { window.history.replaceState({}, title, url); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::String(title), InvokeParam::String(url)]);
}

pub fn history_back() {
    let code = "function() { window.history.back(); }";
    JsFunction::invoke_and_return(code, &[]);
}

pub fn history_forward() {
    let code = "function() { window.history.forward(); }";
    JsFunction::invoke_and_return(code, &[]);
}

pub fn history_go(delta: i32) {
    let code = "function(delta) { window.history.go(delta); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::Float64(delta as f64)]);
}

pub fn history_length() -> u32 {
    let code = "function() { return window.history.length; }";
    JsFunction::invoke_and_return(code, &[]) as u32
}

pub fn location_url() -> String {
    let code = "function() { return window.location.href; }";
    JsFunction::invoke_and_return_string(code, &[])
}

pub fn location_host() -> String {
    let code = "function() { return window.location.host; }";
    JsFunction::invoke_and_return_string(code, &[])
}

pub fn location_hostname() -> String {
    let code = "function() { return window.location.hostname; }";
    JsFunction::invoke_and_return_string(code, &[])
}

pub fn location_pathname() -> String {
    let code = "function() { return window.location.pathname; }";
    JsFunction::invoke_and_return_string(code, &[])
}

pub fn location_search() -> String {
    let code = "function() { return window.location.search; }";
    JsFunction::invoke_and_return_string(code, &[])
}

pub fn location_hash() -> String {
    let code = "function() { return window.location.hash; }";
    JsFunction::invoke_and_return_string(code, &[])
}

pub fn location_reload() {
    let code = "function() { window.location.reload(); }";
    JsFunction::invoke_and_return(code, &[]);
}

pub struct PopStateEvent {}

thread_local! {
    static HISTORY_POP_STATE_HANDLERS: Mutex<HashMap<Rc<ExternRef>, Box<dyn FnMut(PopStateEvent) + 'static>>> = Default::default();
}

fn add_history_pop_state_event_handler(
    id: Rc<ExternRef>,
    handler: Box<dyn FnMut(PopStateEvent) + 'static>,
) {
    HISTORY_POP_STATE_HANDLERS.with(|s| {
        s.lock().unwrap().insert(id, handler);
    });
}

fn remove_history_pop_state_event_handler(id: &Rc<ExternRef>) {
    HISTORY_POP_STATE_HANDLERS.with(|s| {
        s.lock().unwrap().remove(id);
    });
}

#[no_mangle]
pub extern "C" fn web_handle_history_pop_state_event(id: i64) {
    HISTORY_POP_STATE_HANDLERS.with(|s| {

        let handler = s.lock().map(|mut s| {
            let (_, handler) = s.iter_mut().find(|(s, _)| s.value == id as u32).unwrap();
            handler as *mut Box<dyn FnMut(PopStateEvent) + 'static>
        }).unwrap();

        unsafe { (*handler)(PopStateEvent {}) }
    });
}

pub fn add_history_pop_state_event_listener(handler: impl FnMut(PopStateEvent) + 'static) -> Rc<ExternRef> {
    let code = r#"
        function(){
            const handler = (e) => {
                wasmModule.instance.exports.web_handle_history_pop_state_event(id);
            };
            const id = allocate(handler);
            window.addEventListener("popstate",handler);
            return id;
        }"#;
    let function_ref = JsFunction::invoke_and_return_bigint(code, &[]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u32, });
    add_history_pop_state_event_handler(function_handle.clone(), Box::new(handler));
    function_handle
}

pub fn remove_history_pop_state_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ window.removeEventListener('popstate', f); }";
    JsFunction::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    remove_history_pop_state_event_handler(function_handle);
}
