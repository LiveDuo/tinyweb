use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker}
};

use crate::bindings::window::set_timeout;

unsafe fn clone_arc_raw<T>(data: *const ()) -> RawWaker { RawWaker::new(data, waker_vtable::<T>()) }
unsafe fn wake_arc_raw(_data: *const ()) { set_timeout(|| { DEFAULT_EXECUTOR.with_borrow_mut(|s| s.poll_tasks()); }, 0); }
unsafe fn wake_by_ref_arc_raw<T>(_data: *const ()) { set_timeout(|| { DEFAULT_EXECUTOR.with_borrow_mut(|s| s.poll_tasks()); }, 0); }
unsafe fn drop_arc_raw<T>(data: *const ()) { drop(Rc::<T>::from_raw(data as *const T)) }

fn waker_vtable<W>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(clone_arc_raw::<W>, wake_arc_raw, wake_by_ref_arc_raw::<W>, drop_arc_raw::<W>)
}

type TasksList = VecDeque<Box<dyn Any>>;

pub struct Executor {
    tasks: Option<TasksList>,
}

struct Task<T> {
    future: RefCell<Pin<Box<dyn Future<Output = T> + 'static>>>,
}

impl<T: 'static> Task<T> {
    fn poll(&self) -> Poll<T> {
        let mut future = self.future.borrow_mut();

        let ptr = (self as *const Task<T>) as *const ();
        let waker =
            ManuallyDrop::new(unsafe { Waker::from_raw(RawWaker::new(ptr, waker_vtable::<Task<T>>())) });
        let context = &mut Context::from_waker(&*waker);
        future.as_mut().poll(context)
    }
}

impl Executor {
    fn run<T: 'static>(&mut self, future: Pin<Box<dyn Future<Output = T> + 'static>>) {
        self.add_task(future);
        self.poll_tasks();
    }

    fn add_task<T: 'static>(&mut self, future: Pin<Box<dyn Future<Output = T> + 'static>>) {
        let task = Rc::new(Task { future: RefCell::new(future) });
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

        let mut i = 0;
        while i < tasks.len() {
            let task = tasks.pop_front().unwrap();
            let task = task.downcast_ref::<Rc<Task<()>>>().unwrap();
            if task.poll().is_pending() {
                tasks.push_back(Box::new(task.clone()));
            }
            i += 1;
        }
    }
}

thread_local! {
    pub static DEFAULT_EXECUTOR: RefCell<Executor> = RefCell::new(Executor { tasks: None });
}

pub fn run<T: 'static>(future: impl Future<Output = T> + 'static) {
    DEFAULT_EXECUTOR.with_borrow_mut(|s| s.run(Box::pin(future)))
}

pub fn coroutine<T: 'static>(future: impl Future<Output = T> + 'static) {
    let mut a = Some(Box::pin(future));
    set_timeout(
        move || {
            let b = a.take();
            if let Some(b) = b {
                DEFAULT_EXECUTOR.with_borrow_mut(|s| s.run(b))
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
        let has_run = Rc::new(RefCell::new(false));
        let has_run_clone = has_run.clone();
        let future = async move {
            *has_run_clone.borrow_mut() = true;
        };
        DEFAULT_EXECUTOR.with_borrow_mut(|s| { s.run(Box::pin(future)); });
        assert_eq!(*has_run.borrow(), true);
    }

}
