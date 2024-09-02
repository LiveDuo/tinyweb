
use crate::bindings::util::random_i64;
use crate::js::ExternRef;

use std::{
    any::{Any, TypeId}, cell::{RefCell, RefMut}, collections::{HashMap, LinkedList}, future::Future, pin::Pin, rc::Rc, task::{Context, Poll, Waker}
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
                if key.value == id {
                    handler(event);
                    return;
                }
            }
        }
    }
}

pub struct EventHandlerFuture<T> { shared_state: Rc<RefCell<EventHandlerSharedState<T>>>, }

pub struct EventHandlerSharedState<T> { completed: bool, waker: Option<Waker>, result: Option<T>, }

impl<T> Future for EventHandlerFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.borrow_mut();
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
    map: RefCell<HashMap<i64, Rc<RefCell<EventHandlerSharedState<T>>>>>,
}

impl<T> Default for SharedStateMap<T> {
    fn default() -> Self {
        Self { map: RefCell::new(HashMap::new()), }
    }
}

impl<T> SharedStateMap<T> {
    pub fn add_shared_state(&self, id: i64, state: Rc<RefCell<EventHandlerSharedState<T>>>) {
        let mut map = self.map.borrow_mut();
        map.insert(id, state);
    }
    pub fn wake_future(&self, id: i64, result: T) {
        let mut waker = None;
        {
            let mut map = self.map.borrow_mut();
            if let Some(state) = map.remove(&id) {
                let mut shared_state = state.borrow_mut();
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

type Global = LinkedList<(TypeId, &'static RefCell<dyn Any>)>;

thread_local! {
    static GLOBALS_LIST: RefCell<Global> = RefCell::new(LinkedList::new());
}

pub fn globals_get<T: Default + 'static>() -> RefMut<'static, T> {
    
    GLOBALS_LIST.with_borrow_mut(|g| {

        let id = TypeId::of::<T>();
        let p = g.iter().find(|&r| r.0 == id);
        if let Some(v) = p {
            let m = unsafe { &*(v.1 as *const RefCell<dyn Any> as *const RefCell<T>) };
            return m.borrow_mut();
        }
        let v = Box::new(RefCell::new(T::default()));
        let handle = Box::leak(v);
        g.push_front((id, handle));
        
        handle.borrow_mut()
    })

}

impl <T: 'static> EventHandlerFuture<T> {
    pub fn create_future_with_state_id() -> (Self, i64) {
        let shared_state = Rc::new(RefCell::new(EventHandlerSharedState {
            completed: false,
            waker: None,
            result: None,
        }));

        let id = random_i64();
        let state_storage = globals_get::<SharedStateMap<T>>();
        state_storage.add_shared_state(id, shared_state.clone());

        (
            EventHandlerFuture { shared_state: shared_state.clone(), },
            id,
        )
    }

    pub fn wake_future_with_state_id(id: i64, result: T) {
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


