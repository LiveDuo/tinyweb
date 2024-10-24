
use std::rc::Rc;
use std::sync::Mutex;
use std::collections::HashMap;

use crate::js::{ExternRef, InvokeParam};

use crate::bindings::utils::{get_property_i32, get_property_f64};
use crate::allocations::get_string_from_allocation;


pub fn console_log(message: &str) {
    let code = "function(message){ console.log(message); }";
    crate::js::invoke_and_return(code, &[InvokeParam::String(message)]);
}

pub fn console_error(message: &str) {
    let code = "function(message){ console.error(message); }";
    crate::js::invoke_and_return(code, &[InvokeParam::String(message)]);
}

pub fn console_warn(message: &str) {
    let code = "function(message){ console.warn(message); }";
    crate::js::invoke_and_return(code, &[InvokeParam::String(message)]);
}

pub fn console_time(label: &str) {
    let code = "function(label){ console.time(label); }";
    crate::js::invoke_and_return(code, &[InvokeParam::String(label)]);
}

pub fn console_time_end(label: &str) {
    let code = "function(label){ console.timeEnd(label); }";
    crate::js::invoke_and_return(code, &[InvokeParam::String(label)]);
}


pub fn local_storage_set(key: &str, value: &str) {
    let code = "function(key, value){ localStorage.setItem(key, value); }";
    crate::js::invoke_and_return(code, &[InvokeParam::String(key), InvokeParam::String(value)]);
}

pub fn local_storage_remove(key: &str) {
    let code = "function(key){ localStorage.removeItem(key); }";
    crate::js::invoke_and_return(code, &[InvokeParam::String(key)]);
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
    let text_allocation_id = crate::js::invoke_and_return_number(code, &[InvokeParam::String(key)]);
    if text_allocation_id == 0 {
        return None;
    }
    let text = get_string_from_allocation(text_allocation_id as u32);
    Some(text)
}

pub fn local_storage_clear() {
    let code = "function(){ localStorage.clear(); }";
    crate::js::invoke_and_return(code, &[]);
}


thread_local! {
    static TIMEOUT_HANDLERS: Mutex<HashMap<ExternRef, Box<dyn FnMut() + 'static>>> = Default::default();
}

#[no_mangle]
pub fn handle_one_time_empty_callback(callback_id: u32, _allocation_id: u32) {
    TIMEOUT_HANDLERS.with(|h| {
        h.lock().map(|mut h| {
            let function_handle = ExternRef { value: callback_id as u32 };
            let mut handler = h.remove(&function_handle).unwrap();
            handler();
        }).unwrap();
    });
}

pub fn set_timeout(handler: impl FnMut() + 'static, ms: impl Into<f64>) -> f64 {
    let code = r#"
        function(ms){
            const handler = () => {
                wasmModule.instance.exports.handle_one_time_empty_callback(objectId,0);
            };
            objects.push(handler);
            const objectId = objects.length - 1;
            const handle = window.setTimeout(handler, ms);
            return {objectId,handle};
        }"#;
    let obj_handle = crate::js::invoke_and_return_ref(code, &[InvokeParam::Float64(ms.into())]);
    let function_id = get_property_i32(&obj_handle, "objectId");
    let timer_handle = get_property_f64(&obj_handle, "handle");
    TIMEOUT_HANDLERS.with(|h| {
        h.lock().map(|mut s| {
            let function_handle = ExternRef { value: function_id as u32 };
            s.insert(function_handle, Box::new(handler));
        }).unwrap();
    });
    timer_handle
}

pub fn clear_timeout(interval_id: impl Into<f64>) {
    let code = "function(interval_id){ window.clearTimeout(interval_id); }";
    crate::js::invoke_and_return(code, &[InvokeParam::Float64(interval_id.into())]);
}

pub fn history_push_state(title: &str, url: &str) {
    let code = "function(title, url) { window.history.pushState({}, title, url); }";
    crate::js::invoke_and_return(code, &[InvokeParam::String(title), InvokeParam::String(url)]);
}

pub fn history_replace_state(title: &str, url: &str) {
    let code = "function(title, url) { window.history.replaceState({}, title, url); }";
    crate::js::invoke_and_return(code, &[InvokeParam::String(title), InvokeParam::String(url)]);
}

pub fn history_back() {
    let code = "function() { window.history.back(); }";
    crate::js::invoke_and_return(code, &[]);
}

pub fn history_forward() {
    let code = "function() { window.history.forward(); }";
    crate::js::invoke_and_return(code, &[]);
}

pub fn history_go(delta: i32) {
    let code = "function(delta) { window.history.go(delta); }";
    crate::js::invoke_and_return(code, &[InvokeParam::Float64(delta as f64)]);
}

pub fn history_length() -> u32 {
    let code = "function() { return window.history.length; }";
    crate::js::invoke_and_return_number(code, &[]) as u32
}

pub fn location_url() -> String {
    let code = "function() { return window.location.href; }";
    crate::js::invoke_and_return_string(code, &[])
}

pub fn location_host() -> String {
    let code = "function() { return window.location.host; }";
    crate::js::invoke_and_return_string(code, &[])
}

pub fn location_hostname() -> String {
    let code = "function() { return window.location.hostname; }";
    crate::js::invoke_and_return_string(code, &[])
}

pub fn location_pathname() -> String {
    let code = "function() { return window.location.pathname; }";
    crate::js::invoke_and_return_string(code, &[])
}

pub fn location_search() -> String {
    let code = "function() { return window.location.search; }";
    crate::js::invoke_and_return_string(code, &[])
}

pub fn location_hash() -> String {
    let code = "function() { return window.location.hash; }";
    crate::js::invoke_and_return_string(code, &[])
}

pub fn location_reload() {
    let code = "function() { window.location.reload(); }";
    crate::js::invoke_and_return(code, &[]);
}

thread_local! {
    static HISTORY_POP_STATE_HANDLERS: Mutex<HashMap<ExternRef, Box<dyn FnMut() + 'static>>> = Default::default();
}

#[no_mangle]
pub fn handle_pop_state_event_callback(callback_id: u32, _allocation_id: u32) {
    HISTORY_POP_STATE_HANDLERS.with(|s| {

        let handler = s.lock().map(|mut s| {
            let (_, handler) = s.iter_mut().find(|(s, _)| s.value == callback_id).unwrap();
            handler as *mut Box<dyn FnMut() + 'static>
        }).unwrap();

        unsafe { (*handler)() }
    });
}

pub fn add_history_pop_state_event_listener(handler: impl FnMut() + 'static) -> ExternRef {
    let code = r#"
        function(){
            const handler = (e) => {
                wasmModule.instance.exports.handle_pop_state_event_callback(objectId,0);
            };
            objects.push(handler);
            const objectId = objects.length - 1;
            window.addEventListener("popstate",handler);
            return objectId;
        }"#;
    let function_ref = crate::js::invoke_and_return_number(code, &[]);
    let function_handle = ExternRef { value: function_ref as u32, };
    HISTORY_POP_STATE_HANDLERS.with(|s| {
        s.lock().map(|mut s| { s.insert(function_handle.clone(), Box::new(handler)); }).unwrap();
    });
    function_handle
}

pub fn remove_history_pop_state_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ window.removeEventListener('popstate', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    HISTORY_POP_STATE_HANDLERS.with(|s| {
        s.lock().map(|mut h| { h.remove(function_handle); }).unwrap();
    });
}
