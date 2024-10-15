
use tinyweb::signals::Signal;

use tinyweb::bindings::{console, dom};

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| console::console_log(&e.to_string())));

    let signal = Signal::new("title".to_owned());
    signal.on(|_v| {
        let button = dom::create_element("button");
        dom::element_add_click_listener(&button, |_s| {}); // thows error
    });

    let signal_clone = signal.clone();
    let button = dom::create_element("button");
    let button_text = dom::create_text_node("Click");
    dom::append_child(&button, &button_text);
    dom::element_add_click_listener(&button, move |_s| { signal_clone.set("2".to_owned()); });

    let body = dom::query_selector("body");
    dom::append_child(&body, &button);
}
