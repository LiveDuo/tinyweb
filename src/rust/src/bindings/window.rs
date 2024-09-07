
use std::cell::RefCell;
use std::collections::HashMap;

use crate::js::JSFunction;
use crate::bindings::util::*;

thread_local! {
    static ANIMATION_FRAME_EVENT_HANDLERS: RefCell<Option<HashMap<i64, Box<dyn FnMut() + 'static>>>> = RefCell::new(None);
}

#[no_mangle]
pub extern "C" fn web_one_time_empty_handler(id: i64) {
    let mut c = None;
    {
        ANIMATION_FRAME_EVENT_HANDLERS.with_borrow_mut(|h| {
            if let Some(h) = h.as_mut() {
                if let Some(handler) = h.remove(&id) {
                    c = Some(handler);
                }
            }
        });
    }
    if let Some(mut c) = c {
        c();
    }
}

pub fn request_animation_frame(handler: impl FnMut() + 'static) {
    let function_handle = JSFunction::register(r#"
        function(){
            const handler = () => {
                wasmModule.instance.exports.web_one_time_empty_handler(id);
                deallocate(id);
            };
            const id = allocate(handler);
            requestAnimationFrame(handler);
            return id;
        }"#)
    .invoke_and_return_bigint(&[]);
    ANIMATION_FRAME_EVENT_HANDLERS.with_borrow_mut(|h| {
        if h.is_none() {
            *h = Some(HashMap::new());
        }
        h.as_mut()
            .unwrap()
            .insert(function_handle, Box::new(handler));
    });
}

pub fn set_timeout(
    handler: impl FnMut() + 'static,
    ms: impl Into<f64>,
) -> f64 {
    let obj_handle = JSFunction::register(r#"
        function(ms){
            const handler = () => {
                wasmModule.instance.exports.web_one_time_empty_handler(id);
                deallocate(id);
            };
            const id = allocate(handler);
            const handle = window.setTimeout(handler, ms);
            return {id,handle};
        }"#)
    .invoke_and_return_object(&[ms.into().into()]);
    let function_handle = get_property_i64(&obj_handle, "id");
    let timer_handle = get_property_f64(&obj_handle, "handle");
    ANIMATION_FRAME_EVENT_HANDLERS.with_borrow_mut(|h| {
        if h.is_none() {
            *h = Some(HashMap::new());
        }
        h.as_mut()
            .unwrap()
            .insert(function_handle, Box::new(handler));
    });
    timer_handle
}

pub fn clear_timeout(interval_id: impl Into<f64>) {
    let clear_interval = JSFunction::register(r#"
        function(interval_id){
            window.clearTimeout(interval_id);
        }"#);
    clear_interval.invoke(&[interval_id.into().into()]);
}
