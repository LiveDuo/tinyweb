
use std::collections::HashMap;
use std::ops::Deref;

use crate::bindings::dom::{self, MouseEvent};
use crate::bindings::history;
use crate::js::ExternRef;

#[derive(Debug, Clone)]
pub struct El(ExternRef);

impl Deref for El {
    type Target = ExternRef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl El {
    pub fn new(tag: &str) -> El {
        let el = dom::create_element(tag);
        El(el)
    }
    pub fn mount(&self, parent: &ExternRef) {
        dom::append_child(&parent, self);
    }
    pub fn attr(self, name: &str, value: &str) -> Self {
        dom::element_set_attribute(&self, name, value);
        self
    }
    pub fn classes(self, classes: &[&str]) -> Self {
        classes.iter().for_each(|c| { dom::element_add_class(&self, c); });
        self
    }
    pub fn child(self, child: El) -> Self {
        dom::append_child(&self, &child);
        self
    }
    pub fn children(self, children: &[El]) -> Self {
        for child in children {
            dom::append_child(&self, &child);
        }
        self
    }
    pub fn on_mount(self, mut cb: impl FnMut(&Self) + 'static) -> Self {
        cb(&self);
        self
    }
    pub fn on_click(self, cb: impl FnMut(MouseEvent) + 'static) -> Self {
        dom::element_add_click_listener(&self, cb);
        self
    }
    pub fn text(self, text: &str) -> Self {
        
        let el = dom::create_text_node(text);
        dom::append_child(&self, &el);

        self
    }
}


#[derive(Debug, Default)]
pub struct Router { pub root: Option<ExternRef>, pub pages: HashMap::<String, (El, Option<String>)> }

impl Router {
    pub fn navigate(&self, page: &str) {
        
        let (el, title) = self.pages.get(page).unwrap();
        history::history_push_state(&title.to_owned().unwrap_or_default(), page);

        let body = self.root.as_ref().unwrap();
        dom::element_set_inner_html(&body, "");
        
        el.mount(&body);
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;

    #[test]
    fn test_element() {
        
        El::new("div")
            .classes(&[])
            .child(El::new("button").text("button 1"))
            .child(El::new("button").text("button 2"));
    }

}