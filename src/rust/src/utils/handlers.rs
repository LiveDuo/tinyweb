
use crate::bindings::util::random_i64;
use crate::utils::js::ExternRef;

use std::{
    collections::{HashMap, LinkedList},
    hash::{Hash, Hasher},
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
    sync::{Arc, Mutex, MutexGuard},
    any::{Any, TypeId},
};

pub struct FunctionHandle(pub ExternRef);

impl PartialEq for FunctionHandle {
    fn eq(&self, other: &Self) -> bool {
        self.0.value == other.0.value
    }
}

impl Eq for FunctionHandle {}

impl Hash for FunctionHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.value.hash(state);
    }
}

pub struct EventHandler<T> {
    pub listeners: Mutex<Option<HashMap<Arc<FunctionHandle>, Box<dyn FnMut(T) + Send + 'static>>>>,
}

impl<T> EventHandler<T> {
    pub fn add_listener(
        &self,
        id: Arc<FunctionHandle>,
        handler: Box<dyn FnMut(T) + Send + 'static>,
    ) {
        let mut handlers = self.listeners.lock().unwrap();
        if let Some(h) = handlers.as_mut() {
            h.insert(id, handler);
        } else {
            let mut h = HashMap::new();
            h.insert(id, handler);
            *handlers = Some(h);
        }
    }

    pub fn remove_listener(&self, id: &Arc<FunctionHandle>) {
        let mut handlers = self.listeners.lock().unwrap();
        if let Some(h) = handlers.as_mut() {
            h.remove(id);
        }
    }

    pub fn call(&self, id: i64, event: T) {
        let mut handlers = self.listeners.lock().unwrap();
        if let Some(h) = handlers.as_mut() {
            for (key, handler) in h.iter_mut() {
                if key.0.value == id {
                    handler(event);
                    return;
                }
            }
        }
    }
}

pub struct EventHandlerFuture<T> {
    shared_state: Arc<Mutex<EventHandlerSharedState<T>>>,
}

pub struct EventHandlerSharedState<T> {
    completed: bool,
    waker: Option<Waker>,
    result: Option<T>,
}

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
    map: Mutex<HashMap<i64, Arc<Mutex<EventHandlerSharedState<T>>>>>,
}

impl<T> Default for SharedStateMap<T> {
    fn default() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }
}

impl<T> SharedStateMap<T> {
    pub fn add_shared_state(&self, id: i64, state: Arc<Mutex<EventHandlerSharedState<T>>>) {
        let mut map = self.map.lock().unwrap();
        map.insert(id, state);
    }
    pub fn wake_future(&self, id: i64, result: T) {
        let mut waker = None;
        {
            let mut map = self.map.lock().unwrap();
            if let Some(state) = map.remove(&id) {
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

static GLOBALS_LIST: Mutex<LinkedList<(TypeId, &'static Mutex<dyn Any + Send + Sync>)>> =
    Mutex::new(LinkedList::new());

pub fn globals_get<T>() -> MutexGuard<'static, T>
where
    T: 'static + Default + Send + core::marker::Sync,
{
    {
        let mut globals = GLOBALS_LIST.lock().unwrap();
        let id = TypeId::of::<T>();
        let p = globals.iter().find(|&r| r.0 == id);
        if let Some(v) = p {
            let m = unsafe { &*(v.1 as *const Mutex<dyn Any + Send + Sync> as *const Mutex<T>) };
            return m.lock().unwrap();
        }
        let v = Box::new(Mutex::new(T::default()));
        let handle = Box::leak(v);
        globals.push_front((id, handle));
    }
    globals_get()
}

impl<T> EventHandlerFuture<T>
where
    T: 'static + Sync + Send,
{
    pub fn create_future_with_state_id() -> (Self, i64) {
        let shared_state = Arc::new(Mutex::new(EventHandlerSharedState {
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
