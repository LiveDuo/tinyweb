
use core::future::Future;

use crate::js::{ExternRef, InvokeParam};
use crate::allocations::get_string_from_allocation;
use crate::runtime::EventHandlerFuture;

pub fn random() -> f32 {
    let code = "function(){ return Math.random(); }";
    crate::js::invoke_and_return_number(code, &[]) as f32
}

pub fn get_property_u32(element: &ExternRef, property: &str) -> u32 {
    let code = "function(element, property){ return element[property]; }";
    crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element), InvokeParam::String(property)])
}

pub fn set_property_u32(element: &ExternRef, property: &str, value: u32) {
    let code = "function(element, property, value){ element[property] = value; }";
    crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element), InvokeParam::String(property), InvokeParam::Uint32(value)]);
}

pub fn get_property_i64(element: &ExternRef, property: &str) -> i64 {
    let code = "function(element, property){ return element[property]; }";
    crate::js::invoke_and_return_bigint(code, &[InvokeParam::ExternRef(element), InvokeParam::String(property)])
}

pub fn set_property_i64(element: &ExternRef, property: &str, value: i64) {
    let code = "function(element, property, value){ element[property] = value; }";
    crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element), InvokeParam::String(property), InvokeParam::BigInt(value)]);
}

pub fn get_property_f64(element: &ExternRef, property: &str) -> f64 {
    let code = "function(element, property){ return element[property]; }";
    crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element), InvokeParam::String(property)]) as f64
}

pub fn set_property_f64(element: &ExternRef, property: &str, value: f64) {
    let code = "function(element, property, value){ element[property] = value; }";
    crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element), InvokeParam::String(property), InvokeParam::Float64(value)]);
}

pub fn get_property_bool(element: &ExternRef, property: &str) -> bool {
    let code = "function(element, property){ return element[property]?1:0; }";
    let v = crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element), InvokeParam::String(property)]);
    v == 1
}

pub fn set_property_bool(element: &ExternRef, property: &str, value: bool) {
    let code = "function(element, property, value){ element[property] = value !==0; }";
    crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element), InvokeParam::String(property), InvokeParam::Bool(value)]);
}

pub fn get_property_string(element: &ExternRef, property: &str) -> String {
    let code = r#"
        function(element, property){
            const text = element[property];
            const buffer = (new TextEncoder()).encode(text);
            const allocationId = writeBufferToMemory(buffer);
            return allocationId;
        }"#;
    let text_allocation_id = crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element), InvokeParam::String(property)]);
    let text = get_string_from_allocation(text_allocation_id);
    text
}

pub fn set_property_string(element: &ExternRef, property: &str, value: &str) {
    let code = "function(element, p, v){ element[p] = v; }";
    crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element), InvokeParam::String(property), InvokeParam::String(value)]);
}

#[no_mangle]
pub extern "C" fn web_handle_empty_callback(callback_id: u32) {
    EventHandlerFuture::<()>::wake_future_with_state_id(callback_id, ());
}

pub fn sleep(ms: impl Into<f64>) -> impl Future<Output = ()> {
    let code = r#"
        function(ms, state_id){
            window.setTimeout(()=>{
                wasmModule.instance.exports.web_handle_empty_callback(state_id);
            }, ms);
        }"#;
    let (future, state_id) = EventHandlerFuture::<()>::create_future_with_state_id();
    crate::js::invoke_and_return_number(code, &[InvokeParam::Float64(ms.into()), InvokeParam::Float64(state_id as f64)]);
    future
}
