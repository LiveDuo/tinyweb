
use crate::js::{ExternRef, JSFunction};
use crate::params::InvokeParam;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

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

static HISTORY_POP_STATE_EVENT_HANDLERS: Mutex<
    Option<HashMap<Arc<ExternRef>, Box<dyn FnMut(PopStateEvent) + Send + 'static>>>,
> = Mutex::new(None);

fn add_history_pop_state_event_handler(
    id: Arc<ExternRef>,
    handler: Box<dyn FnMut(PopStateEvent) + Send + 'static>,
) {
    let mut handlers = HISTORY_POP_STATE_EVENT_HANDLERS.lock().unwrap();
    if let Some(h) = handlers.as_mut() {
        h.insert(id, handler);
    } else {
        let mut h = HashMap::new();
        h.insert(id, handler);
        *handlers = Some(h);
    }
}

fn remove_history_pop_state_event_handler(id: &Arc<ExternRef>) {
    let mut handlers = HISTORY_POP_STATE_EVENT_HANDLERS.lock().unwrap();
    if let Some(h) = handlers.as_mut() {
        h.remove(id);
    }
}

#[no_mangle]
pub extern "C" fn web_handle_history_pop_state_event(id: i64) {
    let mut handlers = HISTORY_POP_STATE_EVENT_HANDLERS.lock().unwrap();
    if let Some(h) = handlers.as_mut() {
        for (key, handler) in h.iter_mut() {
            if key.value == id {
                handler(PopStateEvent {});
            }
        }
    }
}

pub fn add_history_pop_state_event_listener(
    handler: impl FnMut(PopStateEvent) + Send + 'static,
) -> Arc<ExternRef> {
    let function_ref = JSFunction::register(r#"
        function(){
            const handler = (e) => {
                this.module.instance.exports.web_handle_history_pop_state_event(id);
            };
            const id = this.storeObject(handler);
            window.addEventListener("popstate",handler);
            return id;
        }"#)
    .invoke_and_return_bigint(&[]);
    let function_handle = Arc::new(ExternRef { value: function_ref, });
    add_history_pop_state_event_handler(function_handle.clone(), Box::new(handler));
    function_handle
}

pub fn remove_history_pop_state_listener(
    element: &ExternRef,
    function_handle: &Arc<ExternRef>,
) {
    JSFunction::register(r#"
        function(element, f){
            window.removeEventListener("popstate", f);
        }"#)
    .invoke(&[element.into(), InvokeParam::ExternRef(&function_handle)]);
    remove_history_pop_state_event_handler(function_handle);
}
