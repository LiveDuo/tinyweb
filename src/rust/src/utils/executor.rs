use std::{marker::PhantomData, mem::{self, ManuallyDrop}, ops::Deref, sync::Mutex, task::{RawWaker, RawWakerVTable, Waker}};

extern crate alloc;
use crate::bindings::window::set_timeout;

use {
    alloc::{boxed::Box, collections::vec_deque::VecDeque, sync::Arc},
    core::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
    },
};

#[derive(Debug)]
pub struct WakerRef<'a> {
    waker: ManuallyDrop<Waker>,
    _marker: PhantomData<&'a ()>,
}

pub trait Woke: Send + Sync {
    fn wake(self: Arc<Self>) {
        Self::wake_by_ref(&self)
    }

    fn wake_by_ref(arc_self: &Arc<Self>);
}

impl<'a> WakerRef<'a> {
    pub fn new(waker: &'a Waker) -> Self {
        let waker = ManuallyDrop::new(unsafe { core::ptr::read(waker) });
        WakerRef {
            waker,
            _marker: PhantomData,
        }
    }

    pub fn new_unowned(waker: ManuallyDrop<Waker>) -> Self {
        WakerRef {
            waker,
            _marker: PhantomData,
        }
    }
}

unsafe fn increase_refcount<T: Woke>(data: *const ()) {
    let arc = mem::ManuallyDrop::new(Arc::<T>::from_raw(data as *const T));
    let _arc_clone: mem::ManuallyDrop<_> = arc.clone();
}

unsafe fn clone_arc_raw<T: Woke>(data: *const ()) -> RawWaker {
    increase_refcount::<T>(data);
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

unsafe fn drop_arc_raw<T: Woke>(data: *const ()) {
    drop(Arc::<T>::from_raw(data as *const T))
}

pub fn waker_vtable<W: Woke>() -> &'static RawWakerVTable {
    &RawWakerVTable::new(
        clone_arc_raw::<W>,
        wake_arc_raw::<W>,
        wake_by_ref_arc_raw::<W>,
        drop_arc_raw::<W>,
    )
}

#[inline]
pub fn waker_ref<W>(wake: &Arc<W>) -> WakerRef<'_>
where
    W: Woke,
{
    let ptr = (&**wake as *const W) as *const ();

    let waker =
        ManuallyDrop::new(unsafe { Waker::from_raw(RawWaker::new(ptr, waker_vtable::<W>())) });
    WakerRef::new_unowned(waker)
}

impl Deref for WakerRef<'_> {
    type Target = Waker;

    fn deref(&self) -> &Waker {
        &self.waker
    }
}

type TasksList = VecDeque<Box<dyn Pendable + core::marker::Send + core::marker::Sync>>;

pub struct Executor {
    tasks: Option<TasksList>,
}

trait Pendable {
    fn is_pending(&self) -> bool;
}

struct Task<T> {
    pub future: Mutex<Pin<Box<dyn Future<Output = T> + Send + 'static>>>,
}

impl<T> Woke for Task<T> {
    // tell the executor to poll for new things again but not recursively
    fn wake_by_ref(_: &Arc<Self>) {
        set_timeout(
            || {
                poll_tasks();
            },
            0,
        );
    }
}

impl<T> Pendable for Arc<Task<T>> {
    fn is_pending(&self) -> bool {
        let mut future = self.future.lock().unwrap();
        let waker = waker_ref(self);
        let context = &mut Context::from_waker(&*waker);
        matches!(future.as_mut().poll(context), Poll::Pending)
    }
}

impl Executor {
    pub fn run<T>(&mut self, future: Pin<Box<dyn Future<Output = T> + 'static + Send + Sync>>)
    where
        T: Send + Sync + 'static,
    {
        self.add_task(future);
        self.poll_tasks();
    }

    fn add_task<T>(&mut self, future: Pin<Box<dyn Future<Output = T> + 'static + Send + Sync>>)
    where
        T: Send + Sync + 'static,
    {
        let task = Arc::new(Task {
            future: Mutex::new(future),
        });
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
        if tasks.is_empty() {
            return;
        }
        for _ in 0..tasks.len() {
            let task = tasks.pop_front().unwrap();
            if task.is_pending() {
                tasks.push_back(task);
            }
        }
    }
}

static DEFAULT_EXECUTOR: Mutex<Executor> = Mutex::new(Executor { tasks: None });

pub fn run<T>(future: impl Future<Output = T> + 'static + Send + Sync)
where
    T: Send + Sync + 'static,
{
    DEFAULT_EXECUTOR.lock().unwrap().run(Box::pin(future))
}

pub fn poll_tasks() {
    DEFAULT_EXECUTOR.lock().unwrap().poll_tasks()
}

pub fn coroutine<T>(future: impl Future<Output = T> + 'static + Send + Sync)
where
    T: Send + Sync + 'static,
{
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
