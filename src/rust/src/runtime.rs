use std::{
    collections::VecDeque,
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker}
};

use crate::bindings::window::set_timeout;

fn clone_arc_raw<T>(data: *const ()) -> RawWaker {
    let _arc = unsafe { ManuallyDrop::new(Arc::<T>::from_raw(data as *const T)).clone() };
    RawWaker::new(data, waker_vtable::<T>())
}
fn wake_arc_raw(_data: *const ()) {
    set_timeout(|| { DEFAULT_RUNTIME.lock().unwrap().poll_tasks(); }, 0);
}
fn drop_arc_raw<T>(data: *const ()) {
    unsafe { drop(Arc::<T>::from_raw(data as *const T)) }
}
fn waker_vtable<T>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(clone_arc_raw::<T>, wake_arc_raw, wake_arc_raw, drop_arc_raw::<T>)
}

trait Pendable {
    fn is_pending(&self) -> bool;
}

pub struct Runtime {
    tasks: VecDeque<Box<dyn Pendable + Send>>,
}

struct Task<T> {
    future: Mutex<Pin<Box<dyn Future<Output = T> + Send + 'static>>>,
}

impl<T> Pendable for Arc<Task<T>> {
    fn is_pending(&self) -> bool {
        let mut future = self.future.lock().unwrap();

        let ptr = (&**self as *const Task<T>) as *const ();
        let raw_waker = RawWaker::new(ptr, waker_vtable::<Task<T>>());
        let waker = ManuallyDrop::new(unsafe { Waker::from_raw(raw_waker) });
        let context = &mut Context::from_waker(&*waker);
        matches!(future.as_mut().poll(context), Poll::Pending)
    }
}

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
        run(Box::pin(future));
        assert_eq!(*has_run.lock().unwrap(), true);
    }

}
