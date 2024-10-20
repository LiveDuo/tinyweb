
use std::rc::Rc;
use std::sync::Mutex;
use std::collections::HashMap;

use crate::js::{ExternRef, InvokeParam, JsFunction};

use crate::bindings::utils::{get_property_f64, get_property_i64};
use crate::allocations::get_string_from_allocation;


pub fn console_log(message: &str) {
    let console_log = JsFunction::register(r#"
        function(message){
            console.log(message);
        }"#);
    console_log.invoke(&[InvokeParam::String(message)]);
}

pub fn console_error(message: &str) {
    let console_error = JsFunction::register(r#"
        function(message){
            console.error(message);
        }"#);
    console_error.invoke(&[InvokeParam::String(message)]);
}

pub fn console_warn(message: &str) {
    let console_warn = JsFunction::register(r#"
        function(message){
            console.warn(message);
        }"#);
    console_warn.invoke(&[InvokeParam::String(message)]);
}

pub fn console_time(label: &str) {
    let console_time = JsFunction::register(r#"
        function(label){
            console.time(label);
        }"#);
    console_time.invoke(&[InvokeParam::String(label)]);
}

pub fn console_time_end(label: &str) {
    let console_time_end = JsFunction::register(r#"
        function(label){
            console.timeEnd(label);
        }"#);
    console_time_end.invoke(&[InvokeParam::String(label)]);
}


pub fn local_storage_set(key: &str, value: &str) {
    let local_storage_set = JsFunction::register(r#"
        function(key, value){
            localStorage.setItem(key, value);
        }"#);
    local_storage_set.invoke(&[InvokeParam::String(key), InvokeParam::String(value)]);
}

pub fn local_storage_remove(key: &str) {
    let local_storage_remove = JsFunction::register(r#"
        function(key){
            localStorage.removeItem(key);
        }"#);
    local_storage_remove.invoke(&[InvokeParam::String(key)]);
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
    let text_allocation_id = local_storage_get.invoke(&[InvokeParam::String(key)]);
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
    let obj_handle = JsFunction::register(r#"
        function(ms){
            const handler = () => {
                wasmModule.instance.exports.web_one_time_empty_handler(id);
                deallocate(id);
            };
            const id = allocate(handler);
            const handle = window.setTimeout(handler, ms);
            return {id,handle};
        }"#)
    .invoke_and_return_object(&[InvokeParam::Float64(ms.into())]);
    let function_handle = get_property_i64(&obj_handle, "id");
    let timer_handle = get_property_f64(&obj_handle, "handle");
    TIMEOUT_HANDLERS.with(|h| {
        h.lock().unwrap().insert(function_handle as u32, Box::new(handler));
    });
    timer_handle
}

pub fn clear_timeout(interval_id: impl Into<f64>) {
    let clear_interval = JsFunction::register(r#"
        function(interval_id){
            window.clearTimeout(interval_id);
        }"#);
    clear_interval.invoke(&[InvokeParam::Float64(interval_id.into())]);
}

pub fn history_push_state(title: &str, url: &str) {
    JsFunction::register("
        function(title, url) {
            window.history.pushState({}, title, url);
        }
        ")
    .invoke(&[InvokeParam::String(title), InvokeParam::String(url)]);
}

pub fn history_replace_state(title: &str, url: &str) {
    JsFunction::register("
        function(title, url) {
            window.history.replaceState({}, title, url);
        }
        ")
    .invoke(&[InvokeParam::String(title), InvokeParam::String(url)]);
}

pub fn history_back() {
    JsFunction::register("
        function() {
            window.history.back();
        }
        ")
    .invoke(&[]);
}

pub fn history_forward() {
    JsFunction::register("
        function() {
            window.history.forward();
        }
        ")
    .invoke(&[]);
}

pub fn history_go(delta: i32) {
    JsFunction::register("
        function(delta) {
            window.history.go(delta);
        }
        ")
    .invoke(&[InvokeParam::Float64(delta as f64)]);
}

pub fn history_length() -> u32 {
    JsFunction::register("
        function() {
            return window.history.length;
        }
        ")
    .invoke(&[]) as u32
}

pub fn location_url() -> String {
    JsFunction::register("
        function() {
            return window.location.href;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_host() -> String {
    JsFunction::register("
        function() {
            return window.location.host;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_hostname() -> String {
    JsFunction::register("
        function() {
            return window.location.hostname;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_pathname() -> String {
    JsFunction::register("
        function() {
            return window.location.pathname;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_search() -> String {
    JsFunction::register("
        function() {
            return window.location.search;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_hash() -> String {
    JsFunction::register("
        function() {
            return window.location.hash;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_reload() {
    JsFunction::register("
        function() {
            window.location.reload();
        }
        ")
    .invoke(&[]);
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
    let function_ref = JsFunction::register(r#"
        function(){
            const handler = (e) => {
                wasmModule.instance.exports.web_handle_history_pop_state_event(id);
            };
            const id = allocate(handler);
            window.addEventListener("popstate",handler);
            return id;
        }"#)
    .invoke_and_return_bigint(&[]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u32, });
    add_history_pop_state_event_handler(function_handle.clone(), Box::new(handler));
    function_handle
}

pub fn remove_history_pop_state_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    JsFunction::register(r#"
        function(element, f){
            window.removeEventListener("popstate", f);
        }"#)
    .invoke(&[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    remove_history_pop_state_event_handler(function_handle);
}
