
use tinyweb::element::El;
use tinyweb::bindings::{console, dom};

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| console::console_log(&e.to_string())));

    let body = dom::query_selector("body");
    let page = El::new("div")
        .child(El::new("button").text("button 1").on_click(move |_| { console::console_log("1"); }))
        .child(El::new("button").text("button 2").on_click(move |_| { console::console_log("2"); }));
    page.mount(&body);

}
