
use crate::js::{ExternRef, InvokeParam, JsFunction};

use std::collections::HashMap;
use std::sync::Mutex;
use std::rc::Rc;

pub fn history_push_state(title: &str, url: &str) {
    JsFunction::register("
        function(title, url) {
            window.history.pushState({}, title, url);
        }
        ")
    .invoke(&[title.into(), url.into()]);
}

pub fn history_replace_state(title: &str, url: &str) {
    JsFunction::register("
        function(title, url) {
            window.history.replaceState({}, title, url);
        }
        ")
    .invoke(&[title.into(), url.into()]);
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
    .invoke(&[delta.into()]);
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
    .invoke(&[element.into(), InvokeParam::ExternRef(&function_handle)]);
    remove_history_pop_state_event_handler(function_handle);
}
