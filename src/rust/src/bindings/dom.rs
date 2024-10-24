
use crate::js::{ExternRef, InvokeParam};
use crate::allocations::get_string_from_allocation;

use std::collections::HashMap;
use std::sync::Mutex;
use std::rc::Rc;

pub fn create_element(tag: &str) -> ExternRef {
    let code = "function (t) { return document.createElement(t); }";
    crate::js::invoke_and_return_ref(code, &[InvokeParam::String(tag)])
}

pub fn create_text_node(text: &str) -> ExternRef {
    let code = "function (t) { return document.createTextNode(t); }";
    crate::js::invoke_and_return_ref(code, &[InvokeParam::String(text)])
}

pub fn append_child(parent: &ExternRef, child: &ExternRef) {
    let code = "function (p, e) { p.appendChild(e); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(parent), InvokeParam::ExternRef(child)]);
}

pub fn alert(message: &str) {
    let code = "function(message){ alert(message); }";
    crate::js::invoke_and_return(code, &[InvokeParam::String(message)]);
}

pub fn prompt(message: &str, placeholder: &str) -> String {
    let code = r#"
        function(message, placeholder){
            const text = prompt(message, placeholder);
            const buffer = (new TextEncoder()).encode(text);
            const allocationId = writeBufferToMemory(buffer);
            return allocationId;
        }"#;
    let text_allocation_id = crate::js::invoke_and_return_number(code, &[InvokeParam::String(message), InvokeParam::String(placeholder)]);
    let text = get_string_from_allocation(text_allocation_id as u32);
    text
}

pub fn query_selector(selector: &str) -> ExternRef {
    let code = "function(s){ return document.querySelector(s); }";
    crate::js::invoke_and_return_ref(code, &[InvokeParam::String(selector)])
}

pub fn element_set_inner_html(element: &ExternRef, html: &str) {
    let code = "function(element, html){ element.innerHTML = html; }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::String(html)]);
}

pub fn element_add_class(element: &ExternRef, class: &str) {
    let code = "function(element, c){ element.classList.add(c); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::String(class)]);
}

pub fn element_remove_class(element: &ExternRef, class: &str) {
    let code = "function(element, c){ element.classList.remove(c); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::String(class)]);
}

pub fn element_set_style_attribute(element: &ExternRef, attribute: &str, value: &str) {
    let code = "function(element, attribute, value){ element.style[attribute] = value; }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::String(attribute), InvokeParam::String(value)]);
}

pub fn element_set_attribute(element: &ExternRef, attribute: &str, value: &str) {
    let code = "function(element, attribute, value){ element.setAttribute(attribute, value); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::String(attribute), InvokeParam::String(value)]);
}

pub fn element_remove(element: &ExternRef) {
    let code = "function(element){ element.remove(); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element)]);
}

pub struct ChangeEvent {
    pub value: String,
}

thread_local! {
    static ELEMENT_CHANGE_HANDLERS: Mutex<HashMap<ExternRef, Box<dyn FnMut(ChangeEvent) + 'static>>> = Default::default();
}

#[no_mangle]
pub fn handle_change_event_callback(callback_id: u32, allocation_id: u32) {
    ELEMENT_CHANGE_HANDLERS.with(|s| {

        let handler = s.lock().map(|mut s| {
            let (_, handler) = s.iter_mut().find(|(s, _)| s.value == callback_id).unwrap();
            handler as *mut Box<dyn FnMut(ChangeEvent) + 'static>
        }).unwrap();

        let value = get_string_from_allocation(allocation_id);
        unsafe { (*handler)(ChangeEvent { value }) }

    });
}

pub fn add_change_event_listener(element: &ExternRef, handler: impl FnMut(ChangeEvent) + 'static) -> ExternRef {
    let code = r#"
        function(element ){
            const handler = (e) => {
                const buffer = (new TextEncoder()).encode(e.target.value);
                const allocationId = writeBufferToMemory(buffer);
                wasmModule.instance.exports.handle_change_event_callback(objectId,allocationId);
            };
            objects.push(handler);
            const objectId = objects.length - 1;
            element.addEventListener("change",handler);
            return objectId;
        }"#;
    let function_ref = crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = ExternRef { value: function_ref as u32, };

    ELEMENT_CHANGE_HANDLERS.with(|s| {
        s.lock().map(|mut s| { s.insert(function_handle.clone(), Box::new(handler)); }).unwrap();
    });

    function_handle
}

