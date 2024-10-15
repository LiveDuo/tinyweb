
use std::sync::Mutex;
use std::collections::HashMap;

use crate::js::JsFunction;
use crate::bindings::utils::*;

thread_local! {
    static ANIMATION_FRAME_HANDLERS: Mutex<HashMap<u32, Box<dyn FnMut() + 'static>>> = Default::default();
}

#[no_mangle]
pub extern "C" fn web_one_time_empty_handler(id: i64) {
    ANIMATION_FRAME_HANDLERS.with(|h| {
        if let Some(mut handler) = h.lock().unwrap().remove(&(id as u32)) {
            handler();
        }
    });
}

pub fn request_animation_frame(handler: impl FnMut() + 'static) {
    let function_handle = JsFunction::register(r#"
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
    ANIMATION_FRAME_HANDLERS.with(|h| {
        h.lock().unwrap().insert(function_handle as u32, Box::new(handler));
    });
}

pub fn set_timeout(
    handler: impl FnMut() + 'static,
    ms: impl Into<f64>,
) -> f64 {
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
    .invoke_and_return_object(&[ms.into().into()]);
    let function_handle = get_property_i64(&obj_handle, "id");
    let timer_handle = get_property_f64(&obj_handle, "handle");
    ANIMATION_FRAME_HANDLERS.with(|h| {
        h.lock().unwrap().insert(function_handle as u32, Box::new(handler));
    });
    timer_handle
}

pub fn clear_timeout(interval_id: impl Into<f64>) {
    let clear_interval = JsFunction::register(r#"
        function(interval_id){
            window.clearTimeout(interval_id);
        }"#);
    clear_interval.invoke(&[interval_id.into().into()]);
}
