
use crate::runtime::EventHandlerFuture;
use crate::js::{ExternRef, InvokeParam};

use std::collections::HashMap;
use std::future::Future;
use std::sync::Mutex;
use std::rc::Rc;

thread_local! {
    static HTTP_LOAD_HANDLERS: Mutex<HashMap<u32, Box<dyn FnMut() + 'static>>> = Default::default();
}

fn add_http_load_event_handler(function_handle: u32, handler: Box<dyn FnMut() + 'static>) {

    HTTP_LOAD_HANDLERS.with(|h| {
        h.lock().unwrap().insert(function_handle, handler);
    });
}

#[no_mangle]
pub fn web_handle_http_load_event_handler(callback_id: u32) {
    HTTP_LOAD_HANDLERS.with(|h| {
        if let Some(mut handler) = h.lock().unwrap().remove(&(callback_id as u32)) {
            handler();
        }
    });
}

pub struct XMLHttpRequest(ExternRef);

impl XMLHttpRequest {
    pub fn new() -> XMLHttpRequest {
        let code = "function() { return new XMLHttpRequest(); }";
        let request = crate::js::invoke_and_return_ref(code, &[]);
        XMLHttpRequest(request)
    }

    pub fn open(&self, method: &str, url: &str) {
        let code = "function(request, method, url) { request.open(method, url); }";
        crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(&self.0), InvokeParam::String(method), InvokeParam::String(url)]);
    }

    pub fn send(&self) {
        let code = "function(request) { request.send(); }";
        crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(&self.0)]);
    }

    pub fn send_with_body(&self, body: &str) {
        let code = "function(request, body) { request.send(body); }";
        crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(&self.0), InvokeParam::String(body)]);
    }

    pub fn set_request_header(&self, key: &str, value: &str) {
        let code = "function(request, k, v) { request.setRequestHeader(k, v); }";
        crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(&self.0), InvokeParam::String(key), InvokeParam::String(value)]);
    }

    pub fn response_status(&self) -> u32 {
        let code = "function(request) { return request.status; }";
        crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(&self.0)]) as u32
    }

    pub fn response_text(&self) -> String {
        let code = "function(request) { return request.responseText; }";
        crate::js::invoke_and_return_string(code, &[InvokeParam::ExternRef(&self.0)])
    }

    pub fn response_array_buffer(&self) -> Vec<u8> {
        let code = "function(request) { return request.response; }";
        crate::js::invoke_and_return_array_buffer(code, &[InvokeParam::ExternRef(&self.0)])
    }

    pub fn response_header(&self, key: &str) -> String {
        let code = "function(request, key) { return request.getResponseHeader(key); }";
        crate::js::invoke_and_return_string(code, &[InvokeParam::ExternRef(&self.0), InvokeParam::String(key)])
    }

    pub fn set_on_load(&self, callback: impl FnMut() + 'static) {
        let code = r#"
            function(request){
                const handler = () => {
                    wasmModule.instance.exports.web_handle_http_load_event_handler(objectId);
                };
                objects.push(handler);
                const objectId = objects.length - 1;
                request.onload = handler;
                return objectId;
            }"#;
        let function_ref = crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(&self.0)]);
        add_http_load_event_handler(function_ref as u32, Box::new(callback));
    }

    pub fn set_response_type(&self, response_type: &str) {
        let code = "function(request, response_type) { request.responseType = response_type; }";
        crate::js::invoke_and_return_number(code, &[InvokeParam::ExternRef(&self.0), InvokeParam::String(response_type)]);
    }
}

pub enum HTTPMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH,
}

pub enum FetchResponse {
    Text(u32, String),
    ArrayBuffer(u32, Vec<u8>),
}

pub struct FetchOptions<'a> {
    pub url: &'a str,
    pub action: HTTPMethod,
    pub body: Option<&'a str>,
    pub headers: Option<HashMap<String, String>>,
    pub response_type: FetchResponseType,
}

pub enum FetchResponseType {
    Text,
    ArrayBuffer,
}

impl Default for FetchOptions<'_> {
    fn default() -> Self {
        FetchOptions {
            url: "",
            action: HTTPMethod::GET,
            body: None,
            headers: None,
            response_type: FetchResponseType::Text,
        }
    }
}

pub fn fetch(options: FetchOptions) -> impl Future<Output = FetchResponse> {
    let url = options.url;
    let body = options.body;
    let headers = options.headers;
    let response_type = options.response_type;
    let action = options.action;
    let request = Rc::new(XMLHttpRequest::new());
    let r2 = request.clone();
    let method_str = match action {
        HTTPMethod::GET => "GET",
        HTTPMethod::POST => "POST",
        HTTPMethod::PUT => "PUT",
        HTTPMethod::DELETE => "DELETE",
        HTTPMethod::HEAD => "HEAD",
        HTTPMethod::OPTIONS => "OPTIONS",
        HTTPMethod::PATCH => "PATCH",
    };
    request.open(method_str, &url);
    if let Some(body) = body {
        request.send_with_body(&body);
    } else {
        request.send();
    }
    if let Some(headers) = headers {
        for (key, value) in headers {
            request.set_request_header(&key, &value);
        }
    }
    match response_type {
        FetchResponseType::Text => {
            request.set_response_type("text");
        }
        FetchResponseType::ArrayBuffer => {
            request.set_response_type("arraybuffer");
        }
    }

    let (future, state_id) = EventHandlerFuture::<FetchResponse>::create_future_with_state_id();
    request.set_on_load(move || match response_type {
        FetchResponseType::Text => {
            let status = r2.response_status();
            let text = r2.response_text();
            EventHandlerFuture::<FetchResponse>::wake_future_with_state_id(
                state_id,
                FetchResponse::Text(status, text),
            );
        }
        FetchResponseType::ArrayBuffer => {
            let status = r2.response_status();
            let ab = r2.response_array_buffer();
            EventHandlerFuture::<FetchResponse>::wake_future_with_state_id(
                state_id,
                FetchResponse::ArrayBuffer(status, ab),
            );
        }
    });
    return future;
}
