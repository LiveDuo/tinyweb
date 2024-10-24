
use crate::js::{ExternRef, InvokeParam};
use crate::allocations::get_string_from_allocation;

use std::collections::HashMap;
use std::sync::Mutex;
use std::rc::Rc;

pub fn create_element(tag: &str) -> ExternRef {
    let code = "function (t) { return document.createElement(t); }";
    crate::js::invoke_and_return_object(code, &[InvokeParam::String(tag)])
}

pub fn create_text_node(text: &str) -> ExternRef {
    let code = "function (t) { return document.createTextNode(t); }";
    crate::js::invoke_and_return_object(code, &[InvokeParam::String(text)])
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
    let text_allocation_id = crate::js::invoke_and_return(code, &[InvokeParam::String(message), InvokeParam::String(placeholder)]);
    let text = get_string_from_allocation(text_allocation_id);
    text
}

pub fn query_selector(selector: &str) -> ExternRef {
    let code = "function(s){ return document.querySelector(s); }";
    crate::js::invoke_and_return_object(code, &[InvokeParam::String(selector)])
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
    static ELEMENT_CHANGE_HANDLERS: Mutex<HashMap<Rc<ExternRef>, Box<dyn FnMut(ChangeEvent) + 'static>>> = Default::default();
}

fn add_change_event_handler(id: Rc<ExternRef>, handler: Box<dyn FnMut(ChangeEvent) + 'static>) {
    ELEMENT_CHANGE_HANDLERS.with(|s| {
        s.lock().unwrap().insert(id, handler);
    });
}

fn remove_change_event_handler(id: &Rc<ExternRef>) {

    ELEMENT_CHANGE_HANDLERS.with(|s| {
        s.lock().unwrap().remove(id);
    });
}

#[no_mangle]
pub extern "C" fn web_handle_change_event(id: i64, allocation_id: u32) {
    ELEMENT_CHANGE_HANDLERS.with(|s| {

        let handler = s.lock().map(|mut s| {
            let (_, handler) = s.iter_mut().find(|(s, _)| s.value == id as u32).unwrap();
            handler as *mut Box<dyn FnMut(ChangeEvent) + 'static>
        }).unwrap();

        let value = get_string_from_allocation(allocation_id);
        unsafe { (*handler)(ChangeEvent { value }) }

    });
}

pub fn add_change_event_listener(element: &ExternRef, handler: impl FnMut(ChangeEvent) + 'static) -> Rc<ExternRef> {
    let code = r#"
        function(element ){
            const handler = (e) => {
                const text = e.target.value;
                const buffer = (new TextEncoder()).encode(text);
                const allocationId = writeBufferToMemory(buffer);
                wasmModule.instance.exports.web_handle_change_event(id, allocationId);
            };
            const id = allocate(handler);
            element.addEventListener("change",handler);
            return id;
        }"#;
    let function_ref = crate::js::invoke_and_return_bigint(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u32, });
    add_change_event_handler(function_handle.clone(), Box::new(handler));
    function_handle
}

pub fn element_remove_change_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('change', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    remove_change_event_handler(function_handle);
}

pub struct EventHandler<T> {
    pub listeners: Mutex<HashMap<Rc<ExternRef>, Box<dyn FnMut(T) + 'static>>>,
}

impl<T> EventHandler<T> {
    pub fn add_listener(&self, id: Rc<ExternRef>, handler: Box<dyn FnMut(T) + 'static>) {
        self.listeners.lock().map(|mut s| { s.insert(id, handler); }).unwrap();
    }

    pub fn remove_listener(&self, id: &Rc<ExternRef>) {
        let mut handlers = self.listeners.lock().unwrap();
        handlers.remove(id);
    }

    pub fn call(&self, id: i64, event: T) {

        let handler = self.listeners.lock().map(|mut s| {
            let (_, handler) = s.iter_mut().find(|(s, _)| s.value == id as u32).unwrap();
            handler as *mut Box<dyn FnMut(T) + 'static>
        }).unwrap();

        unsafe { (*handler)(event) }
    }
}

pub struct MouseEvent {
    pub offset_x: f64,
    pub offset_y: f64,
}

thread_local! {
    static MOUSE_EVENT_HANDLER: EventHandler<MouseEvent> = EventHandler { listeners: Default::default() };
}

#[no_mangle]
pub extern "C" fn web_handle_mouse_event_handler(id: i64, x: f64, y: f64) {

    MOUSE_EVENT_HANDLER.with(|s| {
        s.call(id, MouseEvent { offset_x: x, offset_y: y });
    })
}

pub fn element_add_click_listener(element: &ExternRef, handler: impl FnMut(MouseEvent) + 'static) -> Rc<ExternRef> {

    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.web_handle_mouse_event_handler(id,e.offsetX, e.offsetY);
            };
            const id = allocate(handler);
            element.addEventListener("click",handler);
            return id;
        }"#;
    let function_ref = crate::js::invoke_and_return_bigint(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u32, });

    MOUSE_EVENT_HANDLER.with(|s| {
        s.add_listener(function_handle.clone(), Box::new(handler));
    });
    function_handle
}

pub fn element_remove_click_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('click', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.remove_listener(function_handle);
    });
}

pub fn element_add_mouse_move_listener(element: &ExternRef, handler: impl FnMut(MouseEvent) + 'static) -> Rc<ExternRef> {
    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.web_handle_mouse_event_handler(id,e.offsetX, e.offsetY);
            };
            const id = allocate(handler);
            element.addEventListener("mousemove",handler);
            return id;
        }"#;
    let function_ref = crate::js::invoke_and_return_bigint(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u32, });
    MOUSE_EVENT_HANDLER.with(|s| {
        s.add_listener(function_handle.clone(), Box::new(handler));
    });
    function_handle
}

