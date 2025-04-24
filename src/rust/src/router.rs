

use std::collections::HashMap;

use crate::invoke::{Js, ObjectRef};
use crate::element::El;

#[derive(Debug, Clone)]
pub struct Page { pub path: String, pub element: El, pub title: Option<String> }

impl Page {
    pub fn new(path: &str, element: El) -> Self {
        Self { path: path.to_owned(), element, title: None }
    }
    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }
}

#[derive(Debug, Default)]
pub struct Router { pub root: Option<ObjectRef>, pub pages: HashMap::<String, Page> }

impl Router {
    pub fn new(root: &str, pages: &[Page]) -> Self {
        let body = Js::invoke("return document.querySelector({})", &[root.into()]).to_ref().unwrap();
        let pathname = Js::invoke("return window.location.pathname", &[]).to_str().unwrap();
        let page = pages.iter().find(|&s| *s.path == pathname).unwrap_or(&pages[0]);
        page.element.mount(&body);

        let mut default_page = pages.first().cloned().unwrap();
        default_page.path = "/".to_owned();

        let mut pages = pages.iter().map(|p| (p.path.clone(), p.to_owned())).collect::<Vec<_>>();
        pages.push((default_page.path.clone(), default_page.to_owned()));
        Self { pages: HashMap::from_iter(pages), root: Some(body) }
    }
    pub fn navigate(&self, route: &str) {

        // unmount page
        let pathname = Js::invoke("return window.location.pathname", &[]).to_str().unwrap();
        let (_, current_page) = self.pages.iter().find(|&(s, _)| *s == pathname).unwrap();
        current_page.element.unmount();

        // set html
        let body = self.root.as_ref().unwrap();
        Js::invoke("{}.innerHTML = {}", &[body.into(), "".into()]);

        // mount new page
        let page = self.pages.get(route).unwrap();
        page.element.mount(&body);

        // push state
        let page_str = page.title.to_owned().unwrap_or_default();
        Js::invoke("window.history.pushState({ },{},{})", &[page_str.into(), route.into()]);

    }
}
