use std::{
    collections::VecDeque,
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker}
};

use crate::bindings::window::set_timeout;

trait Pendable {
    fn is_pending(&self) -> bool;
}

pub struct Runtime {
    tasks: VecDeque<Box<dyn Pendable + Send + Sync>>,
}

struct Task<T> {
    future: Mutex<Pin<Box<dyn Future<Output = T> + Send + 'static>>>,
}

impl<T> Pendable for Arc<Task<T>> {
    fn is_pending(&self) -> bool {
        let mut future = self.future.lock().unwrap();

        fn clone_arc_raw<W>(data: *const ()) -> RawWaker {
            RawWaker::new(data, waker_vtable::<W>())
        }
        fn wake_arc_raw(_data: *const ()) {
            set_timeout(|| { DEFAULT_RUNTIME.lock().unwrap().poll_tasks(); }, 0);
        }
        fn drop_arc_raw<W>(data: *const ()) {
            unsafe { drop(Arc::<W>::from_raw(data as *const W)) }
        }
        fn waker_vtable<W>() -> &'static RawWakerVTable {
            &RawWakerVTable::new(clone_arc_raw::<W>, wake_arc_raw, wake_arc_raw, drop_arc_raw::<W>)
        }

        let ptr = (&**self as *const Task<T>) as *const ();
        let raw_waker = RawWaker::new(ptr, waker_vtable::<Task<T>>());
        let waker = ManuallyDrop::new(unsafe { Waker::from_raw(raw_waker) });
        let context = &mut Context::from_waker(&*waker);
        matches!(future.as_mut().poll(context), Poll::Pending)
    }
}

impl Runtime {

    fn add_task<T: Send + Sync + 'static>(&mut self, future: Pin<Box<dyn Future<Output = T> + 'static + Send + Sync>>) {
        let task = Arc::new(Task { future: Mutex::new(future), });
        self.tasks.push_back(Box::new(task));
    }

    fn poll_tasks(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        for _ in 0..self.tasks.len() {
            let task = self.tasks.pop_front().unwrap();
            if task.is_pending() {
                self.tasks.push_back(task);
            }
        }
    }
}

static DEFAULT_RUNTIME: Mutex<Runtime> = Mutex::new(Runtime { tasks: VecDeque::new() });

pub fn run<T: Send + Sync + 'static>(future: impl Future<Output = T> + 'static + Send + Sync) {
    DEFAULT_RUNTIME.lock().map(|mut s| {
        s.add_task(Box::pin(future));
        s.poll_tasks();
    }).unwrap()
}

pub fn coroutine<T: Send + Sync + 'static>(future: impl Future<Output = T> + 'static + Send + Sync) {
    let mut a = Some(Box::pin(future));
    set_timeout(
        move || {
            let b = a.take();
            if let Some(b) = b {
                DEFAULT_RUNTIME.lock().map(|mut s| {
                    s.add_task(Box::pin(b));
                    s.poll_tasks();
                }).unwrap()
            }
        },
        0,
    );
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
