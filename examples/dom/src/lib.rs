
use tinyweb::callbacks::create_callback;
use tinyweb::invoke::*;

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| { Js::invoke("console.log({})", &[e.to_string().into()]); }));

    let button = Js::invoke("return document.createElement('button')", &[]).to_ref().unwrap();
    Js::invoke("{}.textContent = 'Click'", &[button.into()]);

    let function_ref = create_callback(move |e| { Js::invoke("alert('hello')", &[]); Js::deallocate(e); });
    Js::invoke("{}.addEventListener('click',{})", &[button.into(), function_ref.into()]);

    let body = Js::invoke("return document.querySelector('body')", &[]).to_ref().unwrap();
    Js::invoke("{}.appendChild({})", &[body.into(), button.into()]);
}
