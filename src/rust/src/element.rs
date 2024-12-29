
use std::future::Future;
use std::cell::RefCell;
use std::rc::Rc;

use crate::invoke::{Js, ObjectRef};
use crate::runtime::Runtime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct El { pub element: ObjectRef, pub callbacks: RefCell<Vec<ObjectRef>> }

impl El {
    pub fn new(tag: &str) -> Self {
        let el = Js::invoke("return document.createElement({})", &[tag.into()]).to_ref().unwrap();
        Self { element: el, callbacks: RefCell::new(vec![]) }
    }
    pub fn from(el: &ObjectRef) -> Self {
        Self { element: el.to_owned(), callbacks: RefCell::new(vec![]) }
    }
    pub fn mount(&self, parent: &ObjectRef) {
        Js::invoke("{}.appendChild({})", &[parent.into(), self.element.into()]);
    }
    pub fn unmount(&self) {
        let mut c = self.callbacks.borrow_mut();
        c.iter().for_each(|p| {
            crate::callbacks::CALLBACK_HANDLERS.with(|s| { let _ = s.borrow_mut().remove(p).unwrap(); });
        });
        c.clear();
    }
    pub fn attr(self, name: &str, value: &str) -> Self {
        Js::invoke("{}.setAttribute({},{})", &[self.element.into(), name.into(), value.into()]);
        self
    }
    pub fn attr_fn(self, name: &str, value: &str, cb: impl Fn() -> bool + 'static) -> Self {
        if cb() {
            Js::invoke("{}.setAttribute({},{})", &[self.element.into(), name.into(), value.into()]);
        }
        self
    }
    pub fn classes(self, classes: &[&str]) -> Self {
        classes.iter().for_each(|&c| { Js::invoke("{}.classList.add({})", &[self.element.into(), c.into()]); });
        self
    }
    pub fn child(self, child: Self) -> Self {
        Js::invoke("{}.appendChild({})", &[self.element.into(), child.element.into()]);
        self
    }
    pub fn children(self, children: &[Self]) -> Self {
        Js::invoke("{}.innerHTML = {}", &[self.element.into(), "".into()]);
        for child in children {
            Js::invoke("{}.appendChild({})", &[self.element.into(), child.element.into()]);
        }
        self
    }
    pub fn on_mount(self, mut cb: impl FnMut(&'static Self) + 'static) -> Self {
        cb(Box::leak(Box::new(self.to_owned())));
        self
    }
    pub fn on_mount_async<Fut: Future<Output = ()>>(self, mut cb: impl FnMut(&'static Self) -> Fut + 'static) -> Self {

        let self_leak = Box::leak(Box::new(self.to_owned()));

        Runtime::block_on(async move {
            cb(self_leak).await;
        });

        self
    }
    pub fn on_event(self, event: &str, cb: impl FnMut(ObjectRef) + 'static) -> Self {

        let function_ref = crate::callbacks::create_callback(cb);
        let code = &format!("{{}}.addEventListener('{}',{{}})", event);
        Js::invoke(code, &[self.element.into(), function_ref.into()]);

        self.callbacks.borrow_mut().push(function_ref);

        self
    }
    pub fn on_event_async<Fut: Future<Output = ()>>(self, event: &str, cb: impl FnMut(ObjectRef) -> Fut + 'static) -> Self {

        let cb = Rc::new(RefCell::new(cb));
        let cb_async = move |e| {
            let cb = cb.clone();

            Runtime::block_on(async move {
                cb.borrow_mut()(e).await;
            });
        };

        let function_ref = crate::callbacks::create_callback(cb_async);
        let code = &format!("{{}}.addEventListener('{}',{{}})", event);
        Js::invoke(code, &[self.element.into(), function_ref.into()]);

        self.callbacks.borrow_mut().push(function_ref);

        self
    }
    pub fn text(self, text: &str) -> Self {

        let el = Js::invoke("return document.createTextNode({})", &[text.into()]).to_ref().unwrap();
        Js::invoke("{}.appendChild({})", &[self.element.into(), el.into()]);

        self
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_element() {

        let el = El::new("div").classes(&[])
            .child(El::new("button").text("button 1"))
            .child(El::new("button").text("button 2"));
        assert_eq!(el, El::from(&ObjectRef::new(0)));

    }

}
