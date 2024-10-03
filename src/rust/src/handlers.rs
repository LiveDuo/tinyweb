
use crate::bindings::utils::random;
use crate::js::ExternRef;

use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::{HashMap, LinkedList},
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll, Waker}
};

pub struct EventHandler<T> {
    pub listeners: RefCell<Option<HashMap<Rc<ExternRef>, Box<dyn FnMut(T) + 'static>>>>,
}

impl<T> EventHandler<T> {
    pub fn add_listener(&self, id: Rc<ExternRef>, handler: Box<dyn FnMut(T) + 'static>) {
        let mut handlers = self.listeners.borrow_mut();
        if let Some(h) = handlers.as_mut() {
            h.insert(id, handler);
        } else {
            let mut h = HashMap::new();
            h.insert(id, handler);
            *handlers = Some(h);
        }
    }

    pub fn remove_listener(&self, id: &Rc<ExternRef>) {
        let mut handlers = self.listeners.borrow_mut();
        if let Some(h) = handlers.as_mut() {
            h.remove(id);
        }
    }

    pub fn call(&self, id: i64, event: T) {
        let mut handlers = self.listeners.borrow_mut();
        if let Some(h) = handlers.as_mut() {
            for (key, handler) in h.iter_mut() {
                if key.value == id as u32 {
                    handler(event);
                    return;
                }
            }
        }
    }
}

pub struct EventHandlerFuture<T> { shared_state: Arc<Mutex<EventHandlerSharedState<T>>>, }

pub struct EventHandlerSharedState<T> { completed: bool, waker: Option<Waker>, result: Option<T>, }

impl<T> Future for EventHandlerFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed && shared_state.result.is_some() {
            let r = shared_state.result.take();
            Poll::Ready(r.unwrap())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

pub struct SharedStateMap<T> {
    map: Mutex<HashMap<u32, Arc<Mutex<EventHandlerSharedState<T>>>>>,
}

impl<T> Default for SharedStateMap<T> {
    fn default() -> Self {
        Self { map: Mutex::new(HashMap::new()), }
    }
}

impl<T> SharedStateMap<T> {
    pub fn add_shared_state(&self, id: u32, state: Arc<Mutex<EventHandlerSharedState<T>>>) {
        let mut map = self.map.lock().unwrap();
        map.insert(id as u32, state);
    }
    pub fn wake_future(&self, id: u32, result: T) {
        let mut waker = None;
        {
            let mut map = self.map.lock().unwrap();
            if let Some(state) = map.remove(&(id as u32)) {
                let mut shared_state = state.lock().unwrap();
                shared_state.completed = true;
                shared_state.result = Some(result);
                waker = shared_state.waker.take();
            }
        }
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

static GLOBALS_LIST: Mutex<LinkedList<(TypeId, &'static Mutex<dyn Any + Send + Sync>)>> = Mutex::new(LinkedList::new());

pub fn globals_get<T: Default + Send + Sync + 'static>() -> MutexGuard<'static, T> {
    {
        let mut globals = GLOBALS_LIST.lock().unwrap();
        let id = TypeId::of::<T>();
        if let Some(v) = globals.iter().find(|&r| r.0 == id) {
            let m = unsafe { &*(v.1 as *const Mutex<dyn Any + Send + Sync> as *const Mutex<T>) };
            return m.lock().unwrap();
        }
        let v = Box::new(Mutex::new(T::default()));
        let handle = Box::leak(v);
        globals.push_front((id, handle));
    }
    globals_get()
}

// https://rust-lang.github.io/async-book/02_execution/03_wakeups.html
impl <T: Send + Sync + 'static> EventHandlerFuture<T> {
    pub fn create_future_with_state_id() -> (Self, u32) {
        let state = EventHandlerSharedState { completed: false, waker: None, result: None, };
        let shared_state = Arc::new(Mutex::new(state));

        let id = (random() * std::f32::MAX) as u32;
        let state_storage = globals_get::<SharedStateMap<T>>();
        state_storage.add_shared_state(id, shared_state.clone());

        (EventHandlerFuture { shared_state: shared_state.clone(), }, id)
    }

    pub fn wake_future_with_state_id(id: u32, result: T) {
        let state_storage = globals_get::<SharedStateMap<T>>();
        state_storage.wake_future(id, result);
    }
}


#[cfg(test)]
mod tests {

    use crate::js::ExternRef;

    use super::*;

    thread_local! {
        static EVENT_HANDLER: EventHandler<()> = EventHandler { listeners: RefCell::new(None), };
    }

    #[test]
    fn test_run() {
 
        let has_run = Rc::new(RefCell::new(false));
        let has_run_clone = has_run.clone();

        // add listener
        let function_handle = Rc::new(ExternRef { value: 0, });
        let handler = move |_| {
            *has_run_clone.borrow_mut() = true;
        };
        EVENT_HANDLER.with(|s| s.add_listener(function_handle.clone(), Box::new(handler)));

        // call listener
        EVENT_HANDLER.with(|s| s.call(0, ()));
        assert_eq!(*has_run.borrow(), true);

        // remove listener
        EVENT_HANDLER.with(|s| s.remove_listener(&function_handle.clone()));
        let count = EVENT_HANDLER.with(|s| {
            s.listeners.borrow().as_ref().map(|s| s.len()).unwrap_or(0)
        });
        assert_eq!(count, 0);
    }

}


