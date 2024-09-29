
use crate::js::{ExternRef, JSFunction, InvokeParam};
use crate::allocations::get_string_from_allocation;
use crate::handlers::EventHandler;

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_element(tag: &str) -> ExternRef {
    let create_fn = JSFunction::register(r#"
        function (t) {
            return document.createElement(t);
        }"#);
    create_fn.invoke_and_return_object(&[tag.into()])
}

pub fn create_text_node(text: &str) -> ExternRef {
    let create_fn = JSFunction::register(r#"
        function (t) {
            return document.createTextNode(t);
        }"#);
    create_fn.invoke_and_return_object(&[text.into()])
}

pub fn append_child(parent: &ExternRef, child: &ExternRef) {
    let append_fn = JSFunction::register("
        function (p, e) {
            p.appendChild(e);
        }");
    append_fn.invoke(&[parent.into(), child.into()]);
}

pub fn alert(message: &str) {
    let message_fn = JSFunction::register(r#"
        function(message){
            alert(message);
        }"#);
    message_fn.invoke(&[message.into()]);
}

pub fn query_selector(selector: &str) -> ExternRef {
    let query_selector = JSFunction::register(r#"
        function(selector){
            return document.querySelector(selector);
        }"#);
    query_selector.invoke_and_return_object(&[selector.into()])
}

pub fn element_set_inner_html(element: &ExternRef, html: &str) {
    let set_inner_html = JSFunction::register(r#"
        function(element, html){
            element.innerHTML = html;
        }"#);
    set_inner_html.invoke(&[element.into(), html.into()]);
}

pub fn element_add_class(element: &ExternRef, class: &str) {
    let add_class = JSFunction::register(r#"
        function(element, c){
            element.classList.add(c);
        }"#);
    add_class.invoke(&[element.into(), class.into()]);
}

pub fn element_remove_class(element: &ExternRef, class: &str) {
    let remove_class = JSFunction::register(r#"
        function(element, c){
            element.classList.remove(c);
        }"#);
    remove_class.invoke(&[element.into(), class.into()]);
}

pub fn element_set_style_attribute(element: &ExternRef, attribute: &str, value: &str) {
    let set_style_attribute = JSFunction::register(r#"
        function(element, attribute, value){
            element.style[attribute] = value;
        }"#);
    set_style_attribute.invoke(&[element.into(), attribute.into(), value.into()]);
}

pub fn element_set_attribute(element: &ExternRef, attribute: &str, value: &str) {
    let set_attribute = JSFunction::register(r#"
        function(element, attribute, value){
            element.setAttribute(attribute, value);
        }"#);
    set_attribute.invoke(&[element.into(), attribute.into(), value.into()]);
}

pub fn element_remove(element: &ExternRef) {
    let remove = JSFunction::register(r#"
        function(element){
            element.remove();
        }"#);
    remove.invoke(&[element.into()]);
}

pub struct ChangeEvent {
    pub value: String,
}

thread_local! {
    pub static CHANGE_EVENT_HANDLERS: RefCell<Option<HashMap<Rc<ExternRef>, Box<dyn FnMut(ChangeEvent) + 'static>>>> = Default::default();
}

fn add_change_event_handler(
    id: Rc<ExternRef>,
    handler: Box<dyn FnMut(ChangeEvent) + 'static>,
) {
    CHANGE_EVENT_HANDLERS.with_borrow_mut(|s| {
        if let Some(h) = s.as_mut() {
            h.insert(id, handler);
        } else {
            let mut h = HashMap::new();
            h.insert(id, handler);
            *s = Some(h);
        }
    });
}

fn remove_change_event_handler(id: &Rc<ExternRef>) {

    CHANGE_EVENT_HANDLERS.with_borrow_mut(|s| {
        if let Some(h) = s.as_mut() {
            h.remove(id);
        }
    });
}

#[no_mangle]
pub extern "C" fn web_handle_change_event(id: u64, allocation_id: usize) {
    CHANGE_EVENT_HANDLERS.with_borrow_mut(|s| {
        if let Some(h) = s.as_mut() {
            for (key, handler) in h.iter_mut() {
                if key.value == id {
                    let value = get_string_from_allocation(allocation_id);
                    handler(ChangeEvent { value });
                }
            }
        }
    });
}

pub fn add_change_event_listener(
    element: &ExternRef,
    handler: impl FnMut(ChangeEvent) + 'static,
) -> Rc<ExternRef> {
    let function_ref = JSFunction::register(r#"
        function(element ){
            const handler = (e) => {
                const value = e.target.value;
                const allocationId = writeStringToMemory(value);
                _wasmModule.instance.exports.web_handle_change_event_handler(id, allocationId);
            };
            const id = allocate(handler);
            element.addEventListener("change",handler);
            return id;
        }"#)
    .invoke_and_return_bigint(&[element.into()]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u64, });
    add_change_event_handler(function_handle.clone(), Box::new(handler));
    function_handle
}

pub fn element_remove_change_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let remove_change_listener = JSFunction::register(r#"
        function(element, f){
            element.removeEventListener("change", f);
        }"#);
    remove_change_listener.invoke(&[element.into(), InvokeParam::ExternRef(&function_handle)]);
    remove_change_event_handler(function_handle);
}

pub struct MouseEvent {
    pub offset_x: f64,
    pub offset_y: f64,
}

thread_local! {
    pub static MOUSE_EVENT_HANDLER: EventHandler<MouseEvent> = EventHandler { listeners: RefCell::new(None) };
}

#[no_mangle]
pub extern "C" fn web_handle_mouse_event_handler(id: u64, x: f64, y: f64) {

    MOUSE_EVENT_HANDLER.with(|s| {
        s.call(id, MouseEvent { offset_x: x, offset_y: y });
    })
}

pub fn element_add_click_listener(
    element: &ExternRef,
    handler: impl FnMut(MouseEvent) + 'static,
) -> Rc<ExternRef> {
    let function_ref = JSFunction::register(r#"
        function(element ){
            const handler = (e) => {
                _wasmModule.instance.exports.web_handle_mouse_event_handler(id,e.offsetX, e.offsetY);
            };
            const id = allocate(handler);
            element.addEventListener("click",handler);
            return id;
        }"#).invoke_and_return_bigint(&[element.into()]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u64, });

    MOUSE_EVENT_HANDLER.with(|s| {
        s.add_listener(function_handle.clone(), Box::new(handler));
    });
    function_handle
}

pub fn element_remove_click_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let remove_click_listener = JSFunction::register(r#"
        function(element, f){
            element.removeEventListener("click", f);
        }"#);
    remove_click_listener.invoke(&[element.into(), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.remove_listener(function_handle);
    });
}

pub fn element_add_mouse_move_listener(
    element: &ExternRef,
    handler: impl FnMut(MouseEvent) + 'static,
) -> Rc<ExternRef> {
    let function_ref = JSFunction::register(r#"
        function(element ){
            const handler = (e) => {
                _wasmModule.instance.exports.web_handle_mouse_event_handler(id,e.offsetX, e.offsetY);
            };
            const id = allocate(handler);
            element.addEventListener("mousemove",handler);
            return id;
        }"#).invoke_and_return_bigint(&[element.into()]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u64, });
    MOUSE_EVENT_HANDLER.with(|s| {
        s.add_listener(function_handle.clone(), Box::new(handler));
    });
    function_handle
}

pub fn element_remove_mouse_move_listener(
    element: &ExternRef,
    function_handle: &Rc<ExternRef>,
) {
    let remove_mouse_move_listener = JSFunction::register(r#"
        function(element, f){
            element.removeEventListener("mousemove", f);
        }"#);
    remove_mouse_move_listener.invoke(&[element.into(), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.remove_listener(function_handle);
    });
}

pub fn element_add_mouse_down_listener(
    element: &ExternRef,
    handler: impl FnMut(MouseEvent) + 'static,
) -> Rc<ExternRef> {
    let function_ref = JSFunction::register(r#"
        function(element ){
            const handler = (e) => {
                _wasmModule.instance.exports.web_handle_mouse_event_handler(id,e.offsetX, e.offsetY);
            };
            const id = allocate(handler);
            element.addEventListener("mousedown",handler);
            return id;
        }"#).invoke_and_return_bigint(&[element.into()]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u64, });
    MOUSE_EVENT_HANDLER.with(|s| {
        s.add_listener(function_handle.clone(), Box::new(handler));
    });
    function_handle
}

pub fn element_remove_mouse_down_listener(
    element: &ExternRef,
    function_handle: &Rc<ExternRef>,
) {
    let remove_mouse_down_listener = JSFunction::register(r#"
        function(element, f){
            element.removeEventListener("mousedown", f);
        }"#);
    remove_mouse_down_listener.invoke(&[element.into(), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.remove_listener(function_handle);
    });
}

pub fn element_add_mouse_up_listener(
    element: &ExternRef,
    handler: impl FnMut(MouseEvent) + 'static,
) -> Rc<ExternRef> {
    let function_ref = JSFunction::register(r#"
        function(element ){
            const handler = (e) => {
                _wasmModule.instance.exports.web_handle_mouse_event_handler(id,e.offsetX, e.offsetY);
            };
            const id = allocate(handler);
            element.addEventListener("mouseup",handler);
            return id;
        }"#).invoke_and_return_bigint(&[element.into()]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u64, });
    MOUSE_EVENT_HANDLER.with(|s| {
        s.add_listener(function_handle.clone(), Box::new(handler));
    });
    function_handle
}

pub fn element_remove_mouse_up_listener(
    element: &ExternRef,
    function_handle: &Rc<ExternRef>,
) {
    let remove_mouse_up_listener = JSFunction::register(r#"
        function(element, f){
            element.removeEventListener("mouseup", f);
        }"#);
    remove_mouse_up_listener.invoke(&[element.into(), InvokeParam::ExternRef(&function_handle)]);
    MOUSE_EVENT_HANDLER.with(|s| {
        s.remove_listener(function_handle);
    });
}

pub struct KeyboardEvent {
    pub key_code: f64,
}

thread_local! {
    pub static KEYBOARD_EVENT_HANDLERS: RefCell<Option<HashMap<Rc<ExternRef>, Box<dyn FnMut(KeyboardEvent) + 'static>>>> = Default::default();
}

fn add_keyboard_event_handler(
    function_handle: Rc<ExternRef>,
    handler: Box<dyn FnMut(KeyboardEvent) + 'static>,
) {

    KEYBOARD_EVENT_HANDLERS.with_borrow_mut(|h| {
        if h.is_none() {
            *h = Some(HashMap::new());
        }
        h.as_mut().unwrap().insert(function_handle, handler);
    });
}

fn remove_keyboard_event_handler(function_handle: &Rc<ExternRef>) {
    KEYBOARD_EVENT_HANDLERS.with_borrow_mut(|h| {
        if h.is_none() {
            return;
        }
        h.as_mut().unwrap().remove(function_handle);
    });
}

#[no_mangle]
pub extern "C" fn web_handle_keyboard_event_handler(id: u64, key_code: f64) {

    KEYBOARD_EVENT_HANDLERS.with_borrow_mut(|s| {
        if let Some(h) = s.as_mut() {
            for (key, handler) in h.iter_mut() {
                if key.value == id {
                    handler(KeyboardEvent { key_code });
                }
            }
        }
    });
}

pub fn element_add_key_down_listener(
    element: &ExternRef,
    handler: impl FnMut(KeyboardEvent) + 'static,
) -> Rc<ExternRef> {
    let function_ref = JSFunction::register(r#"
        function(element ){
            const handler = (e) => {
                _wasmModule.instance.exports.web_handle_keyboard_event_handler(id,e.keyCode);
            };
            const id = allocate(handler);
            element.addEventListener("keydown",handler);
            return id;
        }"#)
    .invoke_and_return_bigint(&[element.into()]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u64, });
    add_keyboard_event_handler(function_handle.clone(), Box::new(handler));
    function_handle
}

pub fn element_remove_key_down_listener(
    element: &ExternRef,
    function_handle: &Rc<ExternRef>,
) {
    let remove_key_down_listener = JSFunction::register(r#"
        function(element, f){
            element.removeEventListener("keydown", f);
        }"#);
    remove_key_down_listener.invoke(&[element.into(), InvokeParam::ExternRef(&function_handle)]);
    remove_keyboard_event_handler(function_handle);
}

pub fn element_add_key_up_listener(
    element: &ExternRef,
    handler: impl FnMut(KeyboardEvent) + 'static,
) -> Rc<ExternRef> {
    let function_ref = JSFunction::register(r#"
        function(element ){
            const handler = (e) => {
                _wasmModule.instance.exports.web_handle_keyboard_event_handler(id,e.keyCode);
            };
            const id = allocate(handler);
            element.addEventListener("keyup",handler);
            return id;
        }"#)
    .invoke_and_return_bigint(&[element.into()]);
    let function_handle = Rc::new(ExternRef { value: function_ref as u64, });
    add_keyboard_event_handler(function_handle.clone(), Box::new(handler));
    function_handle
}

pub fn element_remove_key_up_listener(element: &ExternRef, function_handle: &Rc<ExternRef>) {
    let remove_key_up_listener = JSFunction::register(r#"
        function(element, f){
            element.removeEventListener("keyup", f);
        }"#);
    remove_key_up_listener.invoke(&[element.into(), InvokeParam::ExternRef(&function_handle)]);
    remove_keyboard_event_handler(function_handle);
}
