
use std::sync::{Arc, Mutex};
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

type FnRefAsync = Arc<Mutex<dyn FnMut() + Send + 'static>>;

#[derive(Clone)]
pub struct SignalAsync<T> { value: Arc<Mutex<T>>, subscribers: Arc<Mutex<Vec<FnRefAsync>>> }

impl<T: Clone + Send + 'static> SignalAsync<T> {
    pub fn new(value: T) -> Self {
        Self { value: Arc::new(Mutex::new(value)), subscribers: Default::default(), }
    }
    pub fn get(&self) -> T {
        self.value.lock().unwrap().to_owned()
    }
    pub fn set(&self, new_value: T) {
        // store value
        *self.value.lock().unwrap() = new_value;

        // trigger effects
        self.subscribers.lock().unwrap().iter().for_each(|e| { e.lock().unwrap()(); })
    }
    pub fn on(&self, mut cb: impl FnMut(T) + Send + 'static) {

        // get callback
        let signal_clone = self.clone();
        let cb_ref = Arc::new(Mutex::new(move || { cb(signal_clone.get()); }));

        // store callback
        self.subscribers.lock().unwrap().push(cb_ref.to_owned());

        // trigger once
        cb_ref.lock().unwrap()();
    }
}

#[cfg(test)]
mod tests {

    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn test_signals() {

        // create signal
        let logs: Rc<RefCell<Vec<i32>>> = Default::default();
        let signal = Signal::new(10);
        
        // create effects
        let logs_clone = logs.clone();
        signal.on(move |v| { logs_clone.borrow_mut().push(v); });
        let logs_clone = logs.clone();
        signal.on(move |v| { logs_clone.borrow_mut().push(v); });
        
        // update signal
        signal.set(20);
        signal.set(30);

        // check logs
        let received = logs.borrow().clone();
        assert_eq!(received, vec![10, 10, 20, 20, 30, 30]);
    }
    
    #[test]
    fn test_signals_async() {

        // create signal
        let logs: Arc<Mutex<Vec<i32>>> = Default::default();
        let signal = SignalAsync::new(10);
        
        // create effects
        let logs_clone = logs.clone();
        signal.on(move |v| { logs_clone.lock().unwrap().push(v); });
        let logs_clone = logs.clone();
        signal.on(move |v| { logs_clone.lock().unwrap().push(v); });
        
        // update signal
        signal.set(20);
        signal.set(30);

        // check logs
        let received = logs.lock().unwrap().clone();
        assert_eq!(received, vec![10, 10, 20, 20, 30, 30]);
    }

}


