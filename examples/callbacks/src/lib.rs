
use tinyweb::callbacks::{create_async_callback, create_callback};
use tinyweb::runtime::Runtime;
use tinyweb::invoke::*;

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| { Js::invoke("console.log({})", &[e.to_string().into()]); }));

    // invoke
    Js::invoke("console.log('invoke')", &[]);

    // invoke callback
    let function_ref = create_callback(move |_s| { Js::invoke("console.log('invoke timer')", &[]); });
    Js::invoke("setTimeout({}, 1000)", &[function_ref.into()]);

    // invoke async callback
    let url = "https://pokeapi.co/api/v2/pokemon/1";
    let (callback_ref, future) = create_async_callback();
    Js::invoke("fetch({}).then(r => r.json()).then(r => { {}(r) })", &[url.into(), callback_ref.into()]);
    Runtime::block_on(async move {
        let future_leak = Box::leak(Box::new(future));
        let object_ref = future_leak.await;
        let result = Js::invoke("return {}.name", &[object_ref.into()]).to_str().unwrap();
        Js::invoke("console.log('invoke fetch', {})", &[result.into()]);
        Js::deallocate(object_ref);
    });
}
