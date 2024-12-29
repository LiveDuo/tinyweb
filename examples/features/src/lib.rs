
mod keycodes;

use std::cell::RefCell;

use json::JsonValue;

use tinyweb::callbacks::{create_async_callback, promise};
use tinyweb::router::{Page, Router};
use tinyweb::runtime::Runtime;
use tinyweb::signals::Signal;
use tinyweb::element::El;

use tinyweb::invoke::*;

const BUTTON_CLASSES: &[&str] = &["bg-blue-500", "hover:bg-blue-700", "text-white", "p-2", "rounded", "m-2"];

thread_local! {
    pub static ROUTER: RefCell<Router> = RefCell::new(Router::default());
}

async fn fetch_json(method: &str, url: &str, body: Option<JsonValue>) -> Result<JsonValue, String> {
    let body = body.map(|s| s.dump()).unwrap_or_default();
    let (callback_ref, future) = create_async_callback();
    let request = r#"
        const options = { method: {}, headers: { 'Content-Type': 'application/json' }, body: p0 !== 'GET' ? {} : null };
        fetch({}, options).then(r => r.json()).then(r => { {}(r) })
    "#;
    Js::invoke(request, &[method.into(), body.into(), url.into(), callback_ref.into()]);
    let result_ref = future.await;
    let result = Js::invoke("return JSON.stringify({})", &[result_ref.into()]).to_str().unwrap();
    json::parse(&result).map_err(|_| "Parse error".to_owned())
}

fn page1() -> El {

    // signals
    let signal_key = Signal::new("-".to_owned());
    let signal_count = Signal::new(0);
    let signal_time = Signal::new("-");

    El::new("div")
        .on_mount(move |_| {

            // add listener
            let body = Js::invoke("return document.querySelector({})", &["body".into()]).to_ref().unwrap();

            El::from(&body).on_event("keydown", move |e| {
                let key_code = Js::invoke("return {}[{}]", &[e.into(), "keyCode".into()]).to_num().unwrap();
                let key_name = keycodes::KEYBOARD_MAP[key_code as usize];
                let text = format!("Pressed: {}", key_name);
                signal_key.set(text);
            });

            // start timer
            Runtime::block_on(async move {
                loop {
                    signal_time.set("⏰ tik");
                    promise("window.setTimeout({},{})", move |c| vec![c.into(), 1_000.into()]).await;
                    signal_time.set("⏰ tok");
                    promise("window.setTimeout({},{})", move |c| vec![c.into(), 1_000.into()]).await;
                }
            });

        })
        .classes(&["m-2"])
        .child(El::new("button").text("api").classes(&BUTTON_CLASSES).on_event_async("click", move |_| async {
            let url = format!("https://pokeapi.co/api/v2/pokemon/{}", 1);
            let result = fetch_json("GET", &url, None).await.unwrap();
            let name = result["name"].as_str().unwrap();
            Js::invoke("alert({})", &[name.into()]);
        }))
        .child(El::new("button").text("page 2").classes(&BUTTON_CLASSES).on_event("click", move |_| {
            ROUTER.with(|s| { s.borrow().navigate("/page2"); });
        }))
        .child(El::new("br"))
        .child(El::new("button").text("add").classes(&BUTTON_CLASSES).on_event("click", move |_| {
            let count = signal_count.get() + 1;
            signal_count.set(count);
        }))
        .child(El::new("div").text("0").on_mount(move |el| {
            signal_count.on(move |v| { Js::invoke("{}.innerHTML = {}", &[el.element.into(), v.to_string().into()]); });
        }))
        .child(El::new("div").text("-").on_mount(move |el| {
            signal_time.on(move |v| { Js::invoke("{}.innerHTML = {}", &[el.element.into(), v.into()]); });
        }))
        .child(El::new("div").text("-").on_mount(move |el| {
            signal_key.on(move |v| { Js::invoke("{}.innerHTML = {}", &[el.element.into(), v.into()]); });
        }))
}

fn page2() -> El {
    El::new("div")
        .classes(&["m-2"])
        .child(El::new("button").text("page 1").classes(&BUTTON_CLASSES).on_event("click", move |_| {
            ROUTER.with(|s| { s.borrow().navigate("/page1"); });
        }))
}

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| { Js::invoke("console.log({})", &[e.to_string().into()]); }));

    // init router
    let pages = &[Page::new("/page1", page1(), None), Page::new("/page2", page2(), None)];
    ROUTER.with(|s| { *s.borrow_mut() = Router::new("body", pages); });
}
