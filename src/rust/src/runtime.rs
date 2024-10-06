use std::{
    any::{Any, TypeId},
    collections::{HashMap, LinkedList, VecDeque},
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker}
};

use crate::bindings::window::set_timeout;
use crate::bindings::utils::random;



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

impl<T> SharedStateMap<T> {
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

pub fn globals_get<T: Send + Sync + 'static>() -> MutexGuard<'static, SharedStateMap<T>> {

    let mut globals = GLOBALS_LIST.lock().unwrap();
    let id = TypeId::of::<SharedStateMap<T>>();

    let mutex = if let Some(v) = globals.iter().find(|&r| r.0 == id) {
        unsafe { &*(v.1 as *const Mutex<dyn Any + Send + Sync> as *const Mutex<SharedStateMap<T>>) }
    } else {
        let v = Box::new(Mutex::new(SharedStateMap { map: Mutex::new(HashMap::new()), }));
        let leaked = Box::leak(v);
        globals.push_front((id, leaked));
    
        unsafe { &*(leaked as *const Mutex<SharedStateMap<T>>) }
    };
    return mutex.lock().unwrap();
}

// https://rust-lang.github.io/async-book/02_execution/03_wakeups.html
impl <T: Send + Sync + 'static> EventHandlerFuture<T> {
    pub fn create_future_with_state_id() -> (Self, u32) {
        let state = EventHandlerSharedState { completed: false, waker: None, result: None, };
        let shared_state = Arc::new(Mutex::new(state));

        let id = (random() * std::f32::MAX) as u32;
        let state_storage = globals_get::<T>();
        state_storage.map.lock().map(|mut s| {
            s.insert(id as u32, shared_state.clone());
        }).unwrap();

        (EventHandlerFuture { shared_state: shared_state.clone(), }, id)
    }

    pub fn wake_future_with_state_id(id: u32, result: T) {
        let state_storage = globals_get::<T>();
        state_storage.wake_future(id, result);
    }
}

fn simple_waker<T>(task: &Arc<Task<T>>) -> Waker {

    fn clone_fn<T>(data: *const ()) -> RawWaker {
        let _arc = ManuallyDrop::new(unsafe { Arc::<T>::from_raw(data as *const T) }).clone();
        RawWaker::new(data, waker_vtable::<T>())
    }
    fn wake_fn(_data: *const ()) {
        set_timeout(|| { DEFAULT_RUNTIME.lock().unwrap().poll_tasks(); }, 0);
    }
    fn drop_fn<T>(data: *const ()) {
        unsafe { drop(Arc::<T>::from_raw(data as *const T)) }
    }
    fn waker_vtable<T>() -> &'static RawWakerVTable {
        &RawWakerVTable::new(clone_fn::<T>, wake_fn, wake_fn, drop_fn::<T>)
    }
    let ptr = (&**task as *const Task<T>) as *const ();
    let raw_waker = RawWaker::new(ptr, waker_vtable::<Task<T>>());
    unsafe { Waker::from_raw(raw_waker) }
}

struct Task<T> { future: Mutex<Pin<Box<dyn Future<Output = T> + Send + 'static>>>, }

trait Pendable {
    fn is_pending(&self) -> bool;
}

impl<T> Pendable for Arc<Task<T>> {
    fn is_pending(&self) -> bool {
        let mut future = self.future.lock().unwrap();
        let waker = ManuallyDrop::new(simple_waker::<T>(self));
        let context = &mut Context::from_waker(&waker);
        matches!(future.as_mut().poll(context), Poll::Pending)
    }
}

pub struct Runtime { tasks: VecDeque<Box<dyn Pendable + Send>> }

impl Runtime {

    fn poll_tasks(&mut self) {
        for _ in 0..self.tasks.len() {
            let task = self.tasks.pop_front().unwrap();
            if task.is_pending() {
                self.tasks.push_back(task);
            }
        }
    }
}

static DEFAULT_RUNTIME: Mutex<Runtime> = Mutex::new(Runtime { tasks: VecDeque::new() });

pub fn run<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) {
    DEFAULT_RUNTIME.lock().map(|mut s| {
        let task = Task { future: Mutex::new(Box::pin(future)) };
        s.tasks.push_back(Box::new(Arc::new(task)));
        s.poll_tasks();
    }).unwrap();
}

pub fn coroutine<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) {
    let mut a = Some(Box::pin(future));
    set_timeout(move || { if let Some(b) = a.take() { run(b); } }, 0);
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_run() {
        let has_run = Arc::new(Mutex::new(false));
        let has_run_clone = has_run.clone();
        let future = async move {
            has_run_clone.lock().map(|mut s| { *s = true; }).unwrap();
        };
        run(future);
        assert_eq!(*has_run.lock().unwrap(), true);
    }

    #[test]
    fn test_future() {

        run(async move {
            let (future, state_id) = EventHandlerFuture::<bool>::create_future_with_state_id();
            assert_eq!(future.shared_state.lock().map(|s| s.result).unwrap(), None);
            
            EventHandlerFuture::<bool>::wake_future_with_state_id(state_id, true);
            assert_eq!(future.shared_state.lock().map(|s| s.result).unwrap(), Some(true));
            
            let result = future.await;
            assert_eq!(result, true);
        });

    }

}
