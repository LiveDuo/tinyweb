use std::{
    collections::VecDeque,
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker}
};

use crate::bindings::window::set_timeout;

fn clone_arc_raw<T: Send>(data: *const ()) -> RawWaker {
    let _arc = unsafe { ManuallyDrop::new(Arc::<T>::from_raw(data as *const T)).clone() };
    RawWaker::new(data, waker_vtable::<T>())
}
fn wake_arc_raw<T: Send>(_data: *const ()) {
    set_timeout(|| { DEFAULT_RUNTIME.lock().unwrap().poll_tasks(); }, 0);
}
fn drop_arc_raw<T>(data: *const ()) {
    unsafe { drop(Arc::<T>::from_raw(data as *const T)) }
}
fn waker_vtable<W: Send>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(clone_arc_raw::<W>, wake_arc_raw::<W>, wake_arc_raw::<W>, drop_arc_raw::<W>)
}

trait Pendable {
    fn is_pending(&self) -> bool;
}

type TasksList = VecDeque<Box<dyn Pendable + Send>>;

pub struct Runtime {
    tasks: TasksList,
}

struct Task<T> {
    future: Mutex<Pin<Box<dyn Future<Output = T> + Send + 'static>>>,
}

impl<T> Pendable for Arc<Task<T>> {
    fn is_pending(&self) -> bool {
        let mut future = self.future.lock().unwrap();

        let ptr = (&**self as *const Task<T>) as *const ();
        let waker =
            ManuallyDrop::new(unsafe { Waker::from_raw(RawWaker::new(ptr, waker_vtable::<Task<T>>())) });
        let context = &mut Context::from_waker(&*waker);
        matches!(future.as_mut().poll(context), Poll::Pending)
    }
}

impl Runtime {

    fn add_task<T: Send + 'static>(&mut self, future: Pin<Box<dyn Future<Output = T> + Send + 'static>>) {
        let task = Arc::new(Task { future: Mutex::new(future), });
        self.tasks.push_back(Box::new(task));
    }

    fn poll_tasks(&mut self) {
        if self.tasks.is_empty() {
            self.tasks = TasksList::new();
        }

        if self.tasks.is_empty() { return; }

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
        s.add_task(Box::pin(future));
        s.poll_tasks();
    }).unwrap();
}

pub fn coroutine<T: Send + 'static>(future: impl Future<Output = T> + Send + 'static) {
    let mut a = Some(Box::pin(future));
    set_timeout(
        move || {
            let b = a.take();
            if let Some(b) = b {
                run(b);
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
