
use crate::utils::js::register_function;
use crate::utils::handlers::FunctionHandle;
use crate::utils::js::ExternRef;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub fn history_push_state(title: &str, url: &str) {
    register_function("
        function(title, url) {
            window.history.pushState({}, title, url);
        }
        ")
    .invoke(&[title.into(), url.into()]);
}

pub fn history_replace_state(title: &str, url: &str) {
    register_function("
        function(title, url) {
            window.history.replaceState({}, title, url);
        }
        ")
    .invoke(&[title.into(), url.into()]);
}

pub fn history_back() {
    register_function("
        function() {
            window.history.back();
        }
        ")
    .invoke(&[]);
}

pub fn history_forward() {
    register_function("
        function() {
            window.history.forward();
        }
        ")
    .invoke(&[]);
}

pub fn history_go(delta: i32) {
    register_function("
        function(delta) {
            window.history.go(delta);
        }
        ")
    .invoke(&[delta.into()]);
}

pub fn history_length() -> usize {
    register_function("
        function() {
            return window.history.length;
        }
        ")
    .invoke(&[]) as usize
}

pub fn location_url() -> String {
    register_function("
        function() {
            return window.location.href;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_host() -> String {
    register_function("
        function() {
            return window.location.host;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_hostname() -> String {
    register_function("
        function() {
            return window.location.hostname;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_pathname() -> String {
    register_function("
        function() {
            return window.location.pathname;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_search() -> String {
    register_function("
        function() {
            return window.location.search;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_hash() -> String {
    register_function("
        function() {
            return window.location.hash;
        }
        ")
    .invoke_and_return_string(&[])
}

pub fn location_reload() {
    register_function("
        function() {
            window.location.reload();
        }
        ")
    .invoke(&[]);
}

pub struct PopStateEvent {}

static HISTORY_POP_STATE_EVENT_HANDLERS: Mutex<
    Option<HashMap<Arc<FunctionHandle>, Box<dyn FnMut(PopStateEvent) + Send + 'static>>>,
> = Mutex::new(None);

fn add_history_pop_state_event_handler(
    id: Arc<FunctionHandle>,
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

fn remove_history_pop_state_event_handler(id: &Arc<FunctionHandle>) {
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
            if key.0.value == id {
                handler(PopStateEvent {});
            }
        }
    }
}

pub fn add_history_pop_state_event_listener(
    handler: impl FnMut(PopStateEvent) + Send + 'static,
) -> Arc<FunctionHandle> {
    let function_ref = register_function(r#"
        function(){
            const handler = (e) => {
                this.module.instance.exports.web_handle_history_pop_state_event(id);
            };
            const id = this.storeObject(handler);
            window.addEventListener("popstate",handler);
            return id;
        }"#)
    .invoke_and_return_bigint(&[]);
    let function_handle = Arc::new(FunctionHandle(ExternRef {
        value: function_ref,
    }));
    add_history_pop_state_event_handler(function_handle.clone(), Box::new(handler));
    function_handle
}

pub fn remove_history_pop_state_listener(
    element: &ExternRef,
    function_handle: &Arc<FunctionHandle>,
) {
    register_function(r#"
        function(element, f){
            window.removeEventListener("popstate", f);
        }"#)
    .invoke(&[element.into(), (&(function_handle.0)).into()]);
    remove_history_pop_state_event_handler(function_handle);
}