pub fn element_remove_change_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('change', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    ELEMENT_CHANGE_HANDLERS.with(|s| {
        s.lock().map(|mut s| { s.remove(function_handle); }).unwrap();
    });
}

pub struct MouseEvent {
    pub offset_x: f64,
    pub offset_y: f64,
}

thread_local! {
    static MOUSE_EVENT_HANDLER: Mutex<HashMap<ExternRef, Box<dyn FnMut(MouseEvent) + 'static>>> = Default::default();
}

#[no_mangle]
pub fn handle_mouse_event_callback(callback_id: u32, x: f64, y: f64) {

    MOUSE_EVENT_HANDLER.with(|s| {
        let handler = s.lock().map(|mut s| {
            let (_, handler) = s.iter_mut().find(|(s, _)| s.value == callback_id).unwrap();
            handler as *mut Box<dyn FnMut(MouseEvent) + 'static>
        }).unwrap();

        unsafe { (*handler)(MouseEvent { offset_x: x, offset_y: y }) }
    })
}

pub fn element_add_click_listener(element: &ExternRef, handler: impl FnMut(MouseEvent) + 'static) -> ExternRef {

    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.handle_mouse_event_callback(objectId,e.offsetX,e.offsetY);
            };
            objects.push(handler);
            const objectId = objects.length - 1;
            element.addEventListener("click",handler);
            return objectId;
        }"#;
    let function_ref = crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = ExternRef { value: function_ref as u32, };

    MOUSE_EVENT_HANDLER.with(|s| {
        s.lock().map(|mut s| { s.insert(function_handle.clone(), Box::new(handler)); }).unwrap();
    });
    function_handle
}

pub fn element_remove_click_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('click', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.lock().map(|mut s| { s.remove(function_handle); }).unwrap();
    });
}

pub fn element_add_mouse_move_listener(element: &ExternRef, handler: impl FnMut(MouseEvent) + 'static) -> ExternRef {
    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.handle_mouse_event_callback(objectId,e.offsetX,e.offsetY);
            };
            objects.push(handler);
            const objectId = objects.length - 1;
            element.addEventListener("mousemove",handler);
            return objectId;
        }"#;
    let function_ref = crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = ExternRef { value: function_ref as u32, };
    MOUSE_EVENT_HANDLER.with(|s| {
        s.lock().map(|mut s| { s.insert(function_handle.clone(), Box::new(handler)); }).unwrap();
    });
    function_handle
}

pub fn element_remove_mouse_move_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('mousemove', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.lock().map(|mut s| { s.remove(function_handle); }).unwrap();
    });
}

pub fn element_add_mouse_down_listener(element: &ExternRef, handler: impl FnMut(MouseEvent) + 'static) -> ExternRef {
    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.handle_mouse_event_callback(objectId,e.offsetX,e.offsetY);
            };
            objects.push(handler);
            const objectId = objects.length - 1;
            element.addEventListener("mousedown",handler);
            return objectId;
        }"#;
    let function_ref = crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = ExternRef { value: function_ref as u32, };
    MOUSE_EVENT_HANDLER.with(|s| {
        s.lock().map(|mut s| { s.insert(function_handle.clone(), Box::new(handler)); }).unwrap();
    });
    function_handle
}

pub fn element_remove_mouse_down_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('mousedown', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.lock().map(|mut s| { s.remove(function_handle); }).unwrap();
    });
}

pub fn element_add_mouse_up_listener(element: &ExternRef, handler: impl FnMut(MouseEvent) + 'static) -> ExternRef {
    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.handle_mouse_event_callback(objectId,e.offsetX,e.offsetY);
            };
            objects.push(handler);
            const objectId = objects.length - 1;
            element.addEventListener("mouseup",handler);
            return objectId;
        }"#;
    let function_ref = crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = ExternRef { value: function_ref as u32, };
    MOUSE_EVENT_HANDLER.with(|s| {
        s.lock().map(|mut s| { s.insert(function_handle.clone(), Box::new(handler)); }).unwrap();
    });
    function_handle
}

