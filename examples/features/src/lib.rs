
mod keycodes;

use std::cell::RefCell;
use std::collections::HashMap;

use json::JsonValue;
use tinyweb::element::{El, Router, Page};
use tinyweb::signals::Signal;

use tinyweb::bindings::{console, dom, http_request, history};
use tinyweb::bindings::http_request::*;

const BUTTON_CLASSES: &[&str] = &["bg-blue-500", "hover:bg-blue-700", "text-white", "p-2", "rounded", "m-2"];

thread_local! {
    pub static ROUTER: RefCell<Router> = RefCell::new(Router::default());
}

async fn fetch_json(method: HTTPMethod, url: String, body: Option<JsonValue>) -> JsonValue {
    let body_temp = body.map(|s| s.dump());
    let body = body_temp.as_ref().map(|s| s.as_str());
    let fetch_options = FetchOptions { action: method, url: &url, body, ..Default::default()};
    let fetch_res = http_request::fetch(fetch_options).await;
    let result = match fetch_res { FetchResponse::Text(_, d) => Ok(d), _ => Err(()), };
    json::parse(&result.unwrap()).unwrap()
}

fn page1() -> El {

    // key signal
    let signal_key = Signal::new("-".to_owned());
    let signal_key_clone = signal_key.clone();

    // count signal
    let signal_count = Signal::new(0);
    let signal_count_clone = signal_count.clone();

    // time signal
    let signal_time = Signal::new("-");
    let signal_time_clone = signal_time.clone();

    El::new("div")
        .on_mount(move |_| {

            // add listener
            let body = dom::query_selector("body");
            let signal_key_clone = signal_key_clone.clone();
            let _keyboard_event = dom::element_add_key_down_listener(&body, move |e| {
                let key_name = keycodes::KEYBOARD_MAP[e.key_code as usize];
                let text = format!("Pressed: {}", key_name);
                signal_key_clone.set(text);
            });

            // start timer
            let signal_time_clone = signal_time_clone.clone();
            tinyweb::runtime::run(async move {
                loop {
                    signal_time_clone.set("⏰ tik");
                    tinyweb::bindings::utils::sleep(1_000).await;
                    signal_time_clone.set("⏰ tok");
                    tinyweb::bindings::utils::sleep(1_000).await;
                }
            });

        })
        .classes(&["m-2"])
        .child(El::new("button").text("api").classes(&BUTTON_CLASSES).on_click(|_| {
            tinyweb::runtime::run(async move {
                let url = format!("https://pokeapi.co/api/v2/pokemon/{}", 1);
                let result = fetch_json(HTTPMethod::GET, url, None).await;
                dom::alert(&result["name"].as_str().unwrap());
            });
        }))
        .child(El::new("button").text("page 2").classes(&BUTTON_CLASSES).on_click(move |_| {
            ROUTER.with(|s| { s.borrow().navigate("page2"); });
        }))
        .child(El::new("br"))
        .child(El::new("button").text("add").classes(&BUTTON_CLASSES).on_click(move |_| {
            let count = signal_count_clone.get() + 1;
            signal_count_clone.set(count);
        }))
        .child(El::new("div").text("0").on_mount(move |el| {
            let el_clone = el.clone();
            signal_count.on(move |v| { dom::element_set_inner_html(&el_clone, &v.to_string()); });
        }))
        .child(El::new("div").text("-").on_mount(move |el| {
            let el_clone = el.clone();
            signal_time.on(move |v| { dom::element_set_inner_html(&el_clone, &v.to_string()); });
        }))
        .child(El::new("div").text("-").on_mount(move |el| {
            let el_clone = el.clone();
            signal_key.on(move |v| { dom::element_set_inner_html(&el_clone, &v.to_string()); });
        }))
}

fn page2() -> El {
    El::new("div")
        .classes(&["m-2"])
        .child(El::new("button").text("page 1").classes(&BUTTON_CLASSES).on_click(move |_| {
            ROUTER.with(|s| { s.borrow().navigate("page1"); });
        }))
}

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| console::console_log(&e.to_string())));

    // get pages
    let pages = [
        ("page1".to_owned(), Page { element: page1(), title: None }),
        ("page2".to_owned(), Page { element: page2(), title: None })
    ];

    // load page
    let body = dom::query_selector("body");
    let (_, page) = pages.iter().find(|&(s, _)| *s == history::location_pathname()).unwrap_or(&pages[0]);
    page.element.mount(&body);

    // init router
    ROUTER.with(|s| {
        *s.borrow_mut() = Router { pages: HashMap::from_iter(pages), root: Some(body) };
    });

}
