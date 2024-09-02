
use std::cell::RefCell;
use std::rc::Rc;

// NOTE: since `Fn` can mutate state it has to go behind a smart pointer
type FnRef = Rc<RefCell<dyn FnMut()>>;

#[derive(Clone)]
pub struct Signal<T> { value: Rc<RefCell<T>>, subscribers: Rc<RefCell<Vec<FnRef>>> }

impl<T: Clone + 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        Self { value: Rc::new(RefCell::new(value)), subscribers: Default::default(), }
    }
    pub fn get(&self) -> T {
        self.value.borrow().to_owned()
    }
    pub fn set(&self, new_value: T) {
        // store value
        *self.value.borrow_mut() = new_value;

        // trigger effects
        self.subscribers.borrow().iter().for_each(|e| { e.borrow_mut()(); })
    }
    pub fn on(&self, mut cb: impl FnMut(T) + 'static) {

        // get callback
        let signal_clone = self.clone();
        let cb_ref = Rc::new(RefCell::new(move || { cb(signal_clone.get()); }));

        // store callback
        self.subscribers.borrow_mut().push(cb_ref.to_owned());

        // trigger once
        cb_ref.borrow_mut()();
    }
}

#[cfg(test)]
mod tests {

    use std::sync::{Arc, Mutex};

    use crate::signals::Signal;

    #[test]
    fn test_signals() {

        // create signal
        let logs = Arc::new(Mutex::new(vec![]));
        let signal = Signal::new(10);
        
        // create effects
        let logs_clone = logs.clone();
        signal.on(move |v| {
            logs_clone.lock().map(|mut s| { s.push(v); }).unwrap();
        });
        let logs_clone = logs.clone();
        signal.on(move |v| {
            logs_clone.lock().map(|mut s| { s.push(v); }).unwrap();
        });
        
        // update signal
        signal.set(20);
        signal.set(30);

        // check logs
        let received = logs.lock().map(|s| s.clone()).unwrap();
        assert_eq!(received, vec![10, 10, 20, 20, 30, 30]);
    }

}