pub fn element_remove_mouse_move_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('mousemove', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.remove_listener(function_handle);
    });
}

pub fn element_add_mouse_down_listener(element: &ExternRef, handler: impl FnMut(MouseEvent) + 'static) -> Rc<ExternRef> {
    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.web_handle_mouse_event_handler(id,e.offsetX, e.offsetY);
            };
            const id = allocate(handler);
            element.addEventListener("mousedown",handler);
            return id;
        }"#;
    let function_ref = crate::js::invoke_and_return_bigint(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u32, });
    MOUSE_EVENT_HANDLER.with(|s| {
        s.add_listener(function_handle.clone(), Box::new(handler));
    });
    function_handle
}

pub fn element_remove_mouse_down_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('mousedown', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.remove_listener(function_handle);
    });
}

pub fn element_add_mouse_up_listener(element: &ExternRef, handler: impl FnMut(MouseEvent) + 'static) -> Rc<ExternRef> {
    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.web_handle_mouse_event_handler(id,e.offsetX, e.offsetY);
            };
            const id = allocate(handler);
            element.addEventListener("mouseup",handler);
            return id;
        }"#;
    let function_ref = crate::js::invoke_and_return_bigint(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u32, });
    MOUSE_EVENT_HANDLER.with(|s| {
        s.add_listener(function_handle.clone(), Box::new(handler));
    });
    function_handle
}

pub fn element_remove_mouse_up_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('mouseup', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.remove_listener(function_handle);
    });
}

pub struct KeyboardEvent {
    pub key_code: f64,
}

thread_local! {
    static KEYBOARD_EVENT_HANDLERS: Mutex<HashMap<Rc<ExternRef>, Box<dyn FnMut(KeyboardEvent) + 'static>>> = Default::default();
}

fn add_keyboard_event_handler(function_handle: Rc<ExternRef>, handler: Box<dyn FnMut(KeyboardEvent) + 'static>) {

    KEYBOARD_EVENT_HANDLERS.with(|h| {
        h.lock().unwrap().insert(function_handle, handler);
    });
}

fn remove_keyboard_event_handler(function_handle: &Rc<ExternRef>) {
    KEYBOARD_EVENT_HANDLERS.with(|h| {
        h.lock().unwrap().remove(function_handle);
    });
}

#[no_mangle]
pub extern "C" fn web_handle_keyboard_event_handler(id: i64, key_code: f64) {

    KEYBOARD_EVENT_HANDLERS.with(|s| {

        let handler = s.lock().map(|mut s| {
            let (_, handler) = s.iter_mut().find(|(s, _)| s.value == id as u32).unwrap();
            handler as *mut Box<dyn FnMut(KeyboardEvent) + 'static>
        }).unwrap();

        unsafe { (*handler)(KeyboardEvent { key_code }) }

    });
}

pub fn element_add_key_down_listener(element: &ExternRef, handler: impl FnMut(KeyboardEvent) + 'static) -> Rc<ExternRef> {
    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.web_handle_keyboard_event_handler(id,e.keyCode);
            };
            const id = allocate(handler);
            element.addEventListener("keydown",handler);
            return id;
        }"#;
    let function_ref = crate::js::invoke_and_return_bigint(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u32, });
    add_keyboard_event_handler(function_handle.clone(), Box::new(handler));
    function_handle
}

pub fn element_remove_key_down_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('keydown', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    remove_keyboard_event_handler(function_handle);
}

pub fn element_add_key_up_listener(element: &ExternRef, handler: impl FnMut(KeyboardEvent) + 'static) -> Rc<ExternRef> {
    let code = r#"
        function(element ){
            const handler = (e) => {
                wasmModule.instance.exports.web_handle_keyboard_event_handler(id,e.keyCode);
            };
            const id = allocate(handler);
            element.addEventListener("keyup",handler);
            return id;
        }"#;
    let function_ref = crate::js::invoke_and_return_bigint(code, &[InvokeParam::ExternRef(element)]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u32, });
    add_keyboard_event_handler(function_handle.clone(), Box::new(handler));
    function_handle
}

pub fn element_remove_key_up_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let code = "function(element, f){ element.removeEventListener('keyup', f); }";
    crate::js::invoke_and_return(code, &[InvokeParam::ExternRef(element), InvokeParam::ExternRef(&function_handle)]);
    remove_keyboard_event_handler(function_handle);
}


#[cfg(test)]
mod tests {

    use std::cell::RefCell;

    use crate::js::ExternRef;

    use super::*;

    thread_local! {
        static EVENT_HANDLER: EventHandler<()> = EventHandler { listeners: Default::default() };
    }

    #[test]
    fn test_run() {

        let has_run = Rc::new(RefCell::new(false));
        let has_run_clone = has_run.clone();

        // add listener
        let function_handle = Rc::new(ExternRef { value: 0, });
        let handler = move |_| {
            *has_run_clone.borrow_mut() = true;
        };
        EVENT_HANDLER.with(|s| s.add_listener(function_handle.clone(), Box::new(handler)));

        // call listener
        EVENT_HANDLER.with(|s| s.call(0, ()));
        assert_eq!(*has_run.borrow(), true);

        // remove listener
        EVENT_HANDLER.with(|s| s.remove_listener(&function_handle.clone()));
        let count = EVENT_HANDLER.with(|s| s.listeners.lock().unwrap().len());
        assert_eq!(count, 0);
    }

}
