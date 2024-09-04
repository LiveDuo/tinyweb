use std::{
    collections::VecDeque,
    future::Future,
    mem::{self, ManuallyDrop},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker}
};

use crate::bindings::window::set_timeout;

trait Woke: Send + Sync {
    fn wake(self: Arc<Self>) {
        Self::wake_by_ref(&self)
    }

    fn wake_by_ref(arc_self: &Arc<Self>);
}

unsafe fn clone_arc_raw<T: Woke>(data: *const ()) -> RawWaker {
    let arc = mem::ManuallyDrop::new(Arc::<T>::from_raw(data as *const T));
    let _arc_clone: mem::ManuallyDrop<_> = arc.clone();
    RawWaker::new(data, waker_vtable::<T>())
}

unsafe fn wake_arc_raw<T: Woke>(data: *const ()) {
    let arc: Arc<T> = Arc::from_raw(data as *const T);
    Woke::wake(arc);
}

// retain Arc, but don't touch refcount by wrapping in ManuallyDrop
unsafe fn wake_by_ref_arc_raw<T: Woke>(data: *const ()) {
    let arc = mem::ManuallyDrop::new(Arc::<T>::from_raw(data as *const T));
    Woke::wake_by_ref(&arc);
}

unsafe fn drop_arc_raw<T>(data: *const ()) {
    drop(Arc::<T>::from_raw(data as *const T))
}

fn waker_vtable<W: Woke>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(
        clone_arc_raw::<W>,
        wake_arc_raw::<W>,
        wake_by_ref_arc_raw::<W>,
        drop_arc_raw::<W>
    )
}

type TasksList = VecDeque<Box<dyn Pendable + Send + Sync>>;

pub struct Executor {
    tasks: Option<TasksList>,
}

trait Pendable {
    fn is_pending(&self) -> bool;
}

struct Task<T> {
    future: Mutex<Pin<Box<dyn Future<Output = T> + Send + 'static>>>,
}

impl<T> Woke for Task<T> {
    // tell the executor to poll for new things again but not recursively
    fn wake_by_ref(_: &Arc<Self>) {
        set_timeout(
            || {
                DEFAULT_EXECUTOR.lock().unwrap().poll_tasks();
            },
            0,
        );
    }
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

impl Executor {
    fn run<T: Send + Sync + 'static>(&mut self, future: Pin<Box<dyn Future<Output = T> + 'static + Send + Sync>>) {
        self.add_task(future);
        self.poll_tasks();
    }

    fn add_task<T: Send + Sync + 'static>(&mut self, future: Pin<Box<dyn Future<Output = T> + 'static + Send + Sync>>) {
        let task = Arc::new(Task { future: Mutex::new(future), });
        if self.tasks.is_none() {
            self.tasks = Some(TasksList::new());
        }
        let tasks: &mut TasksList = self.tasks.as_mut().expect("tasks not initialized");
        tasks.push_back(Box::new(task));
    }

    fn poll_tasks(&mut self) {
        if self.tasks.is_none() {
            self.tasks = Some(TasksList::new());
        }

        let tasks: &mut TasksList = self.tasks.as_mut().expect("tasks not initialized");
        if tasks.is_empty() { return; }

        for _ in 0..tasks.len() {
            let task = tasks.pop_front().unwrap();
            if task.is_pending() {
                tasks.push_back(task);
            }
        }
    }
}

static DEFAULT_EXECUTOR: Mutex<Executor> = Mutex::new(Executor { tasks: None });

pub fn run<T: Send + Sync + 'static>(future: impl Future<Output = T> + 'static + Send + Sync) {
    DEFAULT_EXECUTOR.lock().unwrap().run(Box::pin(future))
}

pub fn coroutine<T: Send + Sync + 'static>(future: impl Future<Output = T> + 'static + Send + Sync) {
    let mut a = Some(Box::pin(future));
    set_timeout(
        move || {
            let b = a.take();
            if let Some(b) = b {
                DEFAULT_EXECUTOR.lock().unwrap().run(b);
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
        DEFAULT_EXECUTOR.lock().unwrap().run(Box::pin(future));
        assert_eq!(*has_run.lock().unwrap(), true);
    }

}