pub fn element_remove_mouse_up_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('mouseup', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.lock().map(|mut s| { s.remove(function_handle); }).unwrap();
    });
}

pub struct KeyboardEvent {
    pub key_code: u32,
}

thread_local! {
    static KEYBOARD_EVENT_HANDLERS: Mutex<HashMap<ExternRef, Box<dyn FnMut(KeyboardEvent) + 'static>>> = Default::default();
}

#[no_mangle]
pub fn handle_keyboard_event_callback(callback_id: u32, key_code: u32) {

    KEYBOARD_EVENT_HANDLERS.with(|s| {

        let handler = s.lock().map(|mut s| {
            let (_, handler) = s.iter_mut().find(|(s, _)| s.value == callback_id).unwrap();
            handler as *mut Box<dyn FnMut(KeyboardEvent) + 'static>
        }).unwrap();

        unsafe { (*handler)(KeyboardEvent { key_code }) }

    });
}

pub fn element_add_key_down_listener(element: &ExternRef, handler: impl FnMut(KeyboardEvent) + 'static) -> ExternRef {
    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.handle_keyboard_event_callback(objectId,e.keyCode);
            };
            objects.push(handler);
            const objectId = objects.length - 1;
            element.addEventListener("keydown",handler);
            return objectId;
        }"#;
    let function_ref = crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = ExternRef { value: function_ref as u32, };

    KEYBOARD_EVENT_HANDLERS.with(|h| {
        h.lock().map(|mut s| { s.insert(function_handle.clone(), Box::new(handler)); }).unwrap();
    });

    function_handle
}

pub fn element_remove_key_down_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('keydown', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    KEYBOARD_EVENT_HANDLERS.with(|h| {
        h.lock().map(|mut s| { s.remove(function_handle); }).unwrap();
    });
}

pub fn element_add_key_up_listener(element: &ExternRef, handler: impl FnMut(KeyboardEvent) + 'static) -> ExternRef {
    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.handle_keyboard_event_callback(objectId,e.keyCode);
            };
            objects.push(handler);
            const objectId = objects.length - 1;
            element.addEventListener("keyup",handler);
            return objectId;
        }"#;
    let function_ref = crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = ExternRef { value: function_ref as u32, };

    KEYBOARD_EVENT_HANDLERS.with(|h| {
        h.lock().map(|mut s| { s.insert(function_handle.clone(), Box::new(handler)); }).unwrap();
    });

    function_handle
}

pub fn element_remove_key_up_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('keyup', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    KEYBOARD_EVENT_HANDLERS.with(|h| {
        h.lock().map(|mut s| { s.remove(function_handle); }).unwrap();
    });
}


#[cfg(test)]
mod tests {

    use std::cell::RefCell;

    use crate::js::ExternRef;

    use super::*;

    thread_local! {
        static EVENT_HANDLER: Mutex<HashMap<ExternRef, Box<dyn FnMut(()) + 'static>>> = Default::default();
    }

    #[test]
    fn test_run() {

        let has_run = Rc::new(RefCell::new(false));
        let has_run_clone = has_run.clone();

        // add listener
        let function_handle = ExternRef { value: 0, };
        let handler = move |_| {
            *has_run_clone.borrow_mut() = true;
        };
        EVENT_HANDLER.with(|s| {
            s.lock().map(|mut s| { s.insert(function_handle.clone(), Box::new(handler)); }).unwrap();
        });

        // call listener
        EVENT_HANDLER.with(|s| {

            let handler = s.lock().map(|mut s| {
                let (_, handler) = s.iter_mut().find(|(s, _)| s.value == 0).unwrap();
                handler as *mut Box<dyn FnMut(()) + 'static>
            }).unwrap();

            unsafe { (*handler)(()) }
        });
        assert_eq!(*has_run.borrow(), true);

        // remove listener
        EVENT_HANDLER.with(|s| { s.lock().map(|mut s| { s.remove(&function_handle); }).unwrap(); });
        let count = EVENT_HANDLER.with(|s| s.lock().map(|s| s.len()).unwrap());
        assert_eq!(count, 0);
    }

}
