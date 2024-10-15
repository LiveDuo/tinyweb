
use tinyweb::bindings::{console, dom};

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| console::console_log(&e.to_string())));

    let button = dom::create_element("button");
    let button_text = dom::create_text_node("Click");
    dom::append_child(&button, &button_text);
    dom::element_add_click_listener(&button, move |_s| {
        let button = dom::create_element("button");
        dom::element_add_click_listener(&button, |_s| {}); // thows error
    });

    let body = dom::query_selector("body");
    dom::append_child(&body, &button);
}
