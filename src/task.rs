use crate::serial_println;
use alloc::boxed::Box;
use core::future::Future;
use core::pin::Pin;
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

pub mod executor;
pub mod waker;

static TASK_ID: AtomicU64 = AtomicU64::new(1);

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct TaskId(u64);

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        let task_id = TaskId(TASK_ID.fetch_add(1, Ordering::Relaxed));
        Task {
            id: task_id,
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, ctx: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(ctx)
    }
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(dummy_raw_waker, wake, wake_by_ref, drop);

unsafe fn dummy_raw_waker(_: *const ()) -> RawWaker {
    RawWaker::new(core::ptr::null(), &VTABLE)
}

fn dummy_waker() -> Waker {
    let raw_waker = unsafe { dummy_raw_waker(core::ptr::null()) };
    unsafe { Waker::from_raw(raw_waker) }
}

unsafe fn wake(_: *const ()) {}
unsafe fn wake_by_ref(_: *const ()) {}
unsafe fn drop(_: *const ()) {}


pub async fn sample_async_task() {
    serial_println!("This is sample async task");
}
