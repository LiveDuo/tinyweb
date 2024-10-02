
use crate::js::{ExternRef, InvokeParam, JSFunction};

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

pub fn history_push_state(title: &str, url: &str) {
    JSFunction::register("
        function(title, url) {
            window.history.pushState({}, title, url);
        }
        ")
    .invoke(&[title.into(), url.into()]);
}

pub fn history_replace_state(title: &str, url: &str) {
    JSFunction::register("
        function(title, url) {
            window.history.replaceState({}, title, url);
        }
        ")
    .invoke(&[title.into(), url.into()]);
}

pub fn history_back() {
    JSFunction::register("
        function() {
            window.history.back();
        }
        ")
    .invoke(&[]);
}

pub fn history_forward() {
    JSFunction::register("
        function() {
            window.history.forward();
        }
        ")
    .invoke(&[]);
}

pub fn history_go(delta: i32) {
    JSFunction::register("
        function(delta) {
            window.history.go(delta);
        }
        ")
    .invoke(&[delta.into()]);
}

pub fn history_length() -> usize {
    JSFunction::register("
        function() {
            return window.history.length;
        }
        ")
    .invoke(&[]) as usize
}

pub fn location_url() -> String {
    JSFunction::register("
        function() {
            return window.location.href;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_host() -> String {
    JSFunction::register("
        function() {
            return window.location.host;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_hostname() -> String {
    JSFunction::register("
        function() {
            return window.location.hostname;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_pathname() -> String {
    JSFunction::register("
        function() {
            return window.location.pathname;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_search() -> String {
    JSFunction::register("
        function() {
            return window.location.search;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_hash() -> String {
    JSFunction::register("
        function() {
            return window.location.hash;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_reload() {
    JSFunction::register("
        function() {
            window.location.reload();
        }
        ")
    .invoke(&[]);
}

pub struct PopStateEvent {}

thread_local! {
    pub static HISTORY_POP_STATE_EVENT_HANDLERS: RefCell<Option<HashMap<Rc<ExternRef>, Box<dyn FnMut(PopStateEvent) + 'static>>>> = Default::default();
}

fn add_history_pop_state_event_handler(
    id: Rc<ExternRef>,
    handler: Box<dyn FnMut(PopStateEvent) + 'static>,
) {
    HISTORY_POP_STATE_EVENT_HANDLERS.with_borrow_mut(|s| {
        if let Some(h) = s {
            h.insert(id, handler);
        } else {
            let mut h = HashMap::new();
            h.insert(id, handler);
            *s = Some(h);
        }
    });
}

fn remove_history_pop_state_event_handler(id: &Rc<ExternRef>) {
    HISTORY_POP_STATE_EVENT_HANDLERS.with_borrow_mut(|s| {
        if let Some(h) = s {
            h.remove(id);
        }
    });
}

#[no_mangle]
pub extern "C" fn web_handle_history_pop_state_event(id: i64) {
    HISTORY_POP_STATE_EVENT_HANDLERS.with_borrow_mut(|s| {
        if let Some(h) = s {
            for (key, handler) in h.iter_mut() {
                if key.value == id {
                    handler(PopStateEvent {});
                }
            }
        }
    });
}

pub fn add_history_pop_state_event_listener(
    handler: impl FnMut(PopStateEvent) + 'static,
) -> Rc<ExternRef> {
    let function_ref = JSFunction::register(r#"
        function(){
            const handler = (e) => {
                _wasmModule.instance.exports.web_handle_history_pop_state_event(id);
            };
            const id = allocate(handler);
            window.addEventListener("popstate",handler);
            return id;
        }"#)
    .invoke_and_return_bigint(&[]);
    let function_handle = Rc::new(ExternRef { value: function_ref, });
    add_history_pop_state_event_handler(function_handle.clone(), Box::new(handler));
    function_handle
}

pub fn remove_history_pop_state_listener(
    element: &ExternRef,
    function_handle: &Rc<ExternRef>,
) {
    JSFunction::register(r#"
        function(element, f){
            window.removeEventListener("popstate", f);
        }"#)
    .invoke(&[element.into(), InvokeParam::ExternRef(&function_handle)]);
    remove_history_pop_state_event_handler(function_handle);
}
