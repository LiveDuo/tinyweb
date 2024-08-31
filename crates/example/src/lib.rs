
use web::bindings::dom;

#[no_mangle]
pub fn main() {
    let body = dom::query_selector("body");
    dom::element_set_inner_html(&body, "hello");
}
