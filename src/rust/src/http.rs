
use crate::handlers::create_empty_callback;
use crate::runtime::RuntimeFuture;
use crate::invoke::Js;

use crate::invoke::JsValue::*;

use std::collections::HashMap;
use std::future::Future;
use std::rc::Rc;


#[derive(Default)]
pub enum HttpMethod { #[default] GET, POST, PUT, DELETE, HEAD, OPTIONS, PATCH, }

impl HttpMethod {
    fn to_string(&self) -> String {
        match self {
            Self::GET => "GET".to_owned(),
            Self::POST => "POST".to_owned(),
            Self::PUT => "PUT".to_owned(),
            Self::DELETE => "DELETE".to_owned(),
            Self::HEAD => "HEAD".to_owned(),
            Self::OPTIONS => "OPTIONS".to_owned(),
            Self::PATCH => "PATCH".to_owned(),
        }
    }
}

#[derive(Default)]
pub enum FetchResponseType { #[default] Text, ArrayBuffer }

impl FetchResponseType {
    fn to_string(&self) -> String {
        match self {
            Self::Text => "text".to_owned(),
            Self::ArrayBuffer => "arraybuffer".to_owned(),
        }
    }
}

#[derive(Default)]
pub struct FetchOptions<'a> {
    pub url: &'a str,
    pub method: HttpMethod,
    pub body: Option<&'a str>,
    pub headers: HashMap<String, String>,
    pub response_type: FetchResponseType,
}

pub enum FetchResponse { Text(u32, String), ArrayBuffer(u32, Vec<u8>) }

pub fn fetch(options: FetchOptions) -> impl Future<Output = FetchResponse> {

    // send request
    let request = Rc::new(Js::invoke("return new XMLHttpRequest()", &[])).to_ref().unwrap();
    Js::invoke("{}.open({},{})", &[Ref(request), Str(options.method.to_string()), Str(options.url.to_owned())]);
    options.headers.iter().for_each(|(k, v)| { Js::invoke("{}.setRequestHeader({},{})", &[Ref(request), Str(k.into()), Str(v.into())]); });
    Js::invoke("{}.responseType = {}", &[Ref(request), Str(options.response_type.to_string())]);
    if let Some(body) = options.body {
        Js::invoke("{}.send({})", &[Ref(request), Str(body.into())]);
    } else {
        Js::invoke("{}.send()", &[Ref(request)]);
    }

    // handle response
    let r2 = request.clone();
    let future = RuntimeFuture::new();
    let future_id = future.id();
    let function_ref = create_empty_callback(move || {

        let status = Js::invoke("return {}.status", &[Ref(r2)]).to_num().unwrap() as u32;
        let result = match options.response_type {
            FetchResponseType::Text => {
                FetchResponse::Text(status, Js::invoke("return {}.responseText", &[Ref(r2)]).to_str().unwrap())
            }
            FetchResponseType::ArrayBuffer => {
                FetchResponse::ArrayBuffer(status, Js::invoke("return {}.response", &[Ref(r2)]).to_buffer().unwrap())
            }
        };
        RuntimeFuture::wake(future_id, result);
    });
    Js::invoke("{}.onload = {}", &[Ref(request), Ref(function_ref)]);

    return future;
}
